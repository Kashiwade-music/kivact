//! Role: Default recipes for standard multi-part components.
use super::values::{
    ControlSize, DialogState, FieldState, ProgressState, SelectTriggerState, TooltipArrowState,
};
use kivact_core::custom::PartState;
use kivact_core::geometry::Rect;
use kivact_core::style::{
    Align, ColorToken, Direction, Justify, Position, Style, Theme, pct, style,
};
use kivact_core::view::Orientation;
use kivact_interaction::{
    DisclosureState, ListItemState, MenuItemState, PopupState, TabListState, TabState,
};
/// A select's trigger: a bordered field, its text muted while it shows
/// the placeholder.
pub(super) fn select_trigger(t: &Theme, state: &SelectTriggerState) -> Style {
    let s = trigger(t).height(super::look::height(t, state.size));
    if state.placeholder {
        s.color(ColorToken::TextMuted)
    } else {
        s
    }
}
/// A menu's own trigger, drawn like a select's.
pub(super) fn menu_trigger(t: &Theme, _: &PopupState) -> Style {
    trigger(t)
}
fn trigger(t: &Theme) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .height(t.control_height())
        .padding_x(t.space().md)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .background(ColorToken::Surface)
}
pub(super) fn surface(t: &Theme, _: &PopupState) -> Style {
    style()
        .padding(4)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
        .min_width(160)
        .max_height(320)
}
pub(super) fn option(t: &Theme, _: &ListItemState) -> Style {
    style()
        .padding_x(t.space().md)
        .height(t.control_height())
        .justify_content(kivact_core::style::Justify::Center)
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().md)
        .focus(style().background(ColorToken::Hover))
        .selected(style().background(ColorToken::Hover))
}
/// The box of the selected option's check, at the row's end.
pub(super) fn option_check(_: &Theme, _: &ListItemState) -> Style {
    mark().width(16)
}
/// The secondary line of an option or a menu item.
fn description(t: &Theme) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}
pub(super) fn option_description(t: &Theme, _: &ListItemState) -> Style {
    description(t)
}
/// The box of a mark icon, centred.
fn mark() -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(kivact_core::style::Justify::Center)
        .shrink(0)
}
pub(super) fn select_indicator(_: &Theme, _: &PopupState) -> Style {
    mark().color(ColorToken::TextMuted)
}
pub(super) fn menu_item(t: &Theme, state: &MenuItemState) -> Style {
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8)
        .height(t.control_height())
        .padding_x(t.space().md)
        .focus(style().background(ColorToken::Hover));
    if state.destructive {
        s.color(ColorToken::Danger)
    } else {
        s
    }
}
pub(super) fn separator(_: &Theme, _: &PartState) -> Style {
    style().height(1).margin_y(4).background(ColorToken::Border)
}
/// A heading: muted small text the keys skip.
pub(super) fn menu_label(t: &Theme, _: &PartState) -> Style {
    style()
        .padding_x(t.space().md)
        .padding_y(t.space().xs)
        .font_size(t.text().size_small)
        .weight(t.text().weight_strong)
        .color(ColorToken::TextMuted)
}
pub(super) fn shortcut(_: &Theme, _: &MenuItemState) -> Style {
    style().grow(1).color(ColorToken::TextMuted)
}
pub(super) fn menu_description(t: &Theme, _: &MenuItemState) -> Style {
    description(t)
}
/// The check or radio mark's box: 16 px wide even while empty, so labels
/// line up.
pub(super) fn menu_check(_: &Theme, _: &MenuItemState) -> Style {
    mark().width(16)
}
pub(super) fn menu_submenu(_: &Theme, _: &MenuItemState) -> Style {
    mark()
}
/// The room of an inset row: as wide as the check box.
pub(super) fn menu_inset(_: &Theme, _: &PartState) -> Style {
    style().width(16).shrink(0)
}

pub(super) fn tooltip(t: &Theme, _: &PopupState) -> Style {
    style()
        .padding_x(8)
        .padding_y(4)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
        .radius(t.radius().sm)
        .font_size(t.text().size_small)
}
pub(super) fn tabs_list(_: &Theme, s: &TabListState) -> Style {
    if s.orientation == Orientation::Vertical {
        style()
            .direction(Direction::Column)
            .gap(4)
            .border_right(1, ColorToken::Border)
    } else {
        style()
            .direction(Direction::Row)
            .gap(4)
            .border_bottom(1, ColorToken::Border)
    }
}
pub(super) fn no_tab_indicator(_: &Theme, _: &TabState) -> Style {
    style()
}

pub(super) fn tab(t: &Theme, s: &TabState) -> Style {
    let mark = if s.orientation == Orientation::Vertical {
        style().border_right(2, ColorToken::Accent)
    } else {
        style().border_bottom(2, ColorToken::Accent)
    };
    style()
        .padding_x(t.space().md)
        .height(t.control_height())
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(t.space().sm)
        .justify_content(kivact_core::style::Justify::Center)
        .selected(mark)
}
pub(super) fn panel(t: &Theme, _: &TabState) -> Style {
    style().padding(t.space().md)
}
pub(super) fn dialog_surface(t: &Theme, _: &DialogState) -> Style {
    style()
        .width(480)
        .max_width(pct(100))
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
        .radius(t.radius().md)
}
pub(super) fn dialog_title(t: &Theme, _: &DialogState) -> Style {
    style()
        .font_size(t.text().size + 4.0)
        .weight(t.text().weight_strong)
}
pub(super) fn dialog_description(_: &Theme, _: &DialogState) -> Style {
    style().color(ColorToken::TextMuted)
}
/// The title over the description, clear of the close button.
pub(super) fn dialog_header(t: &Theme, _: &DialogState) -> Style {
    style()
        .padding(t.space().lg)
        .padding_right(t.space().lg + 24.0 + t.space().sm)
        .gap(t.space().sm)
}
/// The media view's box, above the title.
pub(super) fn dialog_media(_: &Theme, _: &DialogState) -> Style {
    style().shrink(0)
}
pub(super) fn dialog_body(t: &Theme, _: &DialogState) -> Style {
    style().padding(t.space().lg)
}
pub(super) fn dialog_footer(t: &Theme, _: &DialogState) -> Style {
    style().padding(t.space().lg).gap(t.space().sm)
}
pub(super) fn dialog_scrim(_: &Theme, _: &DialogState) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(kivact_core::style::Color::BLACK.with_alpha(0.35))
}
/// The close button at the surface's top right, around its `x` icon.
pub(super) fn dialog_close(t: &Theme, _: &DialogState) -> Style {
    let inset = t.space().lg - 4.0;
    mark()
        .position(Position::Absolute)
        .top(inset)
        .right(inset)
        .size(24, 24)
        .radius(t.radius().sm)
        .color(ColorToken::TextMuted)
        .hover(style().background(ColorToken::Hover))
}
/// No arrow: an empty box out of the layout.
pub(super) fn no_arrow(_: &Theme, _: &TooltipArrowState) -> Style {
    style().position(Position::Absolute)
}
/// A field: its label, control, hint and error stacked.
pub(super) fn field(t: &Theme, _: &FieldState) -> Style {
    style().direction(Direction::Column).gap(t.space().xs)
}
/// A field's visible label, in the danger color while the field has an error.
pub(super) fn field_label(t: &Theme, state: &FieldState) -> Style {
    let s = style()
        .font_size(t.text().size_small)
        .weight(t.text().weight_strong);
    if state.invalid {
        s.color(ColorToken::Danger)
    } else {
        s
    }
}
/// A field's hint: muted small text under the control.
pub(super) fn field_hint(t: &Theme, _: &FieldState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::TextMuted)
}
/// A field's error message: small text in the danger color.
pub(super) fn field_error(t: &Theme, _: &FieldState) -> Style {
    style()
        .font_size(t.text().size_small)
        .color(ColorToken::Danger)
}

/// A disclosure: the trigger over the content.
pub(super) fn disclosure_root(_: &Theme, _: &DisclosureState) -> Style {
    style().direction(Direction::Column)
}
/// A disclosure's trigger: the title at the start and the indicator at the end.
pub(super) fn disclosure_trigger(t: &Theme, _: &DisclosureState) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::SpaceBetween)
        .gap(t.space().md)
        .padding_y(t.space().md)
        .font_size(t.text().size)
        .weight(t.text().weight_strong)
        .color(ColorToken::Text)
        .radius(t.radius().md)
        .hover(style().background(ColorToken::Hover))
        .pressed(style().background(ColorToken::Pressed))
}
/// The box of a disclosure's `chevron-down` icon, turned up while it is open.
pub(super) fn disclosure_indicator(_: &Theme, state: &DisclosureState) -> Style {
    style()
        .size(16, 16)
        .shrink(0)
        .color(ColorToken::TextMuted)
        .rotate(if state.open { 180 } else { 0 })
}
/// A disclosure's content under the trigger.
pub(super) fn disclosure_content(t: &Theme, _: &DisclosureState) -> Style {
    style()
        .enter_size(kivact_core::view::Orientation::Vertical)
        .exit_size(kivact_core::view::Orientation::Vertical)
        .direction(Direction::Column)
        .padding_bottom(t.space().md)
        .color(ColorToken::TextMuted)
}
/// The height of a progress bar's track.
fn progress_height(size: ControlSize) -> f64 {
    match size {
        ControlSize::ExtraSmall => 3.0,
        ControlSize::Small => 4.0,
        ControlSize::Medium => 6.0,
        ControlSize::Large => 10.0,
    }
}
/// A progress bar's track: a full-width rounded bar that clips its fill.
pub(super) fn progress_track(_: &Theme, state: &ProgressState) -> Style {
    let height = progress_height(state.size);
    style()
        .width(pct(100))
        .height(height)
        .min_width(0)
        .radius(height / 2.0)
        .background(ColorToken::Border)
        .clip(true)
}
/// A progress bar's fill, painted from the shown fraction (determinate) or
/// the moving segment (indeterminate).
pub(super) fn progress_fill(_: &Theme, state: &ProgressState) -> Style {
    let state = *state;
    style()
        .width(pct(100))
        .height(pct(100))
        .paint(move |d, size| {
            let (x, width) = if state.value.is_some() {
                (0.0, size.width * state.shown.get().clamp(0.0, 1.0))
            } else {
                let (start, length) = state.sweep();
                (size.width * start, size.width * length)
            };
            d.rounded_rect(
                Rect::new(x, 0.0, width, size.height),
                size.height / 2.0,
                ColorToken::Accent,
            );
        })
}

/// Pointer target for pulling a drawer toward its edge.
pub(super) fn drawer_handle(_: &Theme, state: &DialogState) -> Style {
    if matches!(
        state.kind,
        kivact_interaction::DialogMode::Drawer(
            kivact_core::geometry::Edge::Left | kivact_core::geometry::Edge::Right
        )
    ) {
        style().width(24).height(48).shrink(0)
    } else {
        style().height(24).width(48).shrink(0)
    }
}

pub(super) fn hover_card(t: &Theme, _: &crate::HoverCardSurfaceState) -> Style {
    tooltip(
        t,
        &PopupState {
            input: Default::default(),
            open: true,
        },
    )
}
