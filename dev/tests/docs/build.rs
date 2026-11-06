//! Role: Generates one doctest module per Markdown guide under `docs/`.
//!
//! Every `.md` file of the user and kit guides becomes an empty module whose
//! documentation is that file, mirroring the directory tree, so
//! `cargo test -p kivact-docs --doc -- for_kivact_ui_user::en::basic` runs
//! the blocks of one subtree. A new guide needs no registration here.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// The guide trees that hold Rust examples; the developer strategy has none.
const TREES: [&str; 2] = ["for-kivact-user", "for-kivact-ui-user"];

/// Guides (by file name) whose examples need a feature of this crate; their
/// modules, and so their doctests, exist only with that feature.
const FEATURE_GUIDES: [(&str, &str); 1] = [("gpu-texture-sharing.md", "gpu-interop")];

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by Cargo"));
    let root = manifest.join("../../..");
    let docs = root.join("docs");
    let mut code = String::new();
    for readme in ["README.md", "README_JP.md"] {
        let path = root.join(readme);
        println!("cargo::rerun-if-changed={}", path.display());
        guide(&mut code, &module_name(readme), &path);
    }
    for tree in TREES {
        let dir = docs.join(tree);
        println!("cargo::rerun-if-changed={}", dir.display());
        directory(&mut code, &module_name(tree), &dir);
    }
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by Cargo"));
    fs::write(out.join("guides.rs"), code).expect("OUT_DIR is writable");
}

fn directory(code: &mut String, name: &str, dir: &Path) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .expect("guide directory is readable")
        .map(|entry| entry.expect("guide directory is readable").path())
        .collect();
    entries.sort();
    let label = dir
        .file_name()
        .and_then(|n| n.to_str())
        .expect("UTF-8 directory name");
    let _ = writeln!(
        code,
        "#[doc = \"Guides in `{label}/`.\"]\npub mod {name} {{"
    );
    for path in entries {
        let file = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("UTF-8 file name");
        if path.is_dir() {
            directory(code, &module_name(file), &path);
        } else if let Some(stem) = file.strip_suffix(".md") {
            if let Some((_, feature)) = FEATURE_GUIDES.iter().find(|(guide, _)| *guide == file) {
                let _ = writeln!(code, "#[cfg(feature = \"{feature}\")]");
            }
            guide(code, &module_name(stem), &path);
        }
    }
    code.push_str("}\n");
}

fn guide(code: &mut String, name: &str, path: &Path) {
    let _ = writeln!(
        code,
        "#[doc = include_str!(\"{}\")]\npub mod {name} {{}}",
        slash(path)
    );
}

/// A Rust identifier for a file or directory name (`01-first-app` → `_01_first_app`).
fn module_name(file: &str) -> String {
    let stem = file.strip_suffix(".md").unwrap_or(file);
    let mut name: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        name.insert(0, '_');
    }
    name
}

fn slash(path: &Path) -> String {
    let path = fs::canonicalize(path).expect("guide path exists");
    let text = path.to_str().expect("UTF-8 path").replace('\\', "/");
    // `canonicalize` on Windows returns a verbatim `//?/` path; include_str! accepts the plain one.
    text.strip_prefix("//?/").unwrap_or(&text).to_string()
}
