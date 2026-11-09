//! Role: The Scripts panel.

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::{Session, push_capped};

/// The most lines kept per output stream.
const MAX_LINES: usize = 2000;

/// Which stream a line came from.
#[derive(Clone, Copy)]
enum Stream {
    /// The standard output.
    Out,
    /// The standard error.
    Err,
}

/// Whether a script runs.
#[derive(Clone)]
enum ScriptRun {
    /// No script runs.
    Idle,
    /// The named script runs, as `child` (so that the panel can kill it).
    Running {
        /// The file name of the script.
        script: String,
        /// The child process of the run.
        child: Arc<Mutex<Child>>,
    },
}

impl PartialEq for ScriptRun {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ScriptRun::Idle, ScriptRun::Idle) => true,
            (
                ScriptRun::Running {
                    script: a,
                    child: child_a,
                },
                ScriptRun::Running {
                    script: b,
                    child: child_b,
                },
            ) => a == b && Arc::ptr_eq(child_a, child_b),
            _ => false,
        }
    }
}

/// The run of the selected script: its state and its output.
#[derive(Clone)]
struct Runner {
    /// Whether a script runs.
    state: Signal<ScriptRun>,
    /// The standard output lines, oldest first.
    stdout: Signal<Vec<String>>,
    /// The standard error lines, oldest first.
    stderr: Signal<Vec<String>>,
}

/// What the scripts read from the session, as plain data so that the disk is read off the UI thread.
struct Probe {
    /// The directory of the scripts.
    directory: PathBuf,
    /// The configured interpreter.
    python: Option<String>,
    /// The workspace root of the target.
    workspace_root: PathBuf,
}

impl Probe {
    /// What the scripts of `session` are read from.
    fn of(session: Session) -> Probe {
        Probe {
            directory: script_dir(session),
            python: session.config().python.clone(),
            workspace_root: PathBuf::from(session.workspace_root()),
        }
    }
}

/// The scripts, their output and their tables.
pub(crate) fn view(session: Session) -> impl IntoView {
    let directory = script_dir(session);
    let scripts = signal(Vec::<PathBuf>::new());
    let interpreter = signal(String::new());
    let selected = signal(None::<PathBuf>);
    let runner = Runner {
        state: signal(ScriptRun::Idle),
        stdout: signal(Vec::new()),
        stderr: signal(Vec::new()),
    };
    on_mount(move || load(Probe::of(session), scripts, interpreter));
    let cleanup = runner.clone();
    on_cleanup(move || kill_running(&cleanup));
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
    let state = runner.state;
    let stdout = runner.stdout;
    let stderr = runner.stderr;
    let press = runner.clone();
    scroll(
        column((
            row((
                text(format!("Scripts in {}", directory.display())),
                button("Refresh").on_press(move || load(Probe::of(session), scripts, interpreter)),
            ))
            .style(style().gap(8).align_items(Align::Center)),
            show(
                move || scripts.with(Vec::is_empty),
                || text("No .py scripts."),
            ),
            list("Scripts", items, selected),
            row((
                text("Interpreter"),
                text(move || interpreter.get()),
                text(move || {
                    state.with(|state| match state {
                        ScriptRun::Idle => String::new(),
                        ScriptRun::Running { script, .. } => format!("Running {script}"),
                    })
                }),
                button("Run")
                    .disabled(move || {
                        selected.with(Option::is_none)
                            || interpreter.with(String::is_empty)
                            || state.with(|state| matches!(state, ScriptRun::Running { .. }))
                    })
                    .on_press(move || {
                        if let Some(path) = selected.get_untracked() {
                            run_script(session, press.clone(), path, interpreter.get_untracked());
                        }
                    }),
            ))
            .style(style().gap(8).align_items(Align::Center)),
            show(
                move || stdout.with(|lines| !lines.is_empty()),
                move || kivact::ui::code_block(joined(stdout)).title("Output"),
            ),
            show(
                move || stderr.with(|lines| !lines.is_empty()),
                move || kivact::ui::code_block(joined(stderr)).title("Errors"),
            ),
            for_index(session.tables, |_, table| table_view(table.get())),
        ))
        .style(style().gap(8).padding(12).align_items(Align::Start)),
    )
    .style(style().grow(1.0).min_height(0.0))
}

/// The lines of one stream as one text.
fn joined(lines: Signal<Vec<String>>) -> Value<String> {
    memo(move || lines.with(|lines| lines.join("\n")))
}

/// Run the selected script and show its streams line by line; the run ends when the process exits.
fn run_script(session: Session, runner: Runner, path: PathBuf, interpreter: String) {
    runner.stdout.set(Vec::new());
    runner.stderr.set(Vec::new());
    let script = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
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
        let child = Arc::new(Mutex::new(child));
        runner.state.set(ScriptRun::Running {
            script,
            child: child.clone(),
        });
        let receiver = Arc::new(Mutex::new(receiver));
        loop {
            let receiver = receiver.clone();
            let line = blocking(move || {
                receiver
                    .lock()
                    .expect("invariant: the output receiver is not poisoned")
                    .recv()
            })
            .await;
            if runner.state.try_get_untracked().is_none() {
                break;
            }
            match line {
                Ok((Stream::Out, line)) => {
                    runner
                        .stdout
                        .update(|lines| push_capped(lines, line, MAX_LINES));
                }
                Ok((Stream::Err, line)) => {
                    runner
                        .stderr
                        .update(|lines| push_capped(lines, line, MAX_LINES));
                }
                Err(_) => break,
            }
        }
        let status = blocking(move || wait_for_exit(&child)).await;
        let alive = runner.state.try_get_untracked().is_some();
        runner.state.set(ScriptRun::Idle);
        match status {
            _ if !alive => {}
            Ok(status) if status.success() => {}
            Ok(status) => session.report(format!("the script exited with {status}")),
            Err(error) => session.report(format!("run: {error}")),
        }
    });
}

/// Wait until the child exits, holding its lock only for each check so that the panel can kill it.
fn wait_for_exit(child: &Mutex<Child>) -> std::io::Result<std::process::ExitStatus> {
    loop {
        let checked = child
            .lock()
            .expect("invariant: the child lock is not poisoned")
            .try_wait()?;
        if let Some(status) = checked {
            return Ok(status);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Kill the running script, if any; called when the panel goes away.
fn kill_running(runner: &Runner) {
    let child = runner
        .state
        .try_get_untracked()
        .and_then(|state| match state {
            ScriptRun::Idle => None,
            ScriptRun::Running { child, .. } => Some(child),
        });
    if let Some(child) = child
        && let Ok(mut child) = child.lock()
    {
        // The process may have exited already; there is nothing to stop then.
        let _ = child.kill();
    }
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

/// Read the scripts and find the interpreter, then show them; the disk is read on a worker thread.
fn load(probe: Probe, scripts: Signal<Vec<PathBuf>>, interpreter: Signal<String>) {
    spawn(async move {
        let (files, found) = blocking(move || {
            let files = script_files(&probe.directory);
            let found = find_interpreter(probe.python.as_deref(), &probe.workspace_root);
            (files, found)
        })
        .await;
        scripts.set(files);
        interpreter.set(found);
    });
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
fn find_interpreter(python: Option<&str>, root: &Path) -> String {
    if let Some(python) = python {
        return python.to_owned();
    }
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
