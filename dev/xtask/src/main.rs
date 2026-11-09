//! Role: Run the shared xtask-core checks plus this workspace's module rules.
//!
//! `dev/xtask/policy/workspace.toml` holds both the xtask-core policy and
//! the `[module_rules]` this binary adds (`module_rules.rs`). xtask-core
//! rejects tables it does not know, so it reads the policy without
//! `[module_rules]` from `target/xtask/workspace.toml`, with the paths
//! that invoke the workspace's `macro_rules!` added to
//! `analysis.trusted_macros` (`macros.rs`).

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "the check reports its notes and errors on the console"
)]

use std::path::Path;
use std::process::ExitCode;

mod api_shape;
mod canonical_path;
mod check_cli;
mod check_policy;
mod check_source;
mod devtools_codegen;
mod effect_writes;
mod impl_model;
mod kit_literal_color;
mod local_checks;
mod macros;
mod module_rules;
mod name_collision;
mod os_branch;
mod policy;
mod public_surface;
mod public_type_edges;
mod report;
mod state_fields;
mod structural_options;

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("dev/xtask is two levels below the repository root");
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("devtools-codegen") {
        return devtools_codegen::run(root, &arguments[1..]);
    }
    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        println!(
            "Local checks: --only <{}> [--crate <crate> [--module <path>]] [--report]",
            check_cli::RULES.join("|")
        );
        return xtask_core::main(root);
    }
    let (options, arguments) = match check_cli::Options::parse(arguments) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let mut config = match xtask_core::parse_args(root, arguments) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let local = match local_checks::check(&config.root, &config.policy, &options) {
        Ok(errors) => errors,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    if options.only.is_some() {
        for error in &local {
            if options.report {
                println!("{error}");
            } else {
                eprintln!("error: {error}");
            }
        }
        return if local.is_empty() || options.report {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    let rules = match policy::prepare(&mut config) {
        Ok(rules) => rules,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    let mut report = match xtask_core::run(&config) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    match xtask_core::analyze(&config) {
        Ok(analysis) => report
            .errors
            .extend(module_rules::check(&rules, &analysis.targets)),
        Err(error) => report.errors.push(format!("module rules: {error}")),
    }
    report.errors.extend(impl_model::check(&config.root));
    if options.report {
        report.notes.extend(local);
    } else {
        report.errors.extend(local);
    }
    report::print(&report)
}
