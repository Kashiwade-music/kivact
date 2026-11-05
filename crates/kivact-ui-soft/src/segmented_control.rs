//! Role: Style the standard segmented radio-group presentation with Neutral layout options.
use crate::Tokens;
use crate::recipe_style::{fast, ring};
use kivact::style::{Align, Color, Cursor, Direction, Justify, Style, Theme, pct, style};
use kivact::ui::{CheckableState, ControlSize, Look, Variant};
use kivact::view::Orientation;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SegmentedLayout {
    #[default]
    Hug,
    Fill,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SegmentedIconOnly;
impl Variant for SegmentedLayout {}
impl Variant for SegmentedIconOnly {}
pub(crate) fn track(theme: &Theme, look: &Look<Orientation>) -> Style {
    let t = theme.ext::<Tokens>();
    let mut base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(2)
        .padding(4)
        .radius(t.radius.element)
        .background(t.colors.neutral);
    if look.variant::<SegmentedLayout>() == Some(&SegmentedLayout::Fill) {
        base = base.width(pct(100));
    }
    if look.variant::<SegmentedIconOnly>().is_some() {
        base.shrink(0)
    } else {
        base.max_width(pct(100)).min_width(0)
    }
}

pub(crate) fn segment(theme: &Theme, look: &Look<CheckableState>) -> Style {
    let t = theme.ext::<Tokens>();
    let c = &t.colors;
    let (height, padding, font) = match look.size {
        ControlSize::ExtraSmall | ControlSize::Small => (20, 8, 12),
        ControlSize::Medium => (24, 12, 14),
        ControlSize::Large => (28, 12, 14),
    };
    let selected = look.checked.is_checked();
    let (fill, ink) = if selected {
        (c.bg_surface, c.text_primary)
    } else {
        (Color::TRANSPARENT, c.text_secondary)
    };
    let mut base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .height(height)
        .min_width(0)
        .radius(8)
        .padding_x(padding)
        .background(fill)
        .color(ink)
        .font_size(font)
        .line_height(20.0 / font as f64)
        .weight(if selected { 600 } else { 500 })
        .cursor(Cursor::Pointer)
        .focus_visible(ring(&t, c.focus_ring))
        .disabled(style().color(c.text_disabled).cursor(Cursor::Default));
    if !selected && !look.input.disabled {
        base = base
            .hover(style().background(c.overlay_hover))
            .pressed(style().background(c.overlay_pressed));
    }
    if look.variant::<SegmentedLayout>() == Some(&SegmentedLayout::Fill) {
        base = base.grow(1).basis(0);
    }
    fast(&t, base)
}

kivact::state::constant_value!(SegmentedLayout);
