//! Generates the doctest modules and the type-checked `no_run` blocks of the English guides.
//!
//! Every `.md` file of the English user and kit guides, and `README.md`,
//! becomes an empty module whose documentation is a copy of that file, so
//! `cargo test -p kivact-docs --doc -- for_kivact_ui_user_en::basic` runs the
//! blocks of one subtree. In the copy each `rust,no_run` block is marked
//! `ignore`; its code goes instead into one module of `checked.rs`, which
//! `cargo check -p kivact-docs --all-targets` type-checks without linking.
//! The Japanese guides are not compiled: `cargo xtask check` (`docs-pairs`)
//! keeps their Rust blocks equal to the English ones. A new guide needs no
//! registration here.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// The guide trees that hold Rust examples (English only).
const TREES: [&str; 2] = ["for-kivact-user/en", "for-kivact-ui-user/en"];

/// Guides (by file name) whose examples need a feature of this crate.
const FEATURE_GUIDES: [(&str, &str); 1] = [("gpu-texture-sharing.md", "gpu-interop")];

/// What the build writes: the doctest modules and the checked blocks.
#[derive(Default)]
struct Output {
    guides: String,
    checked: String,
}

/// Where the reader is among the fences of one guide.
enum Fence {
    Outside,
    Other,
    NoRun(String),
}

fn main() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("set by Cargo"));
    let root = manifest.join("../../..");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("set by Cargo"));
    let copies = out.join("guides");
    fs::create_dir_all(&copies).expect("OUT_DIR is writable");
    let mut output = Output::default();
    let readme = root.join("README.md");
    println!("cargo::rerun-if-changed={}", readme.display());
    guide(&mut output, &copies, "readme", "readme", &readme, None);
    for tree in TREES {
        let dir = root.join("docs").join(tree);
        println!("cargo::rerun-if-changed={}", dir.display());
        let name = module_name(tree);
        directory(&mut output, &copies, &name, &name, &dir);
    }
    fs::write(out.join("guides.rs"), output.guides).expect("OUT_DIR is writable");
    fs::write(out.join("checked.rs"), output.checked).expect("OUT_DIR is writable");
}

/// `name` is the nested module of `dir`; `prefix` its flat path for the checked blocks.
fn directory(output: &mut Output, copies: &Path, name: &str, prefix: &str, dir: &Path) {
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
        output.guides,
        "#[doc = \"Guides in `{label}/`.\"]\npub mod {name} {{"
    );
    for path in entries {
        let file = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("UTF-8 file name");
        if path.is_dir() {
            let child = module_name(file);
            let flat = format!("{prefix}_{}", child.trim_start_matches('_'));
            directory(output, copies, &child, &flat, &path);
        } else if let Some(stem) = file.strip_suffix(".md") {
            let feature = FEATURE_GUIDES
                .iter()
                .find(|(guide, _)| *guide == file)
                .map(|(_, feature)| *feature);
            let child = module_name(stem);
            let flat = format!("{prefix}_{}", child.trim_start_matches('_'));
            guide(output, copies, &child, &flat, &path, feature);
        }
    }
    output.guides.push_str("}\n");
}

/// Write the doctest module of one guide and the checked modules of its `no_run` blocks.
fn guide(
    output: &mut Output,
    copies: &Path,
    name: &str,
    prefix: &str,
    path: &Path,
    feature: Option<&str>,
) {
    let text = fs::read_to_string(path).expect("guide is readable");
    let (copy, blocks) = split_blocks(&text);
    let copy_path = copies.join(format!("{prefix}.md"));
    fs::write(&copy_path, copy).expect("OUT_DIR is writable");
    let gate = feature
        .map(|feature| format!("#[cfg(feature = \"{feature}\")]\n"))
        .unwrap_or_default();
    let _ = writeln!(
        output.guides,
        "{gate}#[doc = include_str!(\"{}\")]\npub mod {name} {{}}",
        slash(&copy_path)
    );
    for (index, block) in blocks.iter().enumerate() {
        let _ = writeln!(
            output.checked,
            "{gate}mod {prefix}_block_{} {{\n{}\n}}",
            index + 1,
            checked_body(block)
        );
    }
}

/// The guide with each `rust,no_run` fence marked `ignore`, and the bodies of those blocks.
fn split_blocks(text: &str) -> (String, Vec<String>) {
    let mut copy = String::new();
    let mut blocks = Vec::new();
    let mut fence = Fence::Outside;
    for line in text.lines() {
        let info = line.strip_prefix("```");
        fence = match (fence, info) {
            (Fence::Outside, Some(info)) if is_no_run(info) => {
                let _ = writeln!(copy, "```{}", info.replacen("no_run", "ignore", 1));
                Fence::NoRun(String::new())
            }
            (Fence::Outside, Some(_)) => {
                let _ = writeln!(copy, "{line}");
                Fence::Other
            }
            (Fence::Other, Some(_)) => {
                let _ = writeln!(copy, "{line}");
                Fence::Outside
            }
            (Fence::NoRun(body), Some(_)) => {
                blocks.push(body);
                let _ = writeln!(copy, "{line}");
                Fence::Outside
            }
            (Fence::NoRun(mut body), None) => {
                let _ = writeln!(body, "{line}");
                let _ = writeln!(copy, "{line}");
                Fence::NoRun(body)
            }
            (state, None) => {
                let _ = writeln!(copy, "{line}");
                state
            }
        };
    }
    (copy, blocks)
}

fn is_no_run(info: &str) -> bool {
    let attrs: Vec<&str> = info.split(',').map(str::trim).collect();
    attrs.first() == Some(&"rust") && attrs.contains(&"no_run")
}

/// The block as module items: hidden-line markers removed, and the code wrapped
/// in `fn main` the way rustdoc wraps a block that has none.
fn checked_body(block: &str) -> String {
    let lines: Vec<String> = block.lines().map(unhide).collect();
    let code = lines.join("\n");
    let has_main = lines.iter().any(|line| {
        let line = line.trim_start();
        line.starts_with("fn main(")
            || line.starts_with("async fn main(")
            || line.starts_with("pub fn main(")
    });
    if has_main {
        return code;
    }
    let returns_result = lines
        .iter()
        .rev()
        .map(|line| line.trim())
        .find(|line| !line.is_empty())
        .is_some_and(|last| last.starts_with("Ok::<"));
    if returns_result {
        format!(
            "fn main() {{\n    fn _inner() -> core::result::Result<(), impl core::fmt::Debug> {{\n{code}\n    }}\n    let _ = _inner();\n}}"
        )
    } else {
        format!("fn main() {{\n{code}\n}}")
    }
}

/// A line without rustdoc's hidden-line marker (`# ` hides a line; `##` writes `#`).
fn unhide(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed == "#" {
        String::new()
    } else if let Some(rest) = trimmed.strip_prefix("# ") {
        rest.to_owned()
    } else if let Some(rest) = trimmed
        .strip_prefix('#')
        .filter(|rest| rest.starts_with('#'))
    {
        rest.to_owned()
    } else {
        line.to_owned()
    }
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
