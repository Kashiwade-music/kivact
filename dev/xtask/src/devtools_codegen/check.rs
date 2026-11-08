//! Role: Generate the protocol artifacts and compare them with the committed files.

use std::path::{Path, PathBuf};

use crate::devtools_codegen::python;
use crate::devtools_codegen::read::read_protocol;
use crate::devtools_codegen::schema;

/// Generate every artifact: its absolute path and its expected contents.
pub(crate) fn generate(root: &Path) -> Result<Vec<(PathBuf, String)>, String> {
    let protocol = read_protocol(root)?;
    Ok(vec![
        (
            root.join("crates/kivact-devtools-protocol/schema/kivact-devtools.schema.json"),
            schema::schema(&protocol),
        ),
        (
            root.join("crates/kivact-devtools-protocol/schema/devtools-config.schema.json"),
            schema::config_schema(&protocol),
        ),
        (
            root.join("python/kivact-devtools/src/kivact_devtools/_generated.py"),
            python::python(&protocol),
        ),
    ])
}

/// Compare the committed artifacts with freshly generated ones.
pub(crate) fn check(root: &Path) -> Vec<String> {
    let files = match generate(root) {
        Ok(files) => files,
        Err(error) => return vec![error],
    };
    files
        .into_iter()
        .filter_map(|(path, expected)| {
            let current = std::fs::read_to_string(&path)
                .ok()
                .map(|text| text.replace("\r\n", "\n"));
            if current.as_deref() == Some(expected.as_str()) {
                None
            } else {
                Some(outdated(&path))
            }
        })
        .collect()
}

fn outdated(path: &Path) -> String {
    format!(
        "{}: out of date; run `cargo xtask devtools-codegen`",
        path.display()
    )
}
