//! Role: Asset path URL detection for loader policy.
use std::path::Path;

/// Whether `path` is a URL: a scheme of two or more characters followed by
/// `://` (a one-letter scheme would be a Windows drive).
pub(crate) fn is_url(path: &Path) -> bool {
    let Some(text) = path.to_str() else {
        return false;
    };
    let Some((scheme, _)) = text.split_once("://") else {
        return false;
    };
    let mut chars = scheme.chars();
    scheme.len() >= 2
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}
