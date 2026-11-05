//! Role: Lyra toggle and joined button group recipes.
use crate::Tokens;
use kivact::style::{Align, Direction, Style, Theme, style};
use kivact::ui::{
    ButtonGroupMemberState, ButtonGroupRecipes, ButtonGroupState, Look, ToggleGroupItemState,
    ToggleGroupRecipes, ToggleGroupState,
};
use kivact::view::Orientation;
pub(crate) fn toggle_group(r: ToggleGroupRecipes) -> ToggleGroupRecipes {
    r.root(|_| toggle_root).item(|_| toggle_item)
}
fn toggle_root(_: &Theme, s: &Look<ToggleGroupState>) -> Style {
    style()
        .direction(if s.orientation == Orientation::Vertical {
            Direction::Column
        } else if s.rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(if s.orientation == Orientation::Vertical {
            Align::Stretch
        } else {
            Align::Center
        })
        .gap(s.spacing.unwrap_or(2.0) * 4.0)
        .shrink(0)
        .align_self(Align::Start)
}
fn toggle_item(t: &Theme, s: &Look<ToggleGroupItemState>) -> Style {
    let variant = s
        .variant::<crate::ToggleVariant>()
        .copied()
        .unwrap_or_default();
    let joined = s.layout.spacing == Some(0.0);
    let base = crate::choice::toggle_style(
        &t.ext::<Tokens>(),
        variant,
        s.size,
        s.pressed,
        s.leading_icon,
        joined,
    );
    if joined && variant == crate::ToggleVariant::Outline && !s.first {
        if s.layout.orientation == Orientation::Vertical {
            base.border_top(0, kivact::style::Color::TRANSPARENT)
        } else if s.layout.rtl {
            base.border_right(0, kivact::style::Color::TRANSPARENT)
        } else {
            base.border_left(0, kivact::style::Color::TRANSPARENT)
        }
    } else {
        base
    }
}
pub(crate) fn button_group(r: ButtonGroupRecipes) -> ButtonGroupRecipes {
    r.root(|_| button_root)
        .member(|_| member)
        .separator(|_| separator)
        .text(|_| text)
}
fn button_root(_: &Theme, s: &Look<ButtonGroupState>) -> Style {
    style()
        .direction(if s.orientation == Orientation::Vertical {
            Direction::Column
        } else if s.rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Stretch)
        .radius(0)
}
fn member(_: &Theme, s: &Look<ButtonGroupMemberState>) -> Style {
    let base = style().shrink(0);
    if s.first {
        base
    } else if s.layout.orientation == Orientation::Vertical {
        base.margin_top(-1)
    } else if s.layout.rtl {
        base.margin_right(-1)
    } else {
        base.margin_left(-1)
    }
}
fn separator(t: &Theme, s: &Look<ButtonGroupState>) -> Style {
    let base = style()
        .shrink(0)
        .align_self(Align::Stretch)
        .background(t.ext::<Tokens>().input);
    if s.orientation == Orientation::Vertical {
        base.height(1).margin_x(1)
    } else {
        base.width(1).margin_y(1)
    }
}
fn text(t: &Theme, _: &Look<ButtonGroupState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .align_items(Align::Center)
        .gap(8)
        .padding_x(10)
        .border(1, p.border)
        .radius(0)
        .background(p.muted)
        .color(p.foreground)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
}
