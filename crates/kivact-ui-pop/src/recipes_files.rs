//! Role: Apply Pop surfaces to standard file drop targets.
use crate::{Hue, Tokens};
use kivact::interaction::DropzoneState;
use kivact::style::{Align, BorderStyle, ColorToken, Direction, Justify, Style, Theme, style};
use kivact::ui::{DropzoneRecipes, Look};

pub(crate) fn dropzone(r: DropzoneRecipes) -> DropzoneRecipes {
    r.root(|_| root)
        .icon(|_| icon)
        .label(|_| label)
        .hint(|_| hint)
        .status(|_| status)
}
fn root(t: &Theme, s: &Look<DropzoneState>) -> Style {
    let p = t.ext::<Tokens>();
    let hovered = matches!(**s, DropzoneState::DragOver { .. });
    let invalid = matches!(
        **s,
        DropzoneState::Failed(_)
            | DropzoneState::DragOver {
                valid: kivact::interaction::Validity::Invalid(_)
            }
    );
    style()
        .direction(Direction::Column)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .padding_x(16)
        .padding_y(24)
        .gap(4)
        .border(
            1,
            if invalid {
                p.red
            } else if hovered {
                p.blue
            } else {
                p.line_strong
            },
        )
        .border_style(BorderStyle::Dashed)
        .radius(10)
        .background(if hovered {
            p.hue(Hue::Blue).0
        } else {
            p.surface_muted
        })
        .hover(style().border_color(p.blue))
}
fn icon(t: &Theme, _: &Look<DropzoneState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .size(36, 36)
        .margin_bottom(8)
        .radius(8)
        .background(p.surface)
        .border(1, p.line)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(p.text_muted)
}
fn label(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style().weight(500)
}
fn hint(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style().font_size(12).color(ColorToken::TextMuted)
}
fn status(_: &Theme, _: &Look<DropzoneState>) -> Style {
    style().font_size(12).color(ColorToken::Danger)
}
