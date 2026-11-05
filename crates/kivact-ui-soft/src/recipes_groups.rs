//! Role: Style standard joined actions and separated toggle groups in Neutral.
use crate::{Elevation, Tokens};
use kivact::style::{Align, Direction, Style, Theme, style};
use kivact::ui::{
    ButtonGroupMemberState, ButtonGroupRecipes, ButtonGroupState, Look, ToggleGroupRecipes,
    ToggleGroupState,
};
use kivact::view::Orientation;
/// Surface elevation of a joined action group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonGroupVariant(pub Elevation);
impl kivact::ui::Variant for ButtonGroupVariant {}

pub(crate) fn button_group(r: ButtonGroupRecipes) -> ButtonGroupRecipes {
    r.root(|_| root)
        .member(|_| member)
        .separator(|_| separator)
        .text(|_| text)
}
fn root(theme: &Theme, s: &Look<ButtonGroupState>) -> Style {
    let t = theme.ext::<Tokens>();
    let elevation = s
        .variant::<ButtonGroupVariant>()
        .copied()
        .unwrap_or_default()
        .0;
    elevation.apply(
        style()
            .direction(direction(s.orientation, s.rtl))
            .align_items(Align::Stretch)
            .radius(t.radius.element),
        &t,
    )
}
fn direction(orientation: Orientation, rtl: bool) -> Direction {
    if orientation == Orientation::Vertical {
        Direction::Column
    } else if rtl {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}
fn member(theme: &Theme, s: &Look<ButtonGroupMemberState>) -> Style {
    let t = theme.ext::<Tokens>();
    let first = if s.first { t.radius.element } else { 0.0 };
    let last = if s.last { t.radius.element } else { 0.0 };
    let corners = if s.layout.orientation == Orientation::Vertical {
        (first, first, last, last)
    } else if s.layout.rtl {
        (last, first, first, last)
    } else {
        (first, last, last, first)
    };
    style()
        .radius_corners(corners.0, corners.1, corners.2, corners.3)
        .clip(true)
}
fn separator(theme: &Theme, s: &Look<ButtonGroupState>) -> Style {
    let t = theme.ext::<Tokens>();
    let base = style()
        .shrink(0)
        .align_self(Align::Stretch)
        .background(t.colors.border);
    if s.orientation == Orientation::Vertical {
        base.height(1).margin_x(1)
    } else {
        base.width(1).margin_y(1)
    }
}
fn text(theme: &Theme, _: &Look<ButtonGroupState>) -> Style {
    let t = theme.ext::<Tokens>();
    style()
        .align_items(Align::Center)
        .gap(t.spacing.s2)
        .padding_x(t.spacing.s2)
        .border(t.sizes.border_width, t.colors.border)
        .background(t.colors.bg_muted)
        .color(t.colors.text_primary)
        .font_size(t.typography.supporting.size_px)
        .weight(t.typography.weight_medium)
        .text_wrap(false)
}
pub(crate) fn toggle_group(r: ToggleGroupRecipes) -> ToggleGroupRecipes {
    r.root(|_| {
        |theme: &Theme, s: &Look<ToggleGroupState>| {
            let t = theme.ext::<Tokens>();
            style()
                .direction(direction(s.orientation, s.rtl))
                .gap(s.spacing.map_or(t.spacing.s1, |gap| gap * 4.0))
                .align_items(if s.orientation == Orientation::Vertical {
                    Align::Stretch
                } else {
                    Align::Center
                })
        }
    })
    .item(|_| {
        |theme: &Theme, s: &Look<kivact::ui::ToggleGroupItemState>| {
            crate::recipes_controls::toggle(theme, &Look::new(s.toggle))
        }
    })
}
