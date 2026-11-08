//! Role: Run `cargo xtask devtools-codegen`, or check the committed artifacts with `--check`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Run the generator. `--check` only compares; any other argument is an error.
pub(crate) fn run(root: &Path, arguments: &[String]) -> ExitCode {
    let check = arguments.iter().any(|argument| argument == "--check");
    if let Some(unknown) = arguments
        .iter()
        .find(|argument| argument.as_str() != "--check")
    {
        return fail(&format!("unknown argument: {unknown}"));
    }
    if check {
        return check_files(root);
    }
    match super::generate(root) {
        Ok(files) => write_files(&files),
        Err(error) => fail(&error),
    }
}

fn check_files(root: &Path) -> ExitCode {
    let errors = super::check(root);
    if errors.is_empty() {
        return ExitCode::SUCCESS;
    }
    let mut stderr = std::io::stderr().lock();
    for error in &errors {
        if writeln!(stderr, "{error}").is_err() {
            return ExitCode::FAILURE;
        }
    }
    ExitCode::FAILURE
}

fn write_files(files: &[(PathBuf, String)]) -> ExitCode {
    let mut stdout = std::io::stdout().lock();
    for (path, contents) in files {
        if let Some(parent) = path.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            return fail(&format!("{}: {error}", parent.display()));
        }
        if let Err(error) = std::fs::write(path, contents) {
            return fail(&format!("{}: {error}", path.display()));
        }
        if writeln!(stdout, "{}", path.display()).is_err() {
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn fail(message: &str) -> ExitCode {
    let mut stderr = std::io::stderr().lock();
    if writeln!(stderr, "{message}").is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::FAILURE
}
