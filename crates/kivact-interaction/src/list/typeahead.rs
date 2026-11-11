//! Role: Type-to-find navigation for menus and lists.
//!
//! Type-to-find: the pure policy behind letter keys in menus, listboxes and
//! palettes. A control keeps one [`Typeahead`], feeds it each typed character
//! and asks [`typeahead_find`] where to move. The owning machine ends the
//! prefix with `Step::after` after [`TYPEAHEAD_WINDOW`].

use std::time::Duration;

/// Characters typed within this long of the previous one are matched
/// together as one prefix.
pub(crate) const TYPEAHEAD_WINDOW: Duration = Duration::from_millis(500);

/// The characters typed so far.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum Typeahead {
    #[default]
    Idle,
    Typing {
        text: String,
    },
}

/// What to search for after a typed character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TypeaheadQuery {
    /// Lowercase prefix to match.
    pub(crate) needle: String,
    /// Start after the current item instead of at it. This is how one
    /// character (typed alone, or repeated) moves on to the next match
    /// instead of staying where it is.
    pub(crate) from_next: bool,
}

impl Typeahead {
    /// Add a typed character; returns what to search for.
    ///
    /// The owning machine ends the prefix after the window, so a character
    /// always extends the current prefix here. One
    /// character repeated ("aa") searches for that character alone, so it
    /// cycles through the items that start with it.
    pub(crate) fn push(&mut self, c: char) -> TypeaheadQuery {
        let mut text = match std::mem::take(self) {
            Self::Typing { text } => text,
            Self::Idle => String::new(),
        };
        text.extend(c.to_lowercase());
        *self = Self::Typing { text: text.clone() };
        let mut chars = text.chars();
        let first = chars.next();
        let count = text.chars().count();
        let repeated = count > 1 && first.is_some_and(|f| chars.all(|ch| ch == f));
        match (repeated, first) {
            (true, Some(first)) => TypeaheadQuery {
                needle: first.to_string(),
                from_next: true,
            },
            _ => TypeaheadQuery {
                needle: text.clone(),
                from_next: count == 1,
            },
        }
    }
}

/// The next enabled item after `current` (or from it when `from_next` is
/// false), cycling past the end to the start, whose lowercase label starts
/// with the query's needle. Items are `(key, label, disabled)`; labels are
/// lowercased here. Without a current enabled item the search starts at the
/// first enabled one.
pub(crate) fn typeahead_find<K: Clone + PartialEq>(
    items: &[(K, String, bool)],
    current: Option<&K>,
    query: &TypeaheadQuery,
) -> Option<K> {
    let candidates: Vec<(&K, String)> = items
        .iter()
        .filter(|(_, _, disabled)| !disabled)
        .map(|(key, label, _)| (key, label.to_lowercase()))
        .collect();
    let at = current.and_then(|current| candidates.iter().position(|(key, _)| *key == current));
    let start = match at {
        Some(at) if query.from_next => at + 1,
        Some(at) => at,
        None => 0,
    };
    (0..candidates.len())
        .map(|offset| &candidates[(start + offset) % candidates.len()])
        .find(|(_, label)| label.starts_with(&query.needle))
        .map(|(key, _)| (*key).clone())
}
