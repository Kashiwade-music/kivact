//! Role: Write the architecture diagrams on request; the check never writes them.

use std::path::Path;
use std::process::ExitCode;

/// `cargo xtask diagrams [--root PATH] [--prune]`: run xtask-core with writing on (needs Node.js).
pub(crate) fn run(root: &Path, arguments: &[String]) -> ExitCode {
    let mut shared = vec!["check".to_owned()];
    shared.extend(arguments.iter().cloned());
    let mut config = match xtask_core::parse_args(root, shared) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    config.no_write = false;
    if let Err(error) = crate::policy::prepare(&mut config) {
        eprintln!("error: {error}");
        return ExitCode::FAILURE;
    }
    match xtask_core::run(&config) {
        Ok(mut report) => {
            crate::core_filters::apply(&config.root, &mut report.errors);
            crate::report::print(&report)
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
