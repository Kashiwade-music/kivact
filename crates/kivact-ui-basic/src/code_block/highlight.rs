//! Role: Map pretokenized syntax spans to the current theme palette.
use super::{CodeLine, CodeTone};
use crate::CodeColors;
use kivact::style::{Color, StyledRange};
/// The color of `tone`; plain text and punctuation keep the foreground.
fn tone_color(colors: &CodeColors, tone: CodeTone) -> Option<Color> {
    match tone {
        CodeTone::Plain | CodeTone::Punctuation => None,
        CodeTone::Keyword => Some(colors.keyword),
        CodeTone::String => Some(colors.string),
        CodeTone::Number => Some(colors.number),
        CodeTone::Comment => Some(colors.comment),
        CodeTone::Function => Some(colors.function),
        CodeTone::Type => Some(colors.type_name),
        CodeTone::Macro => Some(colors.macro_call),
        CodeTone::Lifetime => Some(colors.lifetime),
        CodeTone::Attribute => Some(colors.attribute),
    }
}

/// Use cached whole-source spans and the current theme palette as a Core highlight callback.
/// The colored ranges of `text`, the editor's line `index`, from the
/// spans of that [`CodeLine`] (the text is theirs, joined); and the index
/// of the next line, the state the editor passes on.
pub fn rust_lines(
    (lines, colors): &(Vec<CodeLine>, CodeColors),
    text: &str,
    index: &usize,
) -> (Vec<StyledRange>, usize) {
    let mut out = Vec::new();
    let mut at = 0;
    for span in lines.get(*index).map_or(&[][..], |line| &line.0) {
        let range = at.min(text.len())..(at + span.text.len()).min(text.len());
        at += span.text.len();
        if let Some(color) = tone_color(colors, span.tone)
            && !range.is_empty()
        {
            out.push(StyledRange {
                range,
                color: Some(color),
                background: None,
                decoration: None,
            });
        }
    }
    (out, index + 1)
}
