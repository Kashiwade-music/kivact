//! Role: Run the workspace's local architectural checks under shared selectors.
use crate::check_cli::{Options, RULES};
use crate::check_policy::is_kit;
use crate::state_fields::{Mode, Strictness};
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn check(root: &Path, policy: &Path, options: &Options) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(policy).map_err(|e| format!("{}: {e}", policy.display()))?;
    let policy: toml::Table = text.parse().map_err(|e| format!("{e}"))?;
    validate_policy(&policy)?;
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
    if !RULES.iter().any(|rule| active(rule)) {
        return Ok(Vec::new());
    }
    let public_needed = [
        "api-shape",
        "canonical-path",
        "name-collision",
        "structural-options",
    ]
    .iter()
    .any(|rule| active(rule));
    let surface = crate::public_surface::build_for(&sources, public_needed);
    let (builders, enums) = model_types(&sources, &surface);
    let empty = toml::Table::new();
    let mut errors = Vec::new();
    for rule in RULES {
        if !options.selected(rule) {
            continue;
        }
        // The widened rule is opt-in until its migration finishes (design 6.1).
        if rule == "state-fields-strict" && options.only.is_none() {
            continue;
        }
        if rule == "devtools-codegen" {
            if options
                .package
                .as_deref()
                .is_none_or(|package| package == "kivact-devtools-protocol")
            {
                errors.extend(crate::devtools_codegen::check(root));
            }
            continue;
        }
        let key = if rule == "state-fields-strict" {
            "state_fields".to_owned()
        } else {
            rule.replace('-', "_")
        };
        let settings = policy
            .get(&key)
            .and_then(toml::Value::as_table)
            .unwrap_or(&empty);
        for source in &sources {
            if !selected_source(source, options) {
                continue;
            }
            run_rule(
                rule,
                source,
                settings,
                &builders,
                &enums,
                &surface,
                options.report,
                &mut errors,
            );
        }
    }
    errors.sort();
    errors.dedup();
    Ok(errors)
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
                    "role_suffixes"
                        | "named_helpers"
                        | "record_types"
                        | "value_modules"
                        | "allowed_files"
                        | "allowed_paths"
                        | "list"
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

pub(crate) fn model_types(
    sources: &[crate::check_source::Source],
    surface: &crate::public_surface::Surface,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut builders = BTreeSet::new();
    let enums: BTreeSet<String> = sources
        .iter()
        .flat_map(|source| {
            source.items.iter().filter_map(|item| {
                if let syn::Item::Enum(item) = item {
                    Some(crate::public_surface::definition(
                        source,
                        &item.ident.to_string(),
                    ))
                } else {
                    None
                }
            })
        })
        .collect();
    for source in sources {
        for item in &source.items {
            if let syn::Item::Impl(item) = item
                && item.trait_.as_ref().is_some_and(|(_, path, _)| {
                    path.segments
                        .last()
                        .is_some_and(|s| s.ident == "NodeBuilder")
                })
                && let Some(identity) = surface.resolve_type(source, &item.self_ty)
            {
                builders.insert(identity);
            }
        }
    }
    (builders, enums)
}

#[allow(
    clippy::too_many_arguments,
    reason = "one shared dispatcher passes the policy, model sets and report flag to every rule"
)]
fn run_rule(
    rule: &str,
    source: &crate::check_source::Source,
    settings: &toml::Table,
    builders: &BTreeSet<String>,
    enums: &BTreeSet<String>,
    surface: &crate::public_surface::Surface,
    report: bool,
    errors: &mut Vec<String>,
) {
    match rule {
        "state-fields" => crate::state_fields::check(
            source,
            settings,
            builders,
            enums,
            surface,
            Mode {
                strictness: Strictness::Current,
                report,
            },
            errors,
        ),
        "state-fields-strict" => crate::state_fields::check(
            source,
            settings,
            builders,
            enums,
            surface,
            Mode {
                strictness: Strictness::Strict,
                report,
            },
            errors,
        ),
        "effect-writes" => crate::effect_writes::check(source, errors),
        "api-shape" => crate::api_shape::check(source, surface, errors),
        "canonical-path" => crate::canonical_path::check(source, surface, errors),
        "name-collision" => crate::name_collision::check(source, surface, errors),
        "kit-literal-color" => crate::kit_literal_color::check(source, settings, errors),
        "os-branch" => crate::os_branch::check(source, settings, errors),
        "structural-options" => crate::structural_options::check(source, settings, surface, errors),
        "devtools-codegen" => {}
        _ => unreachable!("invariant: RULES contains only implemented checks"),
    }
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
        std::fs::write(root.join("crates/demo/src/lib.rs"), "mod state { pub struct Model { a: bool, b: Option<Job> } mod child { pub struct Nested { a: bool, b: bool } } } mod other { pub struct More { a: bool, b: bool } }").unwrap();
        let policy = root.join("policy.toml");
        std::fs::write(&policy, "").unwrap();
        let mut options = crate::check_cli::Options {
            only: Some("state-fields".into()),
            ..Default::default()
        };
        assert_eq!(super::check(&root, &policy, &options).unwrap().len(), 3);
        options.package = Some("demo".into());
        assert_eq!(super::check(&root, &policy, &options).unwrap().len(), 3);
        options.module = Some("state".into());
        assert_eq!(super::check(&root, &policy, &options).unwrap().len(), 2);
        options.module = Some("demo::state::child".into());
        assert_eq!(super::check(&root, &policy, &options).unwrap().len(), 1);
        options.package = Some("unknown".into());
        assert!(super::check(&root, &policy, &options).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
