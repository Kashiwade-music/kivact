//! Role: Style standard action and input groups with console metrics.
use crate::Tokens;
use kivact::style::{Align, ColorToken, Position, auto, pct, style};
use kivact::ui::{ButtonGroupRecipes, InputGroupRecipes, ToggleGroupRecipes, ToolbarRecipes};

/// Console input adornments, either adjacent cells or inset glyphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameInputGroupVariant {
    Inline,
    Adorned,
}
impl kivact::ui::Variant for GameInputGroupVariant {}

pub(crate) fn toolbar(r: ToolbarRecipes) -> ToolbarRecipes {
    r.root(|base| move |t, s| base(t, s).width(auto()).border(1, ColorToken::Border))
}
pub(crate) fn button_group(r: ButtonGroupRecipes) -> ButtonGroupRecipes {
    r.root(|base| move |t, s| base(t, s).align_items(Align::Center))
}
pub(crate) fn toggle_group(r: ToggleGroupRecipes) -> ToggleGroupRecipes {
    r.root(|base| move |t, s| base(t, s).align_items(Align::Center))
        .item(|base| {
            move |t, s| {
                base(t, s)
                    .radius(0)
                    .border_color(t.ext::<Tokens>().line_strong)
            }
        })
}
pub(crate) fn input_group(r: InputGroupRecipes) -> InputGroupRecipes {
    r.root(|base| {
        move |t, s| {
            let result = base(t, s).width(pct(100));
            if s.variant::<GameInputGroupVariant>().is_some() {
                result.gap(0)
            } else {
                result
            }
        }
    })
    .label(|base| {
        move |t, s| {
            if s.variant::<GameInputGroupVariant>().is_some() {
                style().size(0, 0).clip(true)
            } else {
                base(t, s)
            }
        }
    })
    .members(|base| move |t, s| base(t, s).width(pct(100)).align_items(Align::Center))
    .member(|base| {
        move |t, s| {
            let p = t.ext::<Tokens>();
            if s.variant::<GameInputGroupVariant>() == Some(&GameInputGroupVariant::Adorned) {
                if s.first || s.last {
                    let result = style()
                        .position(Position::Absolute)
                        .align_self(Align::Center)
                        .color(ColorToken::TextMuted);
                    if s.first {
                        result.left(10)
                    } else {
                        result.right(8)
                    }
                } else {
                    style().grow(1).min_width(0)
                }
            } else if s.variant::<GameInputGroupVariant>() == Some(&GameInputGroupVariant::Inline) {
                if s.first || s.last {
                    style()
                        .height(p.control_height)
                        .padding_x(10)
                        .align_items(Align::Center)
                        .background(p.surface_3)
                        .border(1, p.line_strong)
                } else {
                    style().grow(1).min_width(0)
                }
            } else {
                base(t, s)
            }
        }
    })
    .addon(|base| {
        move |t, s| {
            base(t, s)
                .padding_x(10)
                .background(t.ext::<Tokens>().surface_3)
                .border_color(t.ext::<Tokens>().line_strong)
                .font_family("monospace")
        }
    })
}
