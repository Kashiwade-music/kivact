//! Role: Splits [module_rules] off the policy and writes the core policy xtask reads.

/// Split `[module_rules]` off the policy, point `config` at the rest and
/// trust the workspace's own macros.
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
    let mut analysis = xtask_core::Policy::from_toml(&core)
        .map_err(located)?
        .analysis;
    analysis
        .trusted_macros
        .extend(crate::macros::workspace_macros(&config.root)?);
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
