//! Role: Check that each Japanese guide has the Rust code blocks of its English twin.
//!
//! The English and Japanese trees must hold the same files. The Rust blocks of
//! a pair are compared as token streams: comments, whitespace and the text of
//! string literals may differ (translated or wrapped), while every token
//! boundary, identifier, punctuation and other literal must match, and every
//! string literal must be valid Rust (design 6.4, chapter 12 TV-06).

use proc_macro2::{Delimiter, Spacing, TokenStream, TokenTree};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The guide trees with an `en` and a `jp` side.
const TREES: [&str; 2] = ["docs/for-kivact-user", "docs/for-kivact-ui-user"];

pub(crate) fn check(root: &Path, errors: &mut Vec<String>) {
    compare(&root.join("README.md"), &root.join("README_JP.md"), errors);
    for tree in TREES {
        let english = root.join(tree).join("en");
        let japanese = root.join(tree).join("jp");
        let mut names = BTreeSet::new();
        for side in [&english, &japanese] {
            let mut files = Vec::new();
            markdown_files(side, &mut files);
            names.extend(
                files
                    .iter()
                    .filter_map(|file| file.strip_prefix(side).ok().map(Path::to_path_buf)),
            );
        }
        for name in names {
            compare(&english.join(&name), &japanese.join(&name), errors);
        }
    }
}

fn markdown_files(directory: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            markdown_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "md") {
            out.push(path);
        }
    }
}

fn compare(en: &Path, jp: &Path, errors: &mut Vec<String>) {
    let (Ok(english), Ok(japanese)) = (std::fs::read_to_string(en), std::fs::read_to_string(jp))
    else {
        let (present, missing) = if en.is_file() { (en, jp) } else { (jp, en) };
        errors.push(format!(
            "docs-pairs: {}: the twin {} is missing",
            present.display(),
            missing.display()
        ));
        return;
    };
    let (english, japanese) = (rust_blocks(&english), rust_blocks(&japanese));
    if english.len() != japanese.len() {
        errors.push(format!(
            "docs-pairs: {}: {} Rust blocks, the English twin has {}",
            jp.display(),
            japanese.len(),
            english.len()
        ));
        return;
    }
    for (index, ((en_info, en_code), (jp_info, jp_code))) in
        english.iter().zip(&japanese).enumerate()
    {
        let block = index + 1;
        match (tokens(en_code), tokens(jp_code)) {
            (Err(error), _) => errors.push(format!("docs-pairs: {}: Rust block {block}: {error}", en.display())),
            (_, Err(error)) => errors.push(format!("docs-pairs: {}: Rust block {block}: {error}", jp.display())),
            (Ok(a), Ok(b)) if en_info != jp_info || a != b => errors.push(format!(
                "docs-pairs: {}: Rust block {block} differs from {} beyond comments, whitespace and string text",
                jp.display(),
                en.display()
            )),
            _ => {}
        }
    }
}

/// The info string and the code of every fenced block whose info string starts with `rust`.
fn rust_blocks(text: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut open: Option<(String, String)> = None;
    for line in text.lines() {
        match (open.take(), line.strip_prefix("```")) {
            (None, Some(info)) => open = Some((info.trim().to_owned(), String::new())),
            (Some((info, body)), Some(_)) => {
                if info.starts_with("rust") {
                    blocks.push((info, body));
                }
            }
            (Some((info, mut body)), None) => {
                body.push_str(line);
                body.push('\n');
                open = Some((info, body));
            }
            (None, None) => {}
        }
    }
    blocks
}

/// The block as comparable tokens: the text of string literals is blanked, everything else kept.
fn tokens(code: &str) -> Result<Vec<String>, String> {
    let stream: TokenStream = code
        .parse()
        .map_err(|error| format!("does not lex: {error}"))?;
    let mut out = Vec::new();
    flatten(stream, &mut out)?;
    Ok(out)
}

fn flatten(stream: TokenStream, out: &mut Vec<String>) -> Result<(), String> {
    for tree in stream {
        match tree {
            TokenTree::Group(group) => {
                let (open, close) = match group.delimiter() {
                    Delimiter::Parenthesis => ("(", ")"),
                    Delimiter::Brace => ("{", "}"),
                    Delimiter::Bracket => ("[", "]"),
                    Delimiter::None => ("", ""),
                };
                out.push(open.to_owned());
                flatten(group.stream(), out)?;
                out.push(close.to_owned());
            }
            TokenTree::Ident(ident) => out.push(ident.to_string()),
            TokenTree::Punct(punct) => {
                let joint = if punct.spacing() == Spacing::Joint {
                    "+"
                } else {
                    ""
                };
                out.push(format!("{}{joint}", punct.as_char()));
            }
            TokenTree::Literal(literal) => out.push(literal_token(&literal.to_string())?),
        }
    }
    Ok(())
}

/// A string literal with its text blanked after checking its escapes; any other literal as written.
fn literal_token(text: &str) -> Result<String, String> {
    let Some(start) = text.find('"') else {
        return Ok(text.to_owned());
    };
    let prefix = &text[..start];
    if !matches!(
        prefix.trim_end_matches('#'),
        "" | "b" | "c" | "r" | "br" | "cr"
    ) {
        return Ok(text.to_owned());
    }
    let end = text.rfind('"').unwrap_or(start);
    let raw = prefix.contains('r');
    if !raw && end > start && !valid_escapes(&text[start + 1..end]) {
        return Err(format!("invalid escape in the string literal {text}"));
    }
    Ok(format!("{prefix}\"\"{}", &text[end + 1..]))
}

/// Whether every backslash of a (non-raw) string literal's text starts a valid Rust escape.
fn valid_escapes(body: &str) -> bool {
    let mut chars = body.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            continue;
        }
        match chars.next() {
            Some('n' | 'r' | 't' | '\\' | '0' | '\'' | '"' | '\n') => {}
            Some('\r') if chars.peek() == Some(&'\n') => {
                chars.next();
            }
            Some('x') => {
                let digits: String = chars.by_ref().take(2).collect();
                if digits.len() != 2 || !digits.chars().all(|d| d.is_ascii_hexdigit()) {
                    return false;
                }
            }
            Some('u') => {
                if chars.next() != Some('{') {
                    return false;
                }
                let mut count = 0;
                loop {
                    match chars.next() {
                        Some('}') if count > 0 => break,
                        Some(d) if d.is_ascii_hexdigit() && count < 6 => count += 1,
                        _ => return false,
                    }
                }
            }
            _ => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    //! Role: Check what may and may not differ between a guide and its twin.
    use super::{rust_blocks, tokens};

    fn same(a: &str, b: &str) -> bool {
        tokens(a).expect("the fixture lexes") == tokens(b).expect("the fixture lexes")
    }

    #[test]
    fn comments_whitespace_and_string_text_may_differ() {
        assert!(same(
            "let name = \"Name\"; // the name\nfield(name).hint(\"Shown\")",
            "let name = \"名前\"; // 名前\nfield(name)\n    .hint(\"表示\")"
        ));
        assert!(same("/* a /* nested */ b */ x", "x"));
    }

    #[test]
    fn token_boundaries_and_names_must_match() {
        assert!(!same("let x = ab;", "letx = ab;"));
        assert!(!same("a <= b", "a < = b"));
        assert!(!same("fn don_t() {}", "fn dont() {}"));
    }

    #[test]
    fn string_literals_must_be_valid() {
        assert!(tokens("let s = \"\\q\";").is_err());
        assert!(tokens("let s = \"\\n\\u{1F600}\\x41\"; let r = r#\"\\q\"#;").is_ok());
    }

    #[test]
    fn info_strings_are_kept_and_other_blocks_ignored() {
        assert_ne!(
            rust_blocks("```rust,no_run\nfn a() {}\n```\n"),
            rust_blocks("```rust\nfn a() {}\n```\n")
        );
        assert!(rust_blocks("```text\nanything\n```\n").is_empty());
    }

    #[test]
    fn a_missing_twin_on_either_side_is_an_error() {
        let root = std::env::temp_dir().join(format!("kivact-docs-pairs-{}", std::process::id()));
        for side in ["en", "jp"] {
            std::fs::create_dir_all(root.join("docs/for-kivact-user").join(side)).unwrap();
        }
        std::fs::write(root.join("README.md"), "").unwrap();
        std::fs::write(root.join("README_JP.md"), "").unwrap();
        std::fs::write(root.join("docs/for-kivact-user/en/only-en.md"), "").unwrap();
        std::fs::write(root.join("docs/for-kivact-user/jp/only-jp.md"), "").unwrap();
        let mut errors = Vec::new();
        super::check(&root, &mut errors);
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(errors.len(), 2, "{errors:?}");
    }
}
