//! Role: The Scripts panel.

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// Which stream a line came from.
#[derive(Clone, Copy)]
enum Stream {
    /// The standard output.
    Out,
    /// The standard error.
    Err,
}

/// The scripts, their output and their tables.
pub(crate) fn view(session: Session) -> impl IntoView {
    let directory = script_dir(session);
    let scripts = signal(script_files(&directory));
    let selected = signal(None::<PathBuf>);
    let stdout = signal(String::new());
    let stderr = signal(String::new());
    let items = memo(move || {
        scripts
            .get()
            .into_iter()
            .map(|path| {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                ListItem::new(path, name)
            })
            .collect::<Vec<_>>()
    });
    let interpreter = interpreter(session);
    column((
        row((
            text(format!("Scripts in {}", directory.display())),
            button("Refresh").on_press(move || scripts.set(script_files(&directory))),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        show(move || scripts.get().is_empty(), || text("No .py scripts.")),
        list("Scripts", items).selection(selected),
        row((
            text("Interpreter"),
            text(interpreter.clone()),
            button("Run")
                .disabled(move || selected.get().is_none())
                .on_press(move || {
                    run_script(session, selected, interpreter.clone(), stdout, stderr);
                }),
        ))
        .style(style().gap(8).align_items(Align::Center)),
        show(
            move || !stdout.get().is_empty(),
            move || kivact::ui::code_block(stdout).title("Output"),
        ),
        show(
            move || !stderr.get().is_empty(),
            move || kivact::ui::code_block(stderr).title("Errors"),
        ),
        for_index(session.tables, |_, table| table_view(table.get())),
    ))
    .style(style().gap(8).padding(12).align_items(Align::Start))
}

/// Run the selected script and show its streams line by line.
fn run_script(
    session: Session,
    selected: Signal<Option<PathBuf>>,
    interpreter: String,
    stdout: Signal<String>,
    stderr: Signal<String>,
) {
    let Some(path) = selected.get_untracked() else {
        return;
    };
    stdout.set(String::new());
    stderr.set(String::new());
    let mut command = std::process::Command::new(&interpreter);
    command
        .arg(&path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(target) = session.target.with(|target| target.path.clone()) {
        command.env("KIVACT_DEVTOOLS_TARGET", target);
    }
    command.env("PYTHONIOENCODING", "utf-8");
    spawn(async move {
        let mut child = match blocking(move || command.spawn()).await {
            Ok(child) => child,
            Err(error) => {
                session.report(format!("run: {error}"));
                return;
            }
        };
        let (sender, receiver) = std::sync::mpsc::channel::<(Stream, String)>();
        let out = child.stdout.take();
        let err = child.stderr.take();
        let out_sender = sender.clone();
        std::thread::spawn(move || read_lines(out, Stream::Out, out_sender));
        std::thread::spawn(move || read_lines(err, Stream::Err, sender));
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));
        loop {
            let receiver = receiver.clone();
            let line = blocking(move || {
                receiver
                    .lock()
                    .expect("invariant: the output receiver is not poisoned")
                    .recv()
            })
            .await;
            match line {
                Ok((Stream::Out, line)) => stdout.update(|text| push_line(text, line)),
                Ok((Stream::Err, line)) => stderr.update(|text| push_line(text, line)),
                Err(_) => break,
            }
        }
        match blocking(move || child.wait()).await {
            Ok(status) if status.success() => {}
            Ok(status) => session.report(format!("the script exited with {status}")),
            Err(error) => session.report(format!("run: {error}")),
        }
    });
}

/// Read one stream to its end and send every line.
fn read_lines<R: std::io::Read>(
    reader: Option<R>,
    stream: Stream,
    sender: std::sync::mpsc::Sender<(Stream, String)>,
) {
    let Some(reader) = reader else {
        return;
    };
    for line in std::io::BufReader::new(reader).lines() {
        match line {
            Ok(line) => {
                if sender.send((stream, line)).is_err() {
                    break;
                }
            }
            Err(error) => {
                let _ = sender.send((stream, format!("<read error: {error}>")));
                break;
            }
        }
    }
}

/// Append one line and its newline.
fn push_line(text: &mut String, line: String) {
    text.push_str(&line);
    text.push('\n');
}

/// The directory the scripts are read from.
fn script_dir(session: Session) -> PathBuf {
    let configured = session.config().scripts_dir();
    let path = Path::new(&configured);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        Path::new(&session.workspace_root()).join(path)
    }
}

/// The `.py` files of `directory`, sorted by path.
fn script_files(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.extension().is_some_and(|extension| extension == "py")
        })
        .collect();
    files.sort();
    files
}

/// The interpreter of the scripts (the configuration, the workspace venv, PATH).
fn interpreter(session: Session) -> String {
    if let Some(python) = &session.config().python {
        return python.clone();
    }
    let workspace_root = session.workspace_root();
    let root = Path::new(&workspace_root);
    for candidate in [".venv/Scripts/python.exe", ".venv/bin/python"] {
        let path = root.join(candidate);
        if path.is_file() {
            return path.to_string_lossy().into_owned();
        }
    }
    for name in ["python3", "python"] {
        if on_path(name) {
            return name.to_owned();
        }
    }
    "python".to_owned()
}

/// Whether `name` is an executable in a `PATH` directory.
fn on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|directory| executable(&directory, name))
}

/// Whether `directory` holds an executable named `name`.
fn executable(directory: &Path, name: &str) -> bool {
    directory.join(name).is_file() || directory.join(format!("{name}.exe")).is_file()
}

/// One table a script showed.
fn table_view(table: p::ShowTableParams) -> View {
    let p::ShowTableParams {
        title,
        columns,
        rows,
    } = table;
    let rows: Vec<(usize, Vec<String>)> = rows.into_iter().enumerate().collect();
    let table_columns = columns
        .iter()
        .enumerate()
        .map(|(index, name)| {
            let name = name.clone();
            DataColumn::new(name, move |row: &(usize, Vec<String>)| {
                row.1.get(index).cloned().unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();
    column((
        text(title),
        data_table(rows, |row: &(usize, Vec<String>)| row.0, table_columns),
    ))
    .into_view()
}
