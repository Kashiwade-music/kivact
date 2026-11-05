//! Role: Style canonical grouped controls and their adornments.
use kivact::style::{Align, Direction, Justify, Theme, style};
use kivact::ui::{InputGroupMemberState, InputGroupRecipes, InputGroupState, Look};
use kivact::view::ReadingDirection;
pub(crate) fn input_group(r: InputGroupRecipes) -> InputGroupRecipes {
    r.root(|_| {
        |_: &Theme, _: &Look<InputGroupState>| {
            style().direction(Direction::Column).gap(4).min_width(0)
        }
    })
    .label(|_| {
        |theme: &Theme, _: &Look<InputGroupState>| {
            let t = theme.ext::<crate::Tokens>();
            style()
                .font_size(t.typography.body.size_px)
                .weight(t.typography.weight_medium)
                .color(t.colors.text_primary)
        }
    })
    .members(|_| {
        |_: &Theme, s: &Look<InputGroupState>| {
            style()
                .direction(if s.direction == ReadingDirection::Rtl {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .align_items(Align::Stretch)
        }
    })
    .member(|_| {
        |theme: &Theme, s: &Look<InputGroupMemberState>| {
            let t = theme.ext::<crate::Tokens>();
            let rtl = s.group.direction == ReadingDirection::Rtl;
            let (left, right) = if rtl {
                (s.last, s.first)
            } else {
                (s.first, s.last)
            };
            let left = if left { t.radius.element } else { 0.0 };
            let right = if right { t.radius.element } else { 0.0 };
            let base = style()
                .height(crate::recipes_field::height(&t, s.group.size))
                .radius_corners(left, right, right, left);
            let base = if s.first {
                base
            } else if rtl {
                base.margin_right(-1)
            } else {
                base.margin_left(-1)
            };
            if s.focused { base.z_index(1) } else { base }
        }
    })
    .addon(|_| {
        |theme: &Theme, s: &Look<InputGroupState>| {
            let t = theme.ext::<crate::Tokens>();
            style()
                .height(crate::recipes_field::height(&t, s.size))
                .padding_x(8)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .border(1, t.colors.border_emphasized)
                .background(t.colors.accent_muted)
                .color(t.colors.text_primary)
                .font_size(14)
        }
    })
}
