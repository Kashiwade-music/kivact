//! Role: Read the rework pending table: rules that only print in listed packages.
//!
//! `[rework.pending]` maps a local rule to the packages whose findings are
//! notes until the units that fix them land; `--strict` makes them errors.
//! The table exists only during the rework (REWORK_DESIGN.md); annex G's last
//! unit deletes it with this module.

use std::collections::{BTreeMap, BTreeSet};

/// The packages each pending rule only reports on.
#[derive(Default)]
pub(crate) struct Pending {
    rules: BTreeMap<String, BTreeSet<String>>,
}

impl Pending {
    pub(crate) fn parse(policy: &toml::Table) -> Result<Self, String> {
        let mut pending = Self::default();
        let Some(rework) = policy.get("rework") else {
            return Ok(pending);
        };
        let rework = rework.as_table().ok_or("rework: expected a table")?;
        for (key, value) in rework {
            if key != "pending" {
                return Err(format!("rework: unknown setting {key}"));
            }
            let table = value.as_table().ok_or("rework.pending: expected a table")?;
            for (rule, packages) in table {
                if !crate::check_cli::RULES.contains(&rule.as_str()) {
                    return Err(format!("rework.pending: unknown rule {rule}"));
                }
                let expected =
                    || format!("rework.pending.{rule}: expected an array of package names");
                let names = packages
                    .as_array()
                    .ok_or_else(expected)?
                    .iter()
                    .map(|name| name.as_str().map(str::to_owned).ok_or_else(expected))
                    .collect::<Result<BTreeSet<_>, _>>()?;
                pending.rules.insert(rule.clone(), names);
            }
        }
        Ok(pending)
    }

    pub(crate) fn covers(&self, rule: &str, package: &str) -> bool {
        self.rules
            .get(rule)
            .is_some_and(|packages| packages.contains(package))
    }
}
