//! Role: Writes one line of drawn text as an SVG `<text>` element.
//!
//! The run's origin is the top of its line box; the SVG baseline is that
//! origin plus the baseline the same text engine measures for the run, so
//! the text sits where a window draws it. Font features and text
//! decorations have no SVG 1.1 form and are not supported.

use super::emitter::Emitter;
use super::encode::{escape, hex, number, opacity_attribute};
use crate::Result;
use kivact_types::TextRun;

/// The generic families SVG names without quotes.
const GENERIC_FAMILIES: [&str; 4] = ["serif", "sans-serif", "monospace", "cursive"];

impl Emitter<'_> {
    pub(super) fn text(&mut self, run: &TextRun) -> Result<()> {
        if !run.features.is_empty() {
            return Err(self.unsupported("OpenType font features on text"));
        }
        if !run.decoration.lines.is_empty() {
            return Err(self.unsupported("text decorations"));
        }
        let baseline = self.measurer.measure(run).baseline;
        let family = run
            .family
            .clone()
            .or_else(|| self.inherited.family.clone())
            .unwrap_or_else(|| "sans-serif".to_owned());
        let mut attributes = format!(
            "x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" font-weight=\"{}\"",
            number(run.origin.x),
            number(run.origin.y + baseline),
            family_list(&family),
            number(run.font_size),
            run.weight
        );
        if run.italic {
            attributes.push_str(" font-style=\"italic\"");
        }
        attributes.push_str(&format!(
            " fill=\"{}\"{}",
            hex(run.color),
            opacity_attribute("fill", run.color)
        ));
        if run.letter_spacing != 0.0 {
            attributes.push_str(&format!(
                " letter-spacing=\"{}\"",
                number(run.letter_spacing)
            ));
        }
        if run.word_spacing != 0.0 {
            attributes.push_str(&format!(" word-spacing=\"{}\"", number(run.word_spacing)));
        }
        self.body.push_str(&format!(
            "<text {attributes} xml:space=\"preserve\">{}</text>\n",
            escape(&run.text)
        ));
        Ok(())
    }
}

/// The `font-family` value of a family name: a generic family as is, a
/// name with a quoted-string form otherwise, with `system-ui` as sans-serif.
fn family_list(family: &str) -> String {
    let family = family.trim();
    if family == "system-ui" || GENERIC_FAMILIES.contains(&family) {
        let generic = if family == "system-ui" {
            "sans-serif"
        } else {
            family
        };
        return escape(generic);
    }
    format!("'{}', sans-serif", escape(family))
}
