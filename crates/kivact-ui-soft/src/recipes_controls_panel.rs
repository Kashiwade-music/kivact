//! Role: Style Astryx Neutral disclosures, splitters and splits (the controls group's panel parts).
use crate::color as c;
use crate::metrics as m;
use crate::radius as r;
use crate::recipe_parts::fast;
use crate::space as s;
use crate::type_scale as t;
use crate::variants::{DisclosureChevron, DisclosureDensity, DisclosureDividers};
use kivact::geometry::Rect;
use kivact::style::{Align, Color, Cursor, FlexDirection, Justify, Style, Theme, pct, style};
use kivact::ui::theme::{
    AxisState, DisclosureRecipeState, DisclosureTheme, Look, SplitTheme, SplitterTheme,
};
use kivact::view::{Orientation, ReadingDirection, use_direction};

/// The focus ring of a panel control: the kit's ring width in the focus color.
fn focus() -> Style {
    style().outline(m::FOCUS_RING_WIDTH, Color::var(&c::FOCUS_RING_VAR))
}

/// Whether the disclosure draws a separator before each grouped panel after the first.
fn dividers(look: &Look<DisclosureRecipeState>) -> bool {
    look.variant::<DisclosureDividers>()
        .is_some_and(|dividers| dividers.0)
}

/// The vertical padding of a disclosure's trigger and content.
fn padding(theme: &Theme, look: &Look<DisclosureRecipeState>) -> f64 {
    match look
        .variant::<DisclosureDensity>()
        .or(dividers(look).then_some(DisclosureDensity::Balanced))
    {
        Some(DisclosureDensity::Compact) => theme.get(&s::SPACE_S1_VAR),
        Some(DisclosureDensity::Balanced) => theme.get(&s::SPACE_S2_VAR),
        Some(DisclosureDensity::Spacious) => theme.get(&s::SPACE_S3_VAR),
        None => 0.0,
    }
}

fn root(_theme: &Theme, look: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    let base = style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100));
    if dividers(look) && look.group.is_some_and(|group| group.index > 0) {
        base.border_top(m::BORDER_WIDTH, c::BORDER)
    } else {
        base
    }
}

fn trigger(theme: &Theme, look: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    let start = look.variant::<DisclosureChevron>() == Some(DisclosureChevron::Start);
    let rtl = use_direction() == ReadingDirection::Rtl;
    let base = style()
        .flex_direction(if start != rtl {
            FlexDirection::RowReverse
        } else {
            FlexDirection::Row
        })
        .width(pct(100))
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(s::SPACE_S2)
        .padding_y(padding(theme, look))
        .font_size(t::LARGE.size_px)
        .line_height(1.0)
        .font_weight(t::WEIGHT_SEMIBOLD)
        .color(c::TEXT_PRIMARY)
        .opacity(if look.input.is_disabled() {
            m::DISABLED_OPACITY
        } else {
            1.0
        })
        .active(style().background(c::OVERLAY_PRESSED))
        .focus_visible(focus());
    fast(base)
}

/// The chevron: turned by the open state and mirrored for a start chevron in RTL.
fn indicator(_theme: &Theme, look: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    let start = look.variant::<DisclosureChevron>() == Some(DisclosureChevron::Start);
    let rtl = use_direction() == ReadingDirection::Rtl;
    let degrees = match (start, look.open) {
        (true, true) | (false, false) => 0.0,
        (true, false) => -90.0,
        (false, true) => 180.0,
    };
    fast(
        style()
            .size(t::LARGE.size_px, t::LARGE.size_px)
            .flex_shrink(0)
            .color(c::ICON_SECONDARY)
            .scale_xy(if start && rtl { -1.0 } else { 1.0 }, 1.0)
            .rotate(degrees),
    )
}

fn content(theme: &Theme, look: &Look<DisclosureRecipeState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .padding_top(s::SPACE_S1)
        .padding_bottom(padding(theme, look))
        .font_size(t::BODY.size_px)
        .line_height(t::BODY.line_height_px / t::BODY.size_px)
        .color(c::TEXT_PRIMARY)
}

/// The handle between two panes: a hairline divider in a wider hit area.
fn handle(theme: &Theme, look: &Look<AxisState>, _base: Style) -> Style {
    let hit = theme.get(&s::SPACE_S4_VAR);
    let vertical = look.orientation == Orientation::Vertical;
    // Splitter has no grip child, while Split only builds one when its
    // caller opts in. Keep the shared handle visible as a divider.
    let divider = if look.input.is_pressed() {
        c::BORDER_EMPHASIZED
    } else {
        c::BORDER
    };
    let line = m::BORDER_WIDTH;
    let base = style()
        .paint(move |draw, size| {
            let rect = if vertical {
                Rect::new(0.0, (size.height - line) / 2.0, size.width, line)
            } else {
                Rect::new((size.width - line) / 2.0, 0.0, line, size.height)
            };
            draw.fill_rect(rect, divider);
        })
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0)
        .z_index(2)
        .focus_visible(focus());
    if vertical {
        base.width(pct(100))
            .height(hit)
            .margin_y(-hit / 2.0)
            .cursor(if look.input.is_disabled() {
                Cursor::Default
            } else {
                Cursor::NsResize
            })
    } else {
        base.height(pct(100))
            .width(hit)
            .margin_x(-hit / 2.0)
            .cursor(if look.input.is_disabled() {
                Cursor::Default
            } else {
                Cursor::EwResize
            })
    }
}

/// The grip on a split's handle: a short rounded bar across the handle.
fn grip(theme: &Theme, look: &Look<AxisState>, _base: Style) -> Style {
    let long = theme.get(&s::SPACE_S8_VAR);
    let grip = fast(
        style()
            .flex_shrink(0)
            .border_radius(r::RADIUS_FULL)
            .background(if look.input.is_pressed() {
                c::BORDER_EMPHASIZED
            } else {
                c::BORDER
            }),
    );
    if look.orientation == Orientation::Vertical {
        grip.size(long, 3.0).translate_y(-5.5)
    } else {
        grip.size(3.0, long)
            .translate_x(if use_direction() == ReadingDirection::Rtl {
                5.5
            } else {
                -5.5
            })
    }
}

/// Install the kit's recipes of the disclosure, the splitter and the split.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            DisclosureTheme::default()
                .root(root)
                .trigger(trigger)
                .indicator(indicator)
                .content(content),
        )
        .with(SplitterTheme::default().handle(handle))
        .with(SplitTheme::default().grip(grip))
}
