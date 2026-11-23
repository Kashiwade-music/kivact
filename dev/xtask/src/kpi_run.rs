//! Role: Run dev/bench/run.sh through bash, for runners that start only cargo.
//!
//! `cargo xtask kpi-run [--no-catalog] <label>` runs the KPI script with the
//! same arguments and returns its exit status. The script's own stdout is
//! dropped (it echoes only part of the report: some rows go to the report file
//! alone), and the whole report file `dev/bench/results/<label>.md` is printed
//! on stdout afterwards, so a runner that keeps the command's output keeps the
//! report, INVALID or not. On Windows the bash on PATH can be WSL's, so Git for
//! Windows' bash is taken from `git --exec-path`.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

/// Run `dev/bench/run.sh` with `arguments`.
pub(crate) fn run(root: &Path, arguments: &[String]) -> ExitCode {
    if arguments.is_empty() {
        eprintln!("usage: cargo xtask kpi-run [--no-catalog] <label>");
        return ExitCode::from(2);
    }
    let bash = match bash() {
        Ok(bash) => bash,
        Err(error) => {
            eprintln!("kpi-run: {error}");
            return ExitCode::from(2);
        }
    };
    let status = Command::new(&bash)
        .arg("dev/bench/run.sh")
        .args(arguments)
        .current_dir(root)
        .stdout(Stdio::null())
        .status();
    print_report(root, arguments);
    match status {
        Ok(status) => match status.code() {
            Some(0) => ExitCode::SUCCESS,
            Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
            None => ExitCode::FAILURE,
        },
        Err(error) => {
            eprintln!("kpi-run: cannot start {}: {error}", bash.display());
            ExitCode::from(2)
        }
    }
}

/// Print the report the script wrote; the label is the last argument.
fn print_report(root: &Path, arguments: &[String]) {
    let Some(label) = arguments.last() else {
        return;
    };
    let report = root.join("dev/bench/results").join(format!("{label}.md"));
    match std::fs::read_to_string(&report) {
        Ok(text) => print!("{text}"),
        Err(error) => eprintln!("kpi-run: cannot read {}: {error}", report.display()),
    }
}

/// The bash that runs the script: the first `bin/bash.exe` or `bin/bash`
/// above git's exec path (`<git>/mingw64/libexec/git-core` gives Git for
/// Windows' `<git>/bin/bash.exe`, `/usr/lib/git-core` gives `/usr/bin/bash`),
/// else `bash` on PATH.
fn bash() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .arg("--exec-path")
        .output()
        .map_err(|e| format!("git is needed to find bash: {e}"))?;
    let exec_path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let found = Path::new(&exec_path).ancestors().find_map(|dir| {
        ["bash.exe", "bash"]
            .iter()
            .map(|name| dir.join("bin").join(name))
            .find(|candidate| candidate.is_file())
    });
    Ok(found.unwrap_or_else(|| PathBuf::from("bash")))
}
