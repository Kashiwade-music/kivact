//! Role: Soft recipes for actual canonical item-list and item-row parts.
use crate::{ItemAlign, ItemDensity, ItemLayout, ItemLook, ListEdge, ListLook, Tokens};
use kivact::custom::PartState;
use kivact::style::{Align, Cursor, Direction, Justify, Style, Theme, style};
use kivact::ui::{ItemListMarker, ItemListRecipes, ItemRowState, Look};
fn density(s: &Look<ItemRowState>) -> ItemDensity {
    s.variant::<ItemLook>()
        .and_then(|look| look.density.map(|v| v.get()))
        .or_else(|| s.variant::<ItemDensity>().copied())
        .unwrap_or_else(|| {
            s.variant::<ListLook>()
                .map_or(ItemDensity::Balanced, |look| look.density.get())
        })
}
fn alignment(s: &Look<ItemRowState>) -> ItemAlign {
    s.variant::<ItemLook>()
        .and_then(|look| look.align.map(|v| v.get()))
        .or_else(|| s.variant::<ItemAlign>().copied())
        .unwrap_or_default()
}
fn layout(s: &Look<ItemRowState>) -> ItemLayout {
    s.variant::<ItemLook>()
        .and_then(|look| look.layout.map(|v| v.get()))
        .or_else(|| s.variant::<ItemLayout>().copied())
        .unwrap_or_default()
}
pub(crate) fn recipes(r: ItemListRecipes) -> ItemListRecipes {
    r.root(|_| root)
        .item(|_| item)
        .body(|_| body)
        .content(|_| content)
        .title(|_| title)
        .description(|_| description)
        .media(|_| media)
        .actions(|_| media)
        .marker(|_| marker)
        .list_header(|_| list_header)
}
fn root(_: &Theme, s: &Look<PartState>) -> Style {
    style().direction(Direction::Column).gap(
        if s.variant::<ListLook>()
            .is_some_and(|look| look.dividers.get())
        {
            0
        } else {
            2
        },
    )
}
fn list_header(_: &Theme, _: &Look<PartState>) -> Style {
    style().direction(Direction::Column).min_width(0)
}
fn item(t: &Theme, s: &Look<ItemRowState>) -> Style {
    let p = t.ext::<Tokens>();
    let look = s.variant::<ListLook>();
    let edge = s
        .variant::<ListEdge>()
        .copied()
        .unwrap_or_else(|| look.map_or(ListEdge::Padded, |look| look.edge.get()));
    let highlighted = s
        .variant::<ItemLook>()
        .is_some_and(|look| look.highlighted.get());
    let selected = if s.actionable && s.is_pressed() {
        p.colors.overlay_pressed
    } else if s.actionable && s.is_hovered() {
        p.colors.overlay_hover
    } else {
        p.colors.accent_muted
    };
    let base = style()
        .direction(Direction::Column)
        .min_width(0)
        .padding_x(if edge == ListEdge::Flush { 0 } else { 12 })
        .padding_y(density(s).padding())
        .radius(if edge == ListEdge::Bleed {
            0.0
        } else {
            p.radius.element
        })
        .background(if highlighted {
            p.colors.overlay_hover
        } else {
            kivact::style::Color::TRANSPARENT
        })
        .selected(style().background(selected));
    let base = if look.is_some_and(|look| look.dividers.get())
        && s.position.is_some_and(|p| p.index + 1 < p.count)
    {
        base.border_bottom(1, p.colors.border)
    } else {
        base
    };
    if s.actionable {
        base.cursor(Cursor::Pointer)
            .hover(style().background(p.colors.overlay_hover))
            .pressed(style().background(p.colors.overlay_pressed))
            .focus_visible(style().ring(p.sizes.focus_ring_width, p.colors.focus_ring))
            .transition(
                kivact::style::TransitionProp::All,
                kivact::motion::Transition::tween(
                    p.motion.fast as f64,
                    p.motion.ease_standard.clone(),
                ),
            )
    } else {
        base
    }
}
fn body(_: &Theme, s: &Look<ItemRowState>) -> Style {
    style()
        .direction(Direction::Row)
        .min_width(0)
        .gap(8)
        .align_items(if alignment(s) == ItemAlign::Center {
            Align::Center
        } else {
            Align::Start
        })
}
fn content(_: &Theme, s: &Look<ItemRowState>) -> Style {
    let inline = layout(s) == ItemLayout::Inline;
    style()
        .grow(1)
        .min_width(0)
        .direction(if inline {
            Direction::Row
        } else {
            Direction::Column
        })
        .gap(if inline { 8 } else { 0 })
}
fn title(t: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .font_size(14)
        .line_height(20.0 / 14.0)
        .weight(400)
        .color(t.ext::<Tokens>().colors.text_primary)
}
fn description(t: &Theme, _: &Look<ItemRowState>) -> Style {
    style()
        .font_size(12)
        .line_height(20.0 / 12.0)
        .color(t.ext::<Tokens>().colors.text_secondary)
}
fn media(_: &Theme, _: &Look<ItemRowState>) -> Style {
    style().shrink(0).min_width(0)
}
fn marker(_: &Theme, s: &Look<ItemRowState>) -> Style {
    let base = style()
        .direction(Direction::Row)
        .justify_content(Justify::End)
        .shrink(0)
        .min_width(0);
    match s.marker {
        ItemListMarker::None => base,
        ItemListMarker::Decimal => {
            let count = s.position.map_or(1, |p| p.count);
            let digits = (u64::from(s.start) + count.saturating_sub(1) as u64)
                .to_string()
                .len();
            base.width(digits as f64 * 8.0 + 8.0)
        }
        _ => base.width(16),
    }
}
