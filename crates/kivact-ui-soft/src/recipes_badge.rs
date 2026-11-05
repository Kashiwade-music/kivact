//! Role: Color canonical badges with Neutral semantic and categorical appearance.
use crate::Tokens;
use kivact::interaction::Tone;
use kivact::style::{Align, Justify, Style, Theme, pct, style};
use kivact::ui::{BadgeState, Look};
/// Categorical colors for badges; use the standard tone for semantic statuses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    /// Use the neutral badge palette.
    Neutral,
    /// Use the blue categorical badge palette.
    Blue,
    /// Use the cyan categorical badge palette.
    Cyan,
    /// Use the green categorical badge palette.
    Green,
    /// Use the orange categorical badge palette.
    Orange,
    /// Use the pink categorical badge palette.
    Pink,
    /// Use the purple categorical badge palette.
    Purple,
    /// Use the red categorical badge palette.
    Red,
    /// Use the teal categorical badge palette.
    Teal,
    /// Use the yellow categorical badge palette.
    Yellow,
    /// Use the gray categorical badge palette.
    Gray,
}
impl kivact::ui::Variant for BadgeVariant {}
pub(crate) fn badge(theme: &Theme, s: &Look<BadgeState>) -> Style {
    let t = theme.ext::<Tokens>();
    let c = &t.colors;
    if s.variant::<crate::RowStatusDot>().is_some() {
        let color = match s.tone {
            Tone::Success => c.text_success,
            Tone::Warning => c.text_warning,
            Tone::Danger => c.text_error,
            _ => c.icon_secondary,
        };
        return style()
            .size(8, 8)
            .min_height(8)
            .min_width(8)
            .padding(0)
            .gap(0)
            .radius(4)
            .background(color);
    }
    let (fill, ink) = match s.tone {
        Tone::Info | Tone::Accent => (c.info, c.on_info),
        Tone::Success => (c.success, c.on_success),
        Tone::Warning => (c.warning, c.on_warning),
        Tone::Danger => (c.error, c.on_error),
        Tone::Neutral => {
            let hue = match s.variant::<BadgeVariant>().copied().unwrap_or_default() {
                BadgeVariant::Blue => c.hue.blue,
                BadgeVariant::Cyan => c.hue.cyan,
                BadgeVariant::Green => c.hue.green,
                BadgeVariant::Orange => c.hue.orange,
                BadgeVariant::Pink => c.hue.pink,
                BadgeVariant::Purple => c.hue.purple,
                BadgeVariant::Red => c.hue.red,
                BadgeVariant::Teal => c.hue.teal,
                BadgeVariant::Yellow => c.hue.yellow,
                _ => c.hue.gray,
            };
            (hue.bg, hue.text)
        }
    };
    style()
        .min_height(t.sizes.badge_height)
        .min_width(0)
        .max_width(pct(100))
        .radius(t.radius.full)
        .padding_x(t.spacing.s2)
        .gap(t.spacing.s1)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(fill)
        .color(ink)
        .font_size(12)
        .line_height(20.0 / 12.0)
        .weight(500)
        .text_wrap(false)
}
