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

mod impl_model;
mod macros;
mod module_rules;

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("dev/xtask is two levels below the repository root");
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h"))
    {
        return xtask_core::main(root);
    }
    let mut config = match xtask_core::parse_args(root, arguments) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(2);
        }
    };
    let rules = match prepare(&mut config) {
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

/// Split `[module_rules]` off the policy, point `config` at the rest and
/// trust the workspace's own macros.
fn prepare(config: &mut xtask_core::Config) -> Result<module_rules::Rules, String> {
    let path = config.policy.clone();
    let located = |error: String| format!("{}: {error}", path.display());
    let text = std::fs::read_to_string(&path).map_err(|error| located(error.to_string()))?;
    let mut table: toml::Table =
        toml::from_str(&text).map_err(|error| located(error.to_string()))?;
    let rules = module_rules::Rules::parse(table.remove("module_rules"))
        .map_err(|error| located(format!("module_rules: {error}")))?;
    let core = toml::to_string(&table).map_err(|error| error.to_string())?;
    let mut analysis = xtask_core::Policy::from_toml(&core)
        .map_err(located)?
        .analysis;
    analysis
        .trusted_macros
        .extend(macros::workspace_macros(&config.root)?);
    let derived = config.root.join("target/xtask/workspace.toml");
    if let Some(parent) = derived.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    std::fs::write(&derived, core).map_err(|error| format!("{}: {error}", derived.display()))?;
    config.policy = derived;
    config.analysis = Some(analysis);
    Ok(rules)
}
