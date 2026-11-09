//! Role: The `.kivact/devtools.json` configuration and its defaults.

use serde::{Deserialize, Serialize};

/// Keys that open DevTools (`"Ctrl+Shift+I"` style chords). `None` means the default; `[]` disables.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct KeyConfig {
    /// Chords that open DevTools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<Vec<String>>,
    /// Chords that open DevTools in pick mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick: Option<Vec<String>>,
}

/// The DevTools configuration read from `.kivact/devtools.json`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DevtoolsConfig {
    /// Key chords.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keys: Option<KeyConfig>,
    /// Path of the DevTools app (relative to the workspace root, or absolute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// Path of the Python interpreter for scripts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    /// Directory of the scripts (relative to the workspace root, or absolute).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripts_dir: Option<String>,
    /// The command that opens a source location; `{file}`, `{line}` and `{column}` are replaced.
    /// Empty means the default.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editor: Vec<String>,
    /// The `$schema` key of the file (ignored).
    #[serde(default, rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
}

const OPEN_KEYS: [&str; 2] = ["F12", "Ctrl+Shift+I"];
const PICK_KEYS: [&str; 1] = ["Ctrl+Shift+C"];
const SCRIPTS_DIR: &str = ".kivact/devtools/scripts";
const EDITOR: [&str; 3] = ["code", "--goto", "{file}:{line}:{column}"];

fn owned(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

impl DevtoolsConfig {
    /// The chords that open DevTools (the default when `keys.open` is not set).
    pub fn open_keys(&self) -> Vec<String> {
        match self.keys.as_ref().and_then(|keys| keys.open.as_deref()) {
            Some(chords) => chords.to_vec(),
            None => owned(&OPEN_KEYS),
        }
    }
    /// The chords that open DevTools in pick mode (the default when `keys.pick` is not set).
    pub fn pick_keys(&self) -> Vec<String> {
        match self.keys.as_ref().and_then(|keys| keys.pick.as_deref()) {
            Some(chords) => chords.to_vec(),
            None => owned(&PICK_KEYS),
        }
    }
    /// The scripts directory text (the default when not set).
    pub fn scripts_dir(&self) -> String {
        self.scripts_dir
            .clone()
            .unwrap_or_else(|| SCRIPTS_DIR.to_owned())
    }
    /// The editor command (the default when empty).
    pub fn editor_command(&self) -> Vec<String> {
        if self.editor.is_empty() {
            owned(&EDITOR)
        } else {
            self.editor.clone()
        }
    }
    /// A copy with every default filled in (the answer of `get_config`).
    pub fn with_defaults(&self) -> DevtoolsConfig {
        DevtoolsConfig {
            keys: Some(KeyConfig {
                open: Some(self.open_keys()),
                pick: Some(self.pick_keys()),
            }),
            scripts_dir: Some(self.scripts_dir()),
            editor: self.editor_command(),
            schema: None,
            ..self.clone()
        }
    }
}
