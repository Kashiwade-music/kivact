//! Role: Run the workspace's local architectural checks under shared selectors.
use crate::check_cli::{Options, RULES};
use crate::check_policy::is_kit;
use crate::pending::Pending;
use std::path::Path;

/// The findings of one run: errors fail it; notes are findings of a pending rule.
pub(crate) struct Outcome {
    pub(crate) errors: Vec<String>,
    pub(crate) notes: Vec<String>,
}

pub(crate) fn check(root: &Path, policy: &Path, options: &Options) -> Result<Outcome, String> {
    let text = std::fs::read_to_string(policy).map_err(|e| format!("{}: {e}", policy.display()))?;
    let policy: toml::Table = text.parse().map_err(|e| format!("{e}"))?;
    validate_policy(&policy)?;
    let pending = Pending::parse(&policy)?;
    let sources = crate::check_source::load(root)?;
    if let Some(package) = &options.package
        && !sources.iter().any(|s| s.package == *package)
    {
        return Err(format!("unknown crate: {package}"));
    }
    let active = |rule: &str| {
        options.selected(rule)
            && sources.iter().any(|source| {
                let kit_only = matches!(rule, "kit-literal-color" | "name-collision");
                (!kit_only || is_kit(&source.package)) && selected_source(source, options)
            })
    };
    let mut outcome = Outcome {
        errors: Vec::new(),
        notes: Vec::new(),
    };
    if !RULES.iter().any(|rule| active(rule)) {
        return Ok(outcome);
    }
    let public_needed = ["api-shape", "canonical-path", "name-collision"]
        .iter()
        .any(|rule| active(rule));
    let surface = crate::public_surface::build_for(&sources, public_needed);
    let index = crate::state::Index::build(&sources);
    let elements = crate::element_shadow::Elements::build(&sources);
    let scope = Scope {
        sources: &sources,
        surface: &surface,
        index: &index,
        elements: &elements,
        policy: &policy,
        pending: &pending,
    };
    scope.run(root, options, &mut outcome);
    for list in [&mut outcome.errors, &mut outcome.notes] {
        list.sort();
        list.dedup();
    }
    Ok(outcome)
}

/// The loaded workspace facts one check run shares across rules.
struct Scope<'a> {
    sources: &'a [crate::check_source::Source],
    surface: &'a crate::public_surface::Surface,
    index: &'a crate::state::Index,
    elements: &'a crate::element_shadow::Elements,
    policy: &'a toml::Table,
    pending: &'a Pending,
}

impl Scope<'_> {
    fn run(&self, root: &Path, options: &Options, outcome: &mut Outcome) {
        let empty = toml::Table::new();
        for rule in RULES {
            if !options.selected(rule) {
                continue;
            }
            if let Some(package) = workspace_rule_package(rule) {
                if options
                    .package
                    .as_deref()
                    .is_none_or(|selected| selected == package)
                {
                    let mut found = Vec::new();
                    run_workspace_rule(rule, root, &mut found);
                    self.route(rule, package, found, options, outcome);
                }
                continue;
            }
            let settings = self
                .policy
                .get(&rule.replace('-', "_"))
                .and_then(toml::Value::as_table)
                .unwrap_or(&empty);
            for source in self.sources {
                if !selected_source(source, options) {
                    continue;
                }
                let mut found = Vec::new();
                self.run_rule(rule, source, settings, &mut found);
                self.route(rule, &source.package, found, options, outcome);
            }
        }
    }

    /// File the findings of `rule` in `package` as notes while the rule is pending there.
    fn route(
        &self,
        rule: &str,
        package: &str,
        found: Vec<String>,
        options: &Options,
        outcome: &mut Outcome,
    ) {
        if !options.strict && self.pending.covers(rule, package) {
            outcome.notes.extend(
                found
                    .into_iter()
                    .map(|finding| format!("pending: {finding}")),
            );
        } else {
            outcome.errors.extend(found);
        }
    }

    fn run_rule(
        &self,
        rule: &str,
        source: &crate::check_source::Source,
        settings: &toml::Table,
        errors: &mut Vec<String>,
    ) {
        match rule {
            "state" => crate::state::check(source, self.index, errors),
            "effect-writes" => crate::effect_writes::check(source, errors),
            "api-shape" => crate::api_shape::check(source, self.surface, errors),
            "canonical-path" => crate::canonical_path::check(source, self.surface, errors),
            "name-collision" => crate::name_collision::check(source, self.surface, errors),
            "kit-literal-color" => crate::kit_literal_color::check(source, settings, errors),
            "os-branch" => crate::os_branch::check(source, settings, errors),
            "interaction-view" => crate::interaction_view::check(source, errors),
            "ui-machine" => crate::ui_machine::check(source, settings, errors),
            "hand-element" => crate::hand_element::check(source, settings, errors),
            "unit-machine" => crate::unit_machine::check(source, errors),
            "element-shadow" => crate::element_shadow::check(source, self.elements, errors),
            _ => unreachable!("invariant: RULES contains only implemented checks"),
        }
    }
}

/// The package a workspace-wide rule reports for; `--crate` selects it by this name.
fn workspace_rule_package(rule: &str) -> Option<&'static str> {
    match rule {
        "docs-pairs" => Some("kivact-docs"),
        _ => None,
    }
}

fn run_workspace_rule(rule: &str, root: &Path, errors: &mut Vec<String>) {
    match rule {
        "docs-pairs" => crate::docs_pairs::check(root, errors),
        _ => unreachable!("invariant: workspace_rule_package names only workspace rules"),
    }
}

/// Print one selected run's notes and errors and choose the exit code.
pub(crate) fn selected(options: &Options, local: &Outcome) -> std::process::ExitCode {
    for note in &local.notes {
        println!("{note}");
    }
    for error in &local.errors {
        if options.report {
            println!("{error}");
        } else {
            eprintln!("error: {error}");
        }
    }
    if local.errors.is_empty() || options.report {
        println!("ok");
        std::process::ExitCode::SUCCESS
    } else {
        eprintln!("{} error(s)", local.errors.len());
        std::process::ExitCode::FAILURE
    }
}

fn validate_policy(policy: &toml::Table) -> Result<(), String> {
    for rule in RULES {
        if let Some(value) = policy.get(&rule.replace('-', "_")) {
            let table = value
                .as_table()
                .ok_or_else(|| format!("{rule}: expected a table"))?;
            for (key, value) in table {
                if !matches!(
                    key.as_str(),
                    "allowed_files" | "allowed_paths" | "allowed_types" | "allowed_impls"
                ) {
                    return Err(format!("{rule}: unknown setting {key}"));
                }
                if !value
                    .as_array()
                    .is_some_and(|items| items.iter().all(toml::Value::is_str))
                {
                    return Err(format!("{rule}.{key}: expected an array of strings"));
                }
            }
        }
    }
    Ok(())
}

fn selected_source(source: &crate::check_source::Source, options: &Options) -> bool {
    if options
        .package
        .as_ref()
        .is_some_and(|package| *package != source.package)
    {
        return false;
    }
    if let Some(module) = &options.module {
        let module = module
            .strip_prefix("crate::")
            .or_else(|| module.strip_prefix(&format!("{}::", source.package)))
            .or_else(|| module.strip_prefix(&format!("{}::", source.package.replace('-', "_"))))
            .unwrap_or(module);
        if source.module != module && !source.module.starts_with(&format!("{module}::")) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    //! Role: Exercise crate selection and module subtree selection.
    #[test]
    fn crate_and_module_selectors_restrict_scope() {
        let root = std::env::temp_dir().join(format!("kivact-xtask-policy-{}", std::process::id()));
        std::fs::create_dir_all(root.join("crates/demo/src")).unwrap();
        std::fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = ['crates/demo']",
        )
        .unwrap();
        std::fs::write(
            root.join("crates/demo/Cargo.toml"),
            "[package]\nname = 'demo'",
        )
        .unwrap();
        std::fs::write(root.join("crates/demo/src/lib.rs"), "mod state { pub struct Model { a: Cell<bool>, b: Option<Job> } mod child { pub struct Nested { a: Cell<bool>, b: bool } } } mod other { pub struct More { a: Cell<bool>, b: bool } }").unwrap();
        let policy = root.join("policy.toml");
        std::fs::write(&policy, "").unwrap();
        let mut options = crate::check_cli::Options {
            only: Some("state".into()),
            ..Default::default()
        };
        assert_eq!(
            super::check(&root, &policy, &options).unwrap().errors.len(),
            3
        );
        options.package = Some("demo".into());
        assert_eq!(
            super::check(&root, &policy, &options).unwrap().errors.len(),
            3
        );
        options.module = Some("state".into());
        assert_eq!(
            super::check(&root, &policy, &options).unwrap().errors.len(),
            2
        );
        options.module = Some("demo::state::child".into());
        assert_eq!(
            super::check(&root, &policy, &options).unwrap().errors.len(),
            1
        );
        options.package = Some("unknown".into());
        assert!(super::check(&root, &policy, &options).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
