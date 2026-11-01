//! Role: Type-to-find navigation for menus and lists.
//!
//! Type-to-find: the pure policy behind letter keys in menus, listboxes and
//! palettes. A control keeps one [`Typeahead`], feeds it each typed character
//! with the current time and asks [`typeahead_find`] where to move.

/// Characters typed within this long of the previous one are matched
/// together as one prefix.
const TYPEAHEAD_WINDOW_NS: u64 = 500_000_000;

/// The characters typed so far and when the last one arrived.
#[derive(Clone, Debug, Default)]
pub(crate) struct Typeahead {
    text: String,
    at_ns: u64,
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
    /// Add a typed character at `now_ns`; returns what to search for.
    ///
    /// A character typed more than 500 ms after the previous
    /// one starts a new prefix; otherwise it extends the prefix. One
    /// character repeated ("aa") searches for that character alone, so it
    /// cycles through the items that start with it.
    pub(crate) fn push(&mut self, c: char, now_ns: u64) -> TypeaheadQuery {
        if self.text.is_empty() || now_ns.saturating_sub(self.at_ns) > TYPEAHEAD_WINDOW_NS {
            self.text.clear();
        }
        self.text.extend(c.to_lowercase());
        self.at_ns = now_ns;
        let mut chars = self.text.chars();
        let first = chars.next();
        let count = self.text.chars().count();
        let repeated = count > 1 && first.is_some_and(|f| chars.all(|ch| ch == f));
        match (repeated, first) {
            (true, Some(first)) => TypeaheadQuery {
                needle: first.to_string(),
                from_next: true,
            },
            _ => TypeaheadQuery {
                needle: self.text.clone(),
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
