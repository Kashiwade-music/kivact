//! Role: Read shared local-check policy values without depending on check execution.

pub(crate) fn strings(table: &toml::Table, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(toml::Value::as_str)
        .map(str::to_owned)
        .collect()
}
pub(crate) fn is_kit(package: &str) -> bool {
    package.starts_with("kivact-ui-")
}
