//! Role: Basic adornment, frame and borderless-control variants for input groups.
use crate::Tokens;
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{
    ButtonState, InputGroupMemberKind, InputGroupMemberState, InputGroupRecipes, InputGroupState,
    Look,
};
/// Draw a standard text input or text area without its own box inside a group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputGroupControl;
impl kivact::ui::Variant for InputGroupControl {}
/// Visual size of a canonical input-group button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InputGroupButtonSize {
    #[default]
    Xs,
    Sm,
    IconXs,
    IconSm,
}
impl kivact::ui::Variant for InputGroupButtonSize {}
kivact::state::constant_value!(InputGroupButtonSize);
/// Reactive visual measurements of a canonical button in an input group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InputGroupButtonLook {
    size: Value<InputGroupButtonSize>,
}
impl Default for InputGroupButtonLook {
    fn default() -> Self {
        Self {
            size: InputGroupButtonSize::Xs.into_value(),
        }
    }
}
impl kivact::ui::Variant for InputGroupButtonLook {}
impl InputGroupButtonLook {
    pub fn size(mut self, size: impl IntoValue<InputGroupButtonSize>) -> Self {
        self.size = size.into_value();
        self
    }
}
fn button_size(s: &Look<ButtonState>) -> Option<InputGroupButtonSize> {
    s.variant::<InputGroupButtonLook>()
        .map(|look| look.size.get())
        .or_else(|| s.variant::<InputGroupButtonSize>().copied())
}
pub(crate) fn button(s: &Look<ButtonState>) -> Option<Style> {
    button_size(s).map(|size| match size {
        InputGroupButtonSize::Xs => style().height(24).padding_x(6).gap(4),
        InputGroupButtonSize::Sm => style().height(32).padding_x(10).gap(4),
        InputGroupButtonSize::IconXs => style().size(24, 24).padding(0),
        InputGroupButtonSize::IconSm => style().size(28, 28).padding(0),
    })
}
pub(crate) fn button_icon(s: &Look<ButtonState>) -> Option<f64> {
    button_size(s).map(|size| match size {
        InputGroupButtonSize::Xs | InputGroupButtonSize::IconXs => 14.0,
        _ => 16.0,
    })
}
pub(crate) fn control(base: Style, multiline: bool) -> Style {
    let group = kivact::ui::use_input_group_state().map(|state| state.get());
    let (above, below) = group.map_or((false, false), |s| (s.has_block_start, s.has_block_end));
    let base = base
        .grow(1)
        .min_width(0)
        .border(0, Color::TRANSPARENT)
        .ring_none()
        .background(Color::TRANSPARENT)
        .disabled(style().opacity(1).background(Color::TRANSPARENT))
        .focus(style().border_color(Color::TRANSPARENT).ring_none())
        .focus_visible(style().border_color(Color::TRANSPARENT).ring_none());
    if multiline {
        base.padding_y(8)
    } else {
        let base = base.height(if above || below { 32 } else { 30 });
        let base = if below { base.padding_top(12) } else { base };
        if above { base.padding_bottom(12) } else { base }
    }
}
pub(crate) fn recipes(r: InputGroupRecipes) -> InputGroupRecipes {
    r.root(|_| root)
        .label(|_| label)
        .members(|_| members)
        .member(|_| member)
        .addon(|_| addon)
        .block_start(|_| block_start)
        .block_end(|_| block_end)
}
fn root(t: &Theme, s: &Look<InputGroupState>) -> Style {
    let p = t.ext::<Tokens>();
    let dark = p.is_dark();
    let base = style()
        .direction(Direction::Column)
        .width(pct(100))
        .min_width(0)
        .min_height(32)
        .shrink(0)
        .radius(0)
        .background(crate::theme_access::field_fill(&p))
        .border(1, p.input);
    let base = if s.input.disabled {
        base.merge(&crate::theme_access::disabled())
            .background(p.input.with_alpha(if dark { 0.8 } else { 0.5 }))
    } else {
        base
    };
    if s.invalid {
        base.border_color(p.destructive)
            .ring(1, p.destructive.with_alpha(if dark { 0.4 } else { 0.2 }))
    } else if s.focused {
        base.merge(&crate::theme_access::focus_ring(&p))
    } else {
        base
    }
}
fn label(_: &Theme, _: &Look<InputGroupState>) -> Style {
    style()
        .position(Position::Absolute)
        .size(1, 1)
        .font_size(1)
        .line_height(kivact::style::LineHeight::px(1))
        .single_line()
        .clip(true)
        .opacity(0)
}
fn members(_: &Theme, s: &Look<InputGroupState>) -> Style {
    style()
        .direction(if s.direction == kivact::view::ReadingDirection::Rtl {
            Direction::RowReverse
        } else {
            Direction::Row
        })
        .align_items(Align::Center)
        .width(pct(100))
        .min_width(0)
}
fn member(t: &Theme, s: &Look<InputGroupMemberState>) -> Style {
    if s.kind == InputGroupMemberKind::Control {
        style()
            .direction(Direction::Row)
            .grow(1)
            .min_width(0)
            .margin_left(if s.group.has_start { -4 } else { 0 })
            .margin_right(if s.group.has_end { -4 } else { 0 })
    } else {
        let base = style()
            .direction(Direction::Row)
            .align_items(Align::Center)
            .justify_content(Justify::Center)
            .gap(8)
            .shrink(0)
            .font_size(12)
            .weight(500)
            .color(t.ext::<Tokens>().muted_foreground);
        let start = s.kind == InputGroupMemberKind::Start;
        if start != (s.group.direction == kivact::view::ReadingDirection::Rtl) {
            base.padding_left(8)
        } else {
            base.padding_right(8)
        }
    }
}
fn addon(t: &Theme, _: &Look<InputGroupState>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8)
        .font_size(12)
        .color(t.ext::<Tokens>().muted_foreground)
}
fn block(t: &Theme, top: bool) -> Style {
    let base = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Start)
        .gap(8)
        .width(pct(100))
        .padding_x(10)
        .font_size(12)
        .weight(500)
        .color(t.ext::<Tokens>().muted_foreground);
    if top {
        base.padding_top(8).padding_bottom(6)
    } else {
        base.padding_top(6).padding_bottom(8)
    }
}
fn block_start(t: &Theme, _: &Look<InputGroupState>) -> Style {
    block(t, true)
}
fn block_end(t: &Theme, _: &Look<InputGroupState>) -> Style {
    block(t, false)
}
