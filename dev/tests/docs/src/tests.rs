//! Role: Checks over the Markdown sources themselves.

use std::fs;
use std::path::{Path, PathBuf};

fn markdown_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            markdown_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

/// The info string and body of every fenced code block of `text`.
fn code_blocks(text: &str) -> Vec<(&str, String)> {
    let mut blocks = Vec::new();
    let mut open: Option<(&str, String)> = None;
    for line in text.lines() {
        match (&mut open, line.strip_prefix("```")) {
            (None, Some(info)) => open = Some((info.trim(), String::new())),
            (Some(_), Some(_)) => blocks.extend(open.take()),
            (Some((_, body)), None) => {
                body.push_str(line);
                body.push('\n');
            }
            (None, None) => {}
        }
    }
    blocks
}

/// A test module in a Rust block only compiles under `test_harness`
/// (rustdoc does not set `cfg(test)` otherwise), so a block with one must be
/// `test_harness` or its tests would silently never run.
#[test]
fn test_modules_in_guides_run_under_the_test_harness() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut files = vec![root.join("README.md")];
    markdown_files(&root.join("docs"), &mut files);
    let mut wrong = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap();
        for (info, body) in code_blocks(&text) {
            let attrs: Vec<&str> = info.split(',').map(str::trim).collect();
            if attrs.first() == Some(&"rust")
                && body.contains("#[cfg(test)]")
                && !attrs.contains(&"test_harness")
            {
                wrong.push(format!("{} ({info})", file.display()));
            }
        }
    }
    assert!(
        wrong.is_empty(),
        "blocks with `#[cfg(test)]` that are not `test_harness`: {wrong:#?}"
    );
}
