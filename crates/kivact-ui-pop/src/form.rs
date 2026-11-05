//! Role: Editable form controls, including platform file drops and pickers.

use crate::Tokens;
use kivact::draw::Shadow;
use kivact::state::{Signal, Value, signal};
use kivact::style::{ColorToken, Theme, style};
pub(crate) fn input_style(t: &Theme) -> kivact::style::Style {
    let p = t.ext::<Tokens>();
    style()
        .height(36)
        .min_height(0)
        .padding_x(12)
        .padding_y(8)
        .radius(8)
        .border(1, p.line)
        .background(p.surface)
        .color(p.text)
        .font_size(14)
        .line_height(1.2)
        .shadow(Shadow::new(0, 1, 2, kivact::style::rgba(0x0f172a, 0.05)))
        .focus(style().border_color(p.blue).ring(3, ColorToken::Focus))
}
/// The text of a value input while it is being edited. It is the field's
/// content, so a partial entry such as `2027/1` stays as typed; the value
/// follows it only when the text parses. A value that changes elsewhere
/// replaces the text, unless the text already parses to that value.
pub(crate) struct Draft<T: 'static> {
    text: Signal<String>,
    parse: fn(&str) -> Option<T>,
    format: fn(T) -> String,
}

impl<T> Clone for Draft<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Draft<T> {}

impl<T: Copy + PartialEq + 'static> Draft<T> {
    /// Show `value` formatted, and keep following it.
    pub(crate) fn new(
        value: Value<T>,
        parse: fn(&str) -> Option<T>,
        format: fn(T) -> String,
    ) -> Self {
        let draft = Draft {
            text: signal(format(value.get_untracked())),
            parse,
            format,
        };
        kivact::state::effect(move || {
            let external = value.get();
            if (draft.parse)(&draft.text.get_untracked()) != Some(external) {
                draft.show(external);
            }
        });
        draft
    }

    /// The field's text, which the text input of the date field edits.
    pub(crate) fn text(self) -> Signal<String> {
        self.text
    }

    /// The value `text` stands for, if it is complete.
    pub(crate) fn parse(self, text: &str) -> Option<T> {
        (self.parse)(text)
    }

    /// Replace the text with `value` formatted (a choice made elsewhere).
    pub(crate) fn show(self, value: T) {
        self.text.set((self.format)(value));
    }
}
