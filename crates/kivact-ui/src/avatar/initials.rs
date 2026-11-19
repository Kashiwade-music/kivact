//! Role: Derive first and last name initials while retaining joined Unicode glyphs.
pub(super) fn initials(name: &str) -> String {
    let mut letters = name.split_whitespace().filter_map(initial);
    let Some(first) = letters.next() else {
        return String::new();
    };
    let last = letters.next_back().unwrap_or_default();
    format!("{first}{last}").to_uppercase()
}

fn initial(word: &str) -> Option<&str> {
    let is_flag = |c| matches!(c, '\u{1f1e6}'..='\u{1f1ff}');
    let mut chars = word.char_indices();
    let (start, first) = chars.find(|&(_, c)| {
        c.is_alphanumeric() || matches!(c, '\u{1f000}'..='\u{1faff}' | '\u{2600}'..='\u{27bf}')
    })?;
    let mut end = start + first.len_utf8();
    let mut join = false;
    for (i, c) in chars {
        let extend = matches!(c, '\u{0300}'..='\u{036f}' | '\u{fe0e}'..='\u{fe0f}' | '\u{1f3fb}'..='\u{1f3ff}' | '\u{200d}');
        if !join && !extend && !(is_flag(first) && is_flag(c) && i == start + first.len_utf8()) {
            break;
        }
        join = c == '\u{200d}';
        end = i + c.len_utf8();
    }
    Some(&word[start..end])
}
