//! Role: Style the native editors, field text, and composite input frames for Soft.
use crate::tokens::{TextStyle, Tokens};
use kivact::motion::Transition;
use kivact::primitive::PartState;
use kivact::style::{Align, Color, Direction, Style, Theme, TransitionProp, pct, style};
use kivact::ui::{
    ControlSize, FieldRecipes, Look, NumberInputRecipes, TagInputRecipes, TextInputState,
};

pub(super) fn height(tokens: &Tokens, size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall | ControlSize::Small => tokens.sizes.control_sm,
        ControlSize::Medium => tokens.sizes.control_md,
        ControlSize::Large => tokens.sizes.control_lg,
    }
}

pub(super) fn text(s: Style, role: TextStyle, color: Color) -> Style {
    s.font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .weight(role.weight)
        .color(color)
}

pub(super) fn frame(tokens: &Tokens, input: &PartState, invalid: bool) -> Style {
    let c = &tokens.colors;
    let hover = if invalid {
        tokens.shadows.inset_error
    } else {
        tokens.shadows.input_hover_ring
    };
    let focus = if invalid {
        style()
    } else {
        style()
            .border_color(c.accent)
            .shadow(tokens.shadows.input_focus_ring)
    };
    style()
        .min_width(0)
        .background(c.bg_surface)
        .radius(tokens.radius.element)
        .border(
            tokens.sizes.border_width,
            if invalid {
                c.text_error
            } else {
                c.border_emphasized
            },
        )
        .hover(if input.focused {
            style()
        } else {
            style().shadow(hover)
        })
        .focus(focus)
        .transition(
            TransitionProp::Paint,
            Transition::tween(tokens.motion.fast as f64, tokens.motion.ease_standard),
        )
}

fn editor(tokens: &Tokens, base: Style) -> Style {
    text(base, tokens.typography.body, tokens.colors.text_primary)
        .placeholder_color(tokens.colors.text_secondary)
        .caret_color(tokens.colors.text_primary)
}

pub(super) fn text_input(theme: &Theme, s: &Look<TextInputState>) -> Style {
    let t = theme.get::<Tokens>();
    editor(&t, frame(&t, &s.input, s.invalid))
        .height(height(&t, s.size))
        .padding_x(t.spacing.s2)
        .padding_y(t.spacing.s1)
}

pub(super) fn text_area(theme: &Theme, s: &Look<TextInputState>) -> Style {
    const DEFAULT_ROWS: f64 = 3.0;
    let t = theme.get::<Tokens>();
    let block = if s.size == ControlSize::Large {
        t.spacing.s2
    } else {
        t.spacing.s1
    };
    editor(&t, frame(&t, &s.input, s.invalid))
        .width(pct(100))
        .min_height(
            DEFAULT_ROWS * t.typography.body.line_height_px + 2.0 * (block + t.sizes.border_width),
        )
        .padding_x(t.spacing.s2)
        .padding_y(block)
}

pub(super) fn field(r: FieldRecipes) -> FieldRecipes {
    r.root(|_| {
        |theme, _| {
            let t = theme.get::<Tokens>();
            style()
                .direction(Direction::Column)
                .min_width(0)
                .gap(t.spacing.s1)
        }
    })
    .label(|_| {
        |theme, _| {
            let t = theme.get::<Tokens>();
            text(style(), t.typography.label, t.colors.text_secondary)
        }
    })
    .hint(|_| {
        |theme, _| {
            let t = theme.get::<Tokens>();
            text(style(), t.typography.supporting, t.colors.text_secondary)
        }
    })
    .error(|_| {
        |theme, _| {
            let t = theme.get::<Tokens>();
            text(style(), t.typography.supporting, t.colors.text_error)
        }
    })
}

fn flat(t: &Tokens) -> Style {
    let clear_edge = style()
        .border_color(Color::TRANSPARENT)
        .shadow(kivact::draw::Shadow {
            color: Color::TRANSPARENT,
            ..t.shadows.input_focus_ring
        });
    editor(t, style())
        .background(Color::TRANSPARENT)
        .border(0, Color::TRANSPARENT)
        .padding(0)
        .hover(clear_edge.clone())
        .focus(clear_edge)
}

pub(super) fn number_input(r: NumberInputRecipes) -> NumberInputRecipes {
    // The standard spinbutton retains its two side buttons.
    r.root(|_| {
        |theme, state| {
            let t = theme.get::<Tokens>();
            frame(&t, state, false)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .height(t.sizes.control_md)
                .clip(true)
        }
    })
    .input(|_| {
        |theme, _| {
            let t = theme.get::<Tokens>();
            flat(&t)
                .height(t.sizes.control_md - 2.0 * t.sizes.border_width)
                .padding_x(t.spacing.s2)
                .grow(1)
        }
    })
}

pub(super) fn tag_input(r: TagInputRecipes) -> TagInputRecipes {
    r.root(|_| {
        |theme, state| {
            let t = theme.get::<Tokens>();
            let inset = t.radius.element - t.radius.inner - t.sizes.border_width;
            let base = frame(&t, state, false);
            let base = if state.focused {
                base.border_color(t.colors.accent)
                    .shadow(t.shadows.input_focus_ring)
            } else {
                base
            };
            base.align_items(Align::Center)
                .column_gap(t.spacing.s1)
                .row_gap(inset)
                .padding(inset)
                .min_height(t.sizes.control_md)
        }
    })
    .tag(|_| {
        |theme, _| {
            let t = theme.get::<Tokens>();
            let role = TextStyle {
                weight: t.typography.weight_medium,
                ..t.typography.supporting
            };
            text(style(), role, t.colors.text_primary)
                .direction(Direction::Row)
                .align_items(Align::Center)
                .gap(t.spacing.s1)
                .height(t.spacing.s6)
                .padding_left(t.spacing.s2)
                .padding_right(t.spacing.s1)
                .radius(t.radius.inner)
                .background(t.colors.neutral)
                .max_width(pct(100))
        }
    })
    .remove(|_| {
        |theme, _| {
            let t = theme.get::<Tokens>();
            let clear = style().background(Color::TRANSPARENT).scale(1.0);
            style()
                .size(t.sizes.icon_sm, t.sizes.icon_sm)
                .padding(0)
                .radius(t.radius.full)
                .color(t.colors.text_primary)
                .hover(clear.clone())
                .pressed(clear)
                .focus_visible(style().ring_with_offset(
                    t.sizes.focus_ring_width,
                    t.sizes.focus_ring_offset,
                    t.colors.focus_ring,
                ))
        }
    })
    .input(|_| {
        |theme, _| {
            const EMPTY_INPUT_MIN_WIDTH: f64 = 60.0;
            let t = theme.get::<Tokens>();
            flat(&t)
                .grow(1)
                .min_width(EMPTY_INPUT_MIN_WIDTH)
                .height(t.spacing.s6)
        }
    })
}
