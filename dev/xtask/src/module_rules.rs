//! Role: Check the direction of module references inside a crate (`[module_rules]`).
//!
//! The policy names, per crate, which of its top-level modules each
//! top-level module (with its descendants) may refer to:
//!
//! ```toml
//! [module_rules.kivact-core]
//! except = ["view -> flow"]
//!
//! [module_rules.kivact-core.uses]
//! style = ["task", "motion"]
//! ```
//!
//! A module of `uses` may refer to its own subtree, to the crate root, to
//! the modules listed for it and to the modules `uses` does not name;
//! a reference to another module `uses` names is an error. A module
//! `uses` does not name is neither restricted nor protected. `except` lists
//! the references against the direction the design allows: `from` is that
//! module alone (not its descendants), `to` is that module or one below it,
//! both without `crate::`. An exception no reference needs is an error too.
//! Only library targets are checked, in every configuration analyzed.

use std::collections::{BTreeMap, BTreeSet};

use xtask_core::TargetGraph;

/// The `[module_rules]` of the policy, by package.
#[derive(Debug, Default)]
pub(crate) struct Rules(BTreeMap<String, CrateRules>);

#[derive(Debug, Default)]
struct CrateRules {
    uses: BTreeMap<String, BTreeSet<String>>,
    except: Vec<(String, String)>,
}

impl Rules {
    /// Read the `module_rules` table (`None`: no rules).
    pub(crate) fn parse(value: Option<toml::Value>) -> Result<Self, String> {
        let Some(value) = value else {
            return Ok(Self::default());
        };
        let table = value.as_table().ok_or("must be a table of packages")?;
        let mut rules = BTreeMap::new();
        for (package, value) in table {
            rules.insert(
                package.clone(),
                CrateRules::parse(value).map_err(|error| format!("{package}: {error}"))?,
            );
        }
        Ok(Self(rules))
    }
}

impl CrateRules {
    fn parse(value: &toml::Value) -> Result<Self, String> {
        let table = value.as_table().ok_or("must be a table")?;
        let mut rules = Self::default();
        for (key, value) in table {
            match key.as_str() {
                "uses" => {
                    let uses = value.as_table().ok_or("`uses` must be a table")?;
                    for (module, used) in uses {
                        rules
                            .uses
                            .insert(module.clone(), names(used, &format!("uses.{module}"))?);
                    }
                }
                "except" => {
                    for entry in names(value, "except")? {
                        let (from, to) = entry
                            .split_once(" -> ")
                            .ok_or_else(|| format!("except `{entry}` is not `from -> to`"))?;
                        rules.except.push((from.to_owned(), to.to_owned()));
                    }
                }
                _ => return Err(format!("unknown key `{key}`")),
            }
        }
        for (module, used) in &rules.uses {
            if let Some(unknown) = used.iter().find(|used| !rules.uses.contains_key(*used)) {
                return Err(format!(
                    "uses.{module}: `{unknown}` is not a module of `uses`"
                ));
            }
        }
        Ok(rules)
    }
}

fn names(value: &toml::Value, key: &str) -> Result<BTreeSet<String>, String> {
    value
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .ok_or_else(|| format!("`{key}` must be an array of strings"))
}

/// The top-level module of `crate::a::b` (`None` for the crate root).
fn top(id: &str) -> Option<&str> {
    id.strip_prefix("crate::")?.split("::").next()
}

fn within(id: &str, module: &str) -> bool {
    id.strip_prefix("crate::").is_some_and(|path| {
        path == module
            || path
                .strip_prefix(module)
                .is_some_and(|rest| rest.starts_with("::"))
    })
}

/// Every reference of the library targets in `targets` against `rules`.
pub(crate) fn check(rules: &Rules, targets: &[TargetGraph]) -> Vec<String> {
    let mut errors = BTreeSet::new();
    for (package, rules) in &rules.0 {
        let mut checked = false;
        let mut used = vec![false; rules.except.len()];
        for target in targets.iter().filter(|target| {
            target.package == *package && target.kind.split('+').any(|kind| kind == "lib")
        }) {
            checked = true;
            for edge in &target.graph.edges {
                let (Some(from), Some(to)) = (top(&edge.from), top(&edge.to)) else {
                    continue;
                };
                let Some(allowed) = rules.uses.get(from) else {
                    continue;
                };
                if from == to || allowed.contains(to) || !rules.uses.contains_key(to) {
                    continue;
                }
                if let Some(i) = rules.except.iter().position(|(except_from, except_to)| {
                    edge.from.strip_prefix("crate::") == Some(except_from.as_str())
                        && within(&edge.to, except_to)
                }) {
                    used[i] = true;
                    continue;
                }
                errors.insert(format!(
                    "{package}: {} refers to {}, but module_rules.{package}.uses.{from} does not allow `{to}` (dev/xtask/policy/workspace.toml)",
                    edge.from, edge.to
                ));
            }
        }
        if !checked {
            errors.insert(format!(
                "module_rules.{package}: no library target of that package was analyzed"
            ));
            continue;
        }
        for ((from, to), used) in rules.except.iter().zip(used) {
            if !used {
                errors.insert(format!(
                    "module_rules.{package}.except: no reference needs `{from} -> {to}`"
                ));
            }
        }
    }
    errors.into_iter().collect()
}
