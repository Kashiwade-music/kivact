//! Role: Run the shared xtask-core checks plus local checks; draw diagrams on request.
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
mod core_filters;
mod diagrams;
mod docs_pairs;
mod effect_writes;
mod element_shadow;
mod hand_element;
mod impl_model;
mod interaction_view;
mod kit_literal_color;
mod kpi_compare;
mod kpi_run;
mod local_checks;
mod macros;
mod module_rules;
mod name_collision;
mod os_branch;
mod policy;
mod public_surface;
mod public_type_edges;
mod report;
mod state;
mod ui_machine;
mod unit_machine;

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("dev/xtask is two levels below the repository root");
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments.first().map(String::as_str) == Some("diagrams") {
        return diagrams::run(root, &arguments[1..]);
    }
    if arguments.first().map(String::as_str) == Some("kpi-compare") {
        return kpi_compare::run(root, &arguments[1..]);
    }
    if arguments.first().map(String::as_str) == Some("kpi-run") {
        return kpi_run::run(root, &arguments[1..]);
    }
    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        return check_cli::help(root);
    }
    let (options, arguments) = match check_cli::Options::parse(arguments) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let mut config = match xtask_core::parse_args(root, arguments) {
        Ok(xtask_core::Command::Check(config)) => *config,
        Ok(xtask_core::Command::Help) => return check_cli::help(root),
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    // The check never writes the diagram file; `cargo xtask diagrams` does.
    if config.output != xtask_core::Output::None {
        eprintln!("`cargo xtask check` writes nothing; use `cargo xtask diagrams`");
        return ExitCode::from(2);
    }
    let local = match local_checks::check(&config.root, &config.policy, &options) {
        Ok(errors) => errors,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::FAILURE;
        }
    };
    if options.only.is_some() || options.package.is_some() {
        return local_checks::selected(&options, &local);
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
    core_filters::apply(&config.root, &mut report.errors);
    match xtask_core::analyze(&config) {
        Ok((targets, _)) => report.errors.extend(module_rules::check(&rules, &targets)),
        Err(error) => report.errors.push(format!("module rules: {error}")),
    }
    report.errors.extend(impl_model::check(&config.root));
    if options.report {
        report.notes.extend(local.errors);
    } else {
        report.errors.extend(local.errors);
    }
    report::print(&report)
}
