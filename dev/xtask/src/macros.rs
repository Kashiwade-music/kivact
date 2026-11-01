//! Role: Find the workspace's `macro_rules!` and every path that invokes them.
//!
//! xtask-core does not expand macros: it reports a local macro unless the
//! path it is invoked by is trusted. Every `.rs` file of the repository
//! (but `target/`, `vendor/` and hidden directories) is tokenized, so
//! comments and string literals are not mistaken for code; each
//! `name!` / `a::b::name!` whose last segment a `macro_rules!` of the
//! workspace defines is trusted by the path written there.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use proc_macro2::{Delimiter, TokenStream, TokenTree};

/// The paths that invoke a `macro_rules!` defined in the workspace below `root`.
pub(crate) fn workspace_macros(root: &Path) -> Result<BTreeSet<String>, String> {
    let mut files = Vec::new();
    rust_files(root, &mut files)?;
    let mut defined = BTreeSet::new();
    let mut invoked = BTreeSet::new();
    for file in files {
        let text = std::fs::read_to_string(&file)
            .map_err(|error| format!("{}: {error}", file.display()))?;
        // A file that does not tokenize does not compile either; rustc reports it.
        if let Ok(tokens) = TokenStream::from_str(&text) {
            scan(tokens, &mut defined, &mut invoked);
        }
    }
    Ok(invoked
        .into_iter()
        .filter(|path| {
            let name = path.rsplit("::").next().unwrap_or(path);
            defined.contains(name)
        })
        .chain(defined.iter().cloned())
        .collect())
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", dir.display()))?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let kind = entry
            .file_type()
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if kind.is_dir() {
            if !(name.starts_with('.') || name == "target" || name == "vendor") {
                rust_files(&path, out)?;
            }
        } else if name.ends_with(".rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// Collect `macro_rules! name` into `defined` and every `path!(..)` into
/// `invoked`, descending into every group (macro arguments included).
fn scan(tokens: TokenStream, defined: &mut BTreeSet<String>, invoked: &mut BTreeSet<String>) {
    let tokens: Vec<TokenTree> = tokens.into_iter().collect();
    // The path being read, and whether a `::` follows its last segment.
    let mut path: Vec<String> = Vec::new();
    let mut joined = false;
    let mut colon = false;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            TokenTree::Ident(ident) => {
                if !joined {
                    path.clear();
                }
                path.push(ident.to_string());
                joined = false;
                colon = false;
            }
            TokenTree::Punct(punct) if punct.as_char() == ':' => {
                joined = colon && !path.is_empty();
                colon = !colon;
            }
            TokenTree::Punct(punct) if punct.as_char() == '!' && !joined => {
                if path == ["macro_rules"] {
                    if let Some(TokenTree::Ident(name)) = tokens.get(i + 1) {
                        defined.insert(name.to_string());
                    }
                } else if !path.is_empty()
                    && matches!(
                        tokens.get(i + 1),
                        Some(TokenTree::Group(group)) if group.delimiter() != Delimiter::None
                    )
                {
                    invoked.insert(path.join("::"));
                }
                path.clear();
                colon = false;
            }
            TokenTree::Group(group) => {
                scan(group.stream(), defined, invoked);
                path.clear();
                (joined, colon) = (false, false);
            }
            _ => {
                path.clear();
                (joined, colon) = (false, false);
            }
        }
    }
}
