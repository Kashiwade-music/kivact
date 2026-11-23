//! Role: Splits the rule tables off the policy and writes the core policy xtask reads.

use std::collections::BTreeSet;
use std::path::Path;

use xtask_core::AnalysisSettings;

/// Split `[module_rules]` and the local rule tables off the policy, point
/// `config` at the rest and trust the workspace's own macros.
pub(crate) fn prepare(
    config: &mut xtask_core::Config,
) -> Result<crate::module_rules::Rules, String> {
    let path = config.policy.clone();
    let located = |error: String| format!("{}: {error}", path.display());
    let text = std::fs::read_to_string(&path).map_err(|error| located(error.to_string()))?;
    let mut table: toml::Table =
        toml::from_str(&text).map_err(|error| located(error.to_string()))?;
    let rules = crate::module_rules::Rules::parse(table.remove("module_rules"))
        .map_err(|error| located(format!("module_rules: {error}")))?;
    for rule in crate::check_cli::RULES {
        table.remove(&rule.replace('-', "_"));
    }
    let core = toml::to_string(&table).map_err(|error| error.to_string())?;
    let policy = xtask_core::Policy::from_toml(&core).map_err(located)?;
    let analysis = with_workspace_macros(policy.analysis(), &config.root)?;
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

/// The policy's analysis settings, with the paths that invoke the workspace's
/// own `macro_rules!` trusted.
fn with_workspace_macros(
    settings: &AnalysisSettings,
    root: &Path,
) -> Result<AnalysisSettings, String> {
    let list = |names: &BTreeSet<String>| names.iter().cloned().collect::<Vec<_>>();
    let mut macros = settings.trusted_macros().clone();
    macros.extend(crate::macros::workspace_macros(root)?);
    AnalysisSettings::new(
        settings.target().map(str::to_owned),
        settings.features().clone(),
        list(settings.cfg()),
        list(settings.trusted_attributes()),
        list(settings.trusted_derives()),
        list(&macros),
        list(settings.trusted_globs()),
    )?
    .with_generated_type_attributes(list(settings.generated_type_attributes()))
}
