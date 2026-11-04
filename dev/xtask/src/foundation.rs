//! Role: Run the workspace's local architectural checks under shared selectors.
use crate::check_cli::{Options, RULES};
use crate::check_policy::{is_kit, strings};
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn check(root: &Path, policy: &Path, options: &Options) -> Result<Vec<String>, String> {
    let text = std::fs::read_to_string(policy).map_err(|e| format!("{}: {e}", policy.display()))?;
    let policy: toml::Table = text.parse().map_err(|e| format!("{e}"))?;
    for rule in RULES {
        if let Some(value) = policy.get(&rule.replace('-', "_")) {
            let table = value
                .as_table()
                .ok_or_else(|| format!("{rule}: expected a table"))?;
            for (key, value) in table {
                if !matches!(
                    key.as_str(),
                    "skip"
                        | "role_suffixes"
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
    let sources = crate::check_source::load(root)?;
    if let Some(package) = &options.package {
        if !sources.iter().any(|s| s.package == *package) {
            return Err(format!("unknown crate: {package}"));
        }
    }
    let active = |rule: &str| {
        let skip = policy
            .get(&rule.replace('-', "_"))
            .and_then(toml::Value::as_table)
            .map(|t| strings(t, "skip"))
            .unwrap_or_default();
        options.selected(rule)
            && sources.iter().any(|source| {
                if matches!(rule, "kit-literal-color" | "name-collision")
                    && !is_kit(&source.package)
                {
                    return false;
                }
                options
                    .package
                    .as_ref()
                    .map_or(!skip.contains(&source.package), |package| {
                        *package == source.package
                    })
            })
    };
    if !RULES.iter().any(|rule| active(rule)) {
        return Ok(Vec::new());
    }
    let public_needed = ["api-shape", "canonical-path", "name-collision"]
        .iter()
        .any(|rule| active(rule));
    let surface = crate::public_surface::build_for(&sources, public_needed);
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
    for source in &sources {
        for item in &source.items {
            if let syn::Item::Impl(item) = item {
                if item.trait_.as_ref().is_some_and(|(_, path, _)| {
                    path.segments
                        .last()
                        .is_some_and(|s| s.ident == "NodeBuilder")
                }) {
                    if let Some(identity) = surface.resolve_type(source, &item.self_ty) {
                        builders.insert(identity);
                    }
                }
            }
        }
    }
    let empty = toml::Table::new();
    let mut errors = Vec::new();
    for rule in RULES {
        if !options.selected(rule) {
            continue;
        }
        let key = rule.replace('-', "_");
        let settings = policy
            .get(&key)
            .and_then(toml::Value::as_table)
            .unwrap_or(&empty);
        let skip = strings(settings, "skip");
        for source in &sources {
            if let Some(package) = &options.package {
                if *package != source.package {
                    continue;
                }
            } else if skip.contains(&source.package) {
                continue;
            }
            if let Some(module) = &options.module {
                let module = module
                    .strip_prefix("crate::")
                    .or_else(|| module.strip_prefix(&format!("{}::", source.package)))
                    .or_else(|| {
                        module.strip_prefix(&format!("{}::", source.package.replace('-', "_")))
                    })
                    .unwrap_or(module);
                if source.module != module && !source.module.starts_with(&format!("{module}::")) {
                    continue;
                }
            }
            match rule {
                "state-fields" => crate::state_fields::check(
                    source,
                    settings,
                    &builders,
                    &enums,
                    &surface,
                    &mut errors,
                ),
                "effect-writes" => crate::effect_writes::check(source, &mut errors),
                "api-shape" => crate::api_shape::check(source, &surface, &mut errors),
                "canonical-path" => crate::canonical_path::check(source, &surface, &mut errors),
                "name-collision" => crate::name_collision::check(source, &surface, &mut errors),
                "kit-literal-color" => {
                    crate::kit_literal_color::check(source, settings, &mut errors)
                }
                "os-branch" => crate::os_branch::check(source, settings, &mut errors),
                "structural-options" => {
                    crate::structural_options::check(source, settings, &mut errors)
                }
                _ => unreachable!("invariant: RULES contains only implemented checks"),
            }
        }
    }
    errors.sort();
    errors.dedup();
    Ok(errors)
}

#[cfg(test)]
mod tests {
    //! Role: Exercise policy skip overrides and module subtree selection.
    #[test]
    fn explicit_crate_overrides_skip_and_modules_restrict_scope() {
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
        std::fs::write(&policy, "[state_fields]\nskip = ['demo']").unwrap();
        let mut options = crate::check_cli::Options {
            only: Some("state-fields".into()),
            ..Default::default()
        };
        assert!(super::check(&root, &policy, &options).unwrap().is_empty());
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
