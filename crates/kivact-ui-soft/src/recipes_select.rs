//! Role: Style the standard select trigger, popup, and option rows for Soft.
use crate::recipes_field::{frame, height, text};
use crate::tokens::Tokens;
use kivact::interaction::{ListItemState, PopupState};
use kivact::motion::Transition;
use kivact::style::{Align, Direction, Justify, Style, Theme, TransitionProp, pct, style};
use kivact::ui::{ControlSize, Look, SelectRecipes, SelectSheetOption, SelectTriggerState};

pub(super) fn select(r: SelectRecipes) -> SelectRecipes {
    r.trigger(|_| trigger)
        .surface(|_| surface)
        .option(|_| option)
        .check(|_| check)
        .indicator(|_| indicator)
        .description(|_| {
            |theme, look| {
                let t = theme.get::<Tokens>();
                let mut role = t.typography.supporting;
                if look.variant::<SelectSheetOption>().is_some() {
                    role.size_px = 14.0;
                }
                text(style().single_line(), role, t.colors.text_secondary)
            }
        })
}

fn trigger(theme: &Theme, s: &Look<SelectTriggerState>) -> Style {
    let t = theme.get::<Tokens>();
    let role = t.typography.body;
    let h = height(&t, s.size);
    let block = (h - role.line_height_px) / 2.0 - t.sizes.border_width;
    let color = if s.placeholder {
        t.colors.text_secondary
    } else {
        t.colors.text_primary
    };
    text(frame(&t, &s.input, false), role, color)
        .width(pct(100))
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .min_height(h)
        .padding_x(t.spacing.s3)
        .padding_y(block)
        .gap(t.spacing.s2)
}

fn surface(theme: &Theme, _: &Look<PopupState>) -> Style {
    const MAX_LIST_HEIGHT: f32 = 300.0;
    const ENTER_SCALE: f32 = 0.95;
    let t = theme.get::<Tokens>();
    let base = style()
        .max_height(MAX_LIST_HEIGHT)
        .padding_x(t.spacing.s1 + t.sizes.border_width)
        .padding_y(t.spacing.s1)
        .background(t.colors.bg_popover)
        .radius(t.radius.container)
        .enter(
            style()
                .opacity(0.0)
                .translate_y(-t.spacing.s2)
                .scale(ENTER_SCALE),
        )
        .transition(
            TransitionProp::Paint,
            Transition::tween(t.motion.fast_max as f64, t.motion.ease_entrance),
        );
    // The popup keeps the standard below-anchor placement and key behavior (C-35).
    t.shadows.low.iter().copied().fold(base, Style::shadow)
}

fn option(theme: &Theme, s: &Look<ListItemState>) -> Style {
    let t = theme.get::<Tokens>();
    let mut role = t.typography.body;
    if s.selected {
        role.weight = t.typography.weight_medium;
    }
    let mut base = style();
    let padding_y = if let Some(SelectSheetOption(size)) = s.variant::<SelectSheetOption>() {
        role.size_px = 16.0;
        role.line_height_px = 24.0;
        let height = match size {
            ControlSize::ExtraSmall | ControlSize::Small => 32.0,
            ControlSize::Medium => 36.0,
            ControlSize::Large => 40.0,
        };
        base = base.min_height(height);
        (height - role.line_height_px) / 2.0
    } else {
        t.spacing.s1_5
    };
    let base = text(base, role, t.colors.text_primary)
        .width(pct(100))
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .padding_x(t.spacing.s2)
        .padding_y(padding_y)
        .gap(t.spacing.s2)
        .radius(t.radius.element);
    if s.active && !s.input.disabled {
        base.background(t.colors.overlay_hover)
    } else {
        base
    }
}

fn check(theme: &Theme, s: &Look<ListItemState>) -> Style {
    let t = theme.get::<Tokens>();
    let size = if s.selected { t.sizes.icon_sm } else { 0.0 };
    let color = if s.input.disabled {
        t.colors.icon_disabled
    } else {
        t.colors.accent
    };
    let base = style().size(size, size).shrink(0).color(color);
    if s.selected {
        base
    } else {
        base.position(kivact::style::Position::Absolute)
    }
}

fn indicator(theme: &Theme, s: &Look<PopupState>) -> Style {
    let t = theme.get::<Tokens>();
    style()
        .size(t.sizes.icon_sm, t.sizes.icon_sm)
        .shrink(0)
        .color(t.colors.icon_secondary)
        .rotate(if s.open { 180 } else { 0 })
        .transition(
            TransitionProp::Paint,
            Transition::tween(t.motion.fast as f64, t.motion.ease_standard).decorative(false),
        )
}
