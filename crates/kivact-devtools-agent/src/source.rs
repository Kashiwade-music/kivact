//! Role: Read a source line to find the name a signal was bound to.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use kivact_devtools_protocol::SourceLocation;

const MAX_FILES: usize = 64;

/// Source files read so far (a pure cache; the oldest is dropped first).
pub(crate) struct SourceCache {
    files: Vec<(PathBuf, Rc<str>)>,
}

impl SourceCache {
    pub(crate) fn new() -> SourceCache {
        SourceCache { files: Vec::new() }
    }

    /// The identifier after `let` (and `mut`) on the line at `location`; `location.file` is
    /// relative to `root`.
    pub(crate) fn name_hint(&mut self, root: &Path, location: &SourceLocation) -> Option<String> {
        let text = self.text(&root.join(&location.file))?;
        let index = usize::try_from(location.line).ok()?.checked_sub(1)?;
        hint_in_line(text.lines().nth(index)?)
    }

    fn text(&mut self, path: &Path) -> Option<Rc<str>> {
        if let Some((_, text)) = self.files.iter().find(|(known, _)| known == path) {
            return Some(Rc::clone(text));
        }
        let text: Rc<str> = std::fs::read_to_string(path).ok()?.into();
        if self.files.len() >= MAX_FILES {
            self.files.remove(0);
        }
        self.files.push((path.to_path_buf(), Rc::clone(&text)));
        Some(text)
    }
}

fn hint_in_line(line: &str) -> Option<String> {
    let word_char = |c: char| c.is_alphanumeric() || c == '_';
    let start = line.match_indices("let ").find_map(|(index, _)| {
        let before = line[..index].chars().next_back();
        (!before.is_some_and(word_char)).then_some(index + 4)
    })?;
    let rest = line[start..].trim_start();
    let rest = rest.strip_prefix("mut ").unwrap_or(rest).trim_start();
    let name: String = rest.chars().take_while(|c| word_char(*c)).collect();
    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the identifier found after `let` on a source line.

    use super::hint_in_line;

    #[test]
    fn finds_the_bound_name() {
        assert_eq!(
            hint_in_line("    let mut count = signal(0);"),
            Some("count".to_owned())
        );
        assert_eq!(hint_in_line("let (a, b) = pair();"), None);
        assert_eq!(hint_in_line("outlet x"), None);
        assert_eq!(hint_in_line("let _ = x"), Some("_".to_owned()));
    }
}
