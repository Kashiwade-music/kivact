//! Role: Print the collected workspace check result and choose its exit code.
use std::process::ExitCode;

pub(crate) fn print(report: &xtask_core::Report) -> ExitCode {
    for note in &report.notes {
        println!("{note}");
    }
    for error in &report.errors {
        eprintln!("error: {error}");
    }
    if report.is_success() {
        println!("ok");
        ExitCode::SUCCESS
    } else {
        eprintln!("{} error(s)", report.errors.len());
        ExitCode::FAILURE
    }
}
