//! Role: The word or line a double or triple click selects in a text input's value.
//!
//! Invariants:
//! - Offsets are byte offsets. An offset past the value is the end of it, and
//!   one inside a character is floored to that character's start, so both
//!   ranges start and end on char boundaries.
//! - A whitespace character is one `char::is_whitespace` accepts. A newline is
//!   never part of a whitespace run: a click on a newline selects that newline.
//! - A word is a run of characters that are not whitespace, so Japanese text
//!   with no spaces in it is one word (there is no word segmentation).

use kivact_types::text_utf::floor_char_boundary;
use std::ops::Range;

/// The range a double click at byte offset `at` selects: the maximal run,
/// around the character the click is on, of whitespace (not crossing a
/// newline) or of other characters. The character is the one starting at
/// `at`, or, at the end of the value, the one before it.
pub(crate) fn word_range(text: &str, at: usize) -> Range<usize> {
    let at = floor_char_boundary(text, at);
    let Some((clicked, c)) = clicked_char(text, at) else {
        return at..at;
    };
    if c == '\n' {
        return clicked;
    }
    let same: fn(char) -> bool = if c.is_whitespace() { is_blank } else { is_word };
    run_around(text, clicked, same)
}

/// The range a triple click at byte offset `at` selects: the line that
/// contains `at`, without its newline. A line runs from after the newline
/// before `at` to the newline after it (a caret before a newline is on the
/// line that newline ends). A single-line value is one line: all of it.
pub(crate) fn line_range(text: &str, at: usize, multiline: bool) -> Range<usize> {
    if !multiline {
        return 0..text.len();
    }
    let at = floor_char_boundary(text, at);
    let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    let end = text[at..].find('\n').map_or(text.len(), |i| at + i);
    start..end
}

/// The character a click at `at` is on and its byte range: the one starting
/// at `at`, or at the end of the value the one before it. `None` for an
/// empty value.
fn clicked_char(text: &str, at: usize) -> Option<(Range<usize>, char)> {
    match text[at..].chars().next() {
        Some(c) => Some((at..at + c.len_utf8(), c)),
        None => text[..at]
            .char_indices()
            .next_back()
            .map(|(i, c)| (i..at, c)),
    }
}

/// A character of a whitespace run: whitespace other than a newline.
fn is_blank(c: char) -> bool {
    c.is_whitespace() && c != '\n'
}

/// A character of a word: anything but whitespace.
fn is_word(c: char) -> bool {
    !c.is_whitespace()
}

/// The maximal run of characters that satisfy `same` around the character
/// at `hit`, which satisfies it too.
fn run_around(text: &str, hit: Range<usize>, same: fn(char) -> bool) -> Range<usize> {
    let start = text[..hit.start]
        .char_indices()
        .rev()
        .find(|&(_, c)| !same(c))
        .map_or(0, |(i, c)| i + c.len_utf8());
    let end = text[hit.end..]
        .find(|c: char| !same(c))
        .map_or(text.len(), |i| hit.end + i);
    start..end
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the double-click word and triple-click line ranges.

    use super::{line_range, word_range};

    #[test]
    fn a_double_click_selects_the_ascii_word_under_the_pointer() {
        let text = "hello world foo";
        assert_eq!(word_range(text, 0), 0..5, "the first word");
        assert_eq!(word_range(text, 6), 6..11, "at the word's first character");
        assert_eq!(word_range(text, 7), 6..11, "inside the word");
        assert_eq!(word_range(text, 10), 6..11, "at its last character");
        assert_eq!(word_range(text, 12), 12..15, "the last word");
    }

    #[test]
    fn a_click_on_a_space_selects_the_space() {
        assert_eq!(
            word_range("hello world", 5),
            5..6,
            "the space after the word"
        );
        assert_eq!(word_range("a   b", 2), 1..4, "the whole run of spaces");
        assert_eq!(word_range("a \t b", 2), 1..4, "tabs and spaces alike");
    }

    #[test]
    fn a_click_at_the_end_of_the_text_takes_the_character_before_it() {
        assert_eq!(
            word_range("hello world", 11),
            6..11,
            "the word before the end"
        );
        assert_eq!(word_range("abc ", 4), 3..4, "a space before the end");
    }

    #[test]
    fn a_whitespace_run_does_not_cross_a_newline() {
        // "a \n b": `a` 0, space 1, newline 2, space 3, `b` 4.
        assert_eq!(word_range("a \n b", 1), 1..2, "stops before the newline");
        assert_eq!(word_range("a \n b", 3), 3..4, "stops after the newline");
    }

    #[test]
    fn a_newline_selects_itself_and_ends_a_word() {
        // "a\nb": `a` 0, newline 1, `b` 2.
        assert_eq!(word_range("a\nb", 1), 1..2, "the newline alone");
        assert_eq!(word_range("a\nb", 2), 2..3, "the word after the newline");
        assert_eq!(
            word_range("a\nb", 3),
            2..3,
            "the end takes the character before"
        );
        assert_eq!(word_range("a\n", 2), 1..2, "a newline at the end");
    }

    #[test]
    fn an_empty_value_selects_nothing() {
        assert_eq!(word_range("", 0), 0..0);
        assert_eq!(line_range("", 0, true), 0..0);
    }

    #[test]
    fn a_word_of_japanese_text_is_its_run_of_multi_byte_characters() {
        // "日本語 テキスト": 日 0..3, 本 3..6, 語 6..9, space 9..10, テ 10..13, ...
        let text = "日本語 テキスト";
        assert_eq!(word_range(text, 0), 0..9, "the first character");
        assert_eq!(word_range(text, 4), 0..9, "inside 本 floors to its start");
        assert_eq!(word_range(text, 9), 9..10, "the space");
        assert_eq!(word_range(text, 10), 10..22, "the katakana word");
        assert_eq!(
            word_range(text, 22),
            10..22,
            "the end takes the character before"
        );
        assert_eq!(word_range(text, 1000), 10..22, "past the end clamps to it");
    }

    #[test]
    fn a_full_width_space_is_whitespace() {
        // "あ" 0..3, ideographic space 3..6, "い" 6..9.
        let text = "あ\u{3000}い";
        assert_eq!(word_range(text, 0), 0..3);
        assert_eq!(word_range(text, 3), 3..6);
        assert_eq!(word_range(text, 6), 6..9);
    }

    #[test]
    fn a_triple_click_selects_the_line_under_the_pointer() {
        // "one\ntwo\n\nthree": `one` 0..3, newline 3, `two` 4..7, newline 7,
        // an empty line at 8 (newline 8), `three` 9..14.
        let text = "one\ntwo\n\nthree";
        assert_eq!(line_range(text, 1, true), 0..3);
        assert_eq!(line_range(text, 5, true), 4..7);
        assert_eq!(line_range(text, 8, true), 8..8, "an empty line");
        assert_eq!(
            line_range(text, 14, true),
            9..14,
            "the end of the last line"
        );
    }

    #[test]
    fn a_newline_belongs_to_the_line_before_it() {
        let text = "one\ntwo";
        assert_eq!(
            line_range(text, 3, true),
            0..3,
            "the caret before the newline"
        );
        assert_eq!(
            line_range(text, 4, true),
            4..7,
            "after the newline is the next line"
        );
    }

    #[test]
    fn a_line_of_japanese_text_is_selected_by_its_bytes() {
        // "日本\n語": `日本` 0..6, newline 6, `語` 7..10.
        assert_eq!(line_range("日本\n語", 7, true), 7..10);
        assert_eq!(
            line_range("日本\n語", 4, true),
            0..6,
            "inside 本 floors to its start"
        );
    }

    #[test]
    fn a_single_line_value_is_one_line_even_with_a_newline_in_it() {
        assert_eq!(line_range("hello world", 3, false), 0..11);
        assert_eq!(line_range("a\nb", 2, false), 0..3);
    }
}
