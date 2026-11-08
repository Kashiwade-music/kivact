//! Role: Draw Game selection, navigation and overlay control geometry.
use crate::choice::{Boxed, ChoiceCard, Navigation};
use crate::recipes_controls::{button_style, diamond};
use crate::{Tokens, choice_style};
use kivact::custom::PartState;
use kivact::draw::{LineCap, LineJoin, Path, Shadow, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::interaction::{
    ListItemState, MenuItemState, PopupState, TabListState, TabState, ToastState, Tone,
};
use kivact::style::{
    Align, Color, ColorToken, Cursor, Direction, Justify, Position, Style, Theme, pct, style,
};
use kivact::ui::{AxisState, ButtonVariant, CheckableState, ControlSize, DialogState, Look};
use kivact::view::Orientation;

pub(crate) fn select_trigger(t: &Theme, s: &Look<kivact::ui::SelectTriggerState>) -> Style {
    // The heights of the buttons of the same size.
    let h = t.ext::<Tokens>().control_height;
    trigger(t).height(match s.size {
        ControlSize::ExtraSmall => h - 12.,
        ControlSize::Small => h - 6.,
        ControlSize::Medium => h,
        ControlSize::Large => h + 8.,
    })
}

pub(crate) fn menu_trigger(t: &Theme, _: &Look<PopupState>) -> Style {
    trigger(t)
}

fn trigger(t: &Theme) -> Style {
    style()
        .direction(Direction::Row)
        .height(t.ext::<Tokens>().control_height)
        .padding_x(12)
        .gap(8)
        .align_items(Align::Center)
        .background(t.colors().surface)
        .border(1, t.ext::<Tokens>().line_strong)
        .focus(
            style()
                .border_color(t.colors().accent)
                .ring(3, t.ext::<Tokens>().primary_weak),
        )
}

pub(crate) fn select_surface(_: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .padding_y(4)
        .min_width(220)
        .max_height(248)
        .background(ColorToken::Surface)
        .border(1, ColorToken::Border)
}

pub(crate) fn select_option(_: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .direction(Direction::Row)
        .padding_left(12)
        .padding_right(26)
        .height(32)
        .align_items(Align::Center)
        .hover(style().background(ColorToken::SurfaceAlt))
        .selected(
            style()
                .background(ColorToken::Selection)
                .color(ColorToken::Accent),
        )
        .focus(
            style()
                .ring(2, ColorToken::Focus)
                .background(ColorToken::Accent)
                .color(ColorToken::AccentText),
        )
}

/// The `check` of the selected option, in the row's color.
pub(crate) fn select_check(_: &Theme, _: &Look<ListItemState>) -> Style {
    style()
        .position(Position::Absolute)
        .right(10)
        .top(pct(50) - 8)
        .size(16, 16)
}

/// The box of the trigger's `chevron-down`, turned over while open.
pub(crate) fn select_indicator(_: &Theme, state: &Look<PopupState>) -> Style {
    let s = style().size(16, 16).shrink(0).color(ColorToken::Text);
    if state.open { s.rotate(180) } else { s }
}

pub(crate) fn menu_surface(t: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .direction(Direction::Column)
        .min_width(220)
        .padding(4)
        .background(t.colors().surface)
        .border(1, t.ext::<Tokens>().line_strong)
        .shadow(Shadow::new(0, 14, 36, t.colors().shadow))
}

pub(crate) fn menu_item(t: &Theme, _: &Look<MenuItemState>) -> Style {
    let hot = style()
        .background(t.colors().accent)
        .color(t.colors().accent_text);
    style()
        .direction(Direction::Row)
        .height(32)
        .padding_x(10)
        .gap(10)
        .align_items(Align::Center)
        .color(t.colors().text)
        .hover(hot.clone())
        .focus(hot)
}

pub(crate) fn menu_shortcut(t: &Theme, _: &Look<MenuItemState>) -> Style {
    style()
        .font_family(t.ext::<Tokens>().mono_family.as_str())
        .font_size(10.5)
        .weight(600)
        .color(ColorToken::TextMuted)
}

pub(crate) fn menu_separator(_: &Theme, _: &Look<kivact::custom::PartState>) -> Style {
    style().height(1).margin_y(4).background(ColorToken::Border)
}

pub(crate) fn dialog_surface(t: &Theme, s: &Look<DialogState>) -> Style {
    use kivact::geometry::Edge;
    use kivact::interaction::DialogMode::*;
    use kivact::motion::{Easing, Transition};
    use kivact::style::TransitionProp;
    let ease = Easing::CubicBezier(0.2, 0.8, 0.2, 1.);
    match s.kind {
        // A drawer slides in from its edge.
        Drawer(side) => style()
            .width(420)
            .max_width(pct(100))
            .background(t.colors().surface)
            .border_left(1, t.ext::<Tokens>().line_strong)
            .enter(style().opacity(0.).translate(
                match side {
                    Edge::Left => -40.,
                    Edge::Right => 40.,
                    _ => 0.,
                },
                match side {
                    Edge::Top => -40.,
                    Edge::Bottom => 40.,
                    _ => 0.,
                },
            ))
            .transition(TransitionProp::All, Transition::tween(240, ease)),
        Popover => style()
            .width(280)
            .padding(14)
            .gap(8)
            .background(ColorToken::Surface)
            .border(1, ColorToken::Border),
        // A modal rises a little as it fades in.
        _ => style()
            .width(520)
            .max_width(pct(100) - 32)
            .background(t.colors().surface)
            .border(1, t.ext::<Tokens>().line_strong)
            .shadow(Shadow::new(0, 14, 36, t.colors().shadow))
            .enter(style().opacity(0.).scale(0.985).translate(0, 10))
            .transition(TransitionProp::All, Transition::tween(220, ease)),
    }
}

pub(crate) fn dialog_header(_: &Theme, s: &Look<DialogState>) -> Style {
    if s.kind == kivact::interaction::DialogMode::Popover {
        style().align_items(Align::Center)
    } else {
        // The title over the description, clear of the 32 px close button.
        style()
            .padding_left(20)
            .padding_right(56)
            .padding_top(14)
            .padding_bottom(12)
            .gap(8)
            .border_top(4, ColorToken::Accent)
            .border_bottom(1, ColorToken::Border)
    }
}

pub(crate) fn dialog_title(t: &Theme, s: &Look<DialogState>) -> Style {
    if s.kind == kivact::interaction::DialogMode::Popover {
        style().weight(700)
    } else {
        style()
            .font_family(t.ext::<Tokens>().display_family.as_str())
            .font_size(22)
            .weight(700)
            .italic(true)
            .line_height(1.)
    }
}

pub(crate) fn dialog_description(_: &Theme, _: &Look<DialogState>) -> Style {
    style().font_size(13).color(ColorToken::TextMuted)
}

pub(crate) fn dialog_body(_: &Theme, s: &Look<DialogState>) -> Style {
    if s.kind == kivact::interaction::DialogMode::Popover {
        style().gap(6)
    } else {
        style().padding_x(20).padding_y(18)
    }
}

pub(crate) fn dialog_footer(_: &Theme, _: &Look<DialogState>) -> Style {
    style()
        .shrink(0)
        .padding_x(18)
        .padding_y(14)
        .gap(10)
        .justify_content(Justify::End)
        .align_items(Align::Center)
        .background(ColorToken::SurfaceAlt)
        .border_top(1, ColorToken::Border)
}

pub(crate) fn dialog_scrim(_: &Theme, _: &Look<DialogState>) -> Style {
    style().background(ColorToken::Overlay).backdrop_blur(2)
}

/// The close button around the theme's `x` icon.
pub(crate) fn dialog_close(_: &Theme, _: &Look<DialogState>) -> Style {
    style()
        .size(32, 32)
        .position(Position::Absolute)
        .top(16)
        .right(12)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(ColorToken::TextMuted)
        .hover(
            style()
                .background(ColorToken::SurfaceAlt)
                .border(1, ColorToken::Border)
                .color(ColorToken::Text),
        )
}

pub(crate) fn tooltip_surface(t: &Theme, _: &Look<PopupState>) -> Style {
    style()
        .padding_x(9)
        .padding_y(5)
        .max_width(320)
        .background(t.ext::<Tokens>().navy)
        .color(t.ext::<Tokens>().on_dark)
        .border_bottom(2, t.ext::<Tokens>().cyan)
        .font_size(11)
}

pub(crate) fn tabs_list(_: &Theme, state: &Look<TabListState>) -> Style {
    if state.variant::<Boxed>().is_some() {
        return choice_style::boxed_list();
    }
    if state.variant::<Navigation>().is_some() {
        return choice_style::navigation_list();
    }
    style()
        .direction(Direction::Row)
        .width(pct(100))
        .padding(0)
        .background(kivact::style::Color::TRANSPARENT)
        .border(0, ColorToken::Border)
        .border_bottom(1, ColorToken::Border)
        .gap(8)
}

pub(crate) fn tab(_: &Theme, state: &Look<TabState>) -> Style {
    if state.variant::<Boxed>().is_some() {
        return choice_style::boxed_tab();
    }
    if state.variant::<Navigation>().is_some() {
        return choice_style::navigation_item();
    }
    style()
        .direction(Direction::Row)
        .gap(7)
        .height(38)
        .padding_x(16)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border_bottom(3, kivact::style::Color::TRANSPARENT)
        .color(ColorToken::TextMuted)
        .selected(
            style()
                .border_color(ColorToken::Accent)
                .color(ColorToken::Accent)
                .weight(700),
        )
        .hover(style().background(ColorToken::Selection))
}

pub(crate) fn radio_group(_: &Theme, orientation: &Look<Orientation>) -> Style {
    if orientation.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
    {
        return choice_style::segment_group();
    }
    if orientation.variant::<ChoiceCard>().is_some() {
        return choice_style::card_group();
    }
    let s = style().gap(10);
    match **orientation {
        Orientation::Vertical => s.direction(Direction::Column),
        _ => s.direction(Direction::Row).wrap(true),
    }
}

pub(crate) fn radio_item(t: &Theme, state: &Look<CheckableState>) -> Style {
    if state.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
    {
        // A small slanted secondary button, filled while chosen.
        return button_style(
            t,
            ButtonVariant::Secondary,
            None,
            ControlSize::Small,
            false,
            false,
        )
        .clip_path(
            kivact::draw::Shape::polygon()
                .point(8, 0)
                .point(pct(100), 0)
                .point(pct(100) - 8, pct(100))
                .point(0, pct(100)),
        )
        .selected(choice_style::segment_chosen());
    }
    if state.variant::<ChoiceCard>().is_some() {
        return choice_style::card_item(state.checked.is_checked());
    }
    style()
        .direction(Direction::Row)
        .gap(9)
        .align_items(Align::Center)
        .min_height(24)
        .color(ColorToken::Text)
}

/// The diamond around the theme's `dot`, which is a diamond too.
pub(crate) fn radio_indicator(_: &Theme, state: &Look<CheckableState>) -> Style {
    if state.variant::<kivact::ui::RadioGroupVariant>()
        == Some(&kivact::ui::RadioGroupVariant::Segmented)
        || state.variant::<ChoiceCard>().is_some()
    {
        return choice_style::hidden_indicator();
    }
    let checked = state.checked.is_checked();
    style()
        .size(16, 16)
        .shrink(0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(ColorToken::Accent)
        .paint(move |d, size| {
            let color = if checked {
                d.theme().colors().accent
            } else {
                d.theme().ext::<Tokens>().line_strong
            };
            d.stroke_path(
                &diamond(0., 0., size.width.min(size.height), 1.),
                &Stroke::new(1.5).cap(LineCap::Round).join(LineJoin::Round),
                color,
            );
        })
}

pub(crate) fn radio_description(t: &Theme, _: &Look<CheckableState>) -> Style {
    style().font_size(11.5).color(t.ext::<Tokens>().text_3)
}

/// A 4 px bar across the middle of a box, along its longer side, leaving
/// `inset` at both ends.
fn bar(size: kivact::geometry::Size, inset: f64) -> Rect {
    if size.width >= size.height {
        Rect::new(
            inset,
            (size.height - 4.) / 2.,
            (size.width - 2. * inset).max(0.),
            4.,
        )
    } else {
        Rect::new(
            (size.width - 4.) / 2.,
            inset,
            4.,
            (size.height - 2. * inset).max(0.),
        )
    }
}

/// A 28 px track holding the bar between the thumb's centres at the ends.
pub(crate) fn slider_track(_: &Theme, state: &Look<AxisState>) -> Style {
    let s = style().shrink(0).cursor(Cursor::Pointer).paint(|d, size| {
        d.fill_rect(bar(size, THUMB.0 / 2.), d.theme().ext::<Tokens>().surface_3);
    });
    match state.orientation {
        Orientation::Vertical => s.width(28).height(160),
        _ => s.height(28).width(pct(100)).min_width(80),
    }
}

pub(crate) fn slider_range(_: &Theme, _: &Look<AxisState>) -> Style {
    style().paint(|d, size| d.fill_rect(bar(size, 0.), ColorToken::Accent))
}

/// The console's parallelogram thumb on the surface, outlined in the accent.
pub(crate) fn slider_thumb(_: &Theme, state: &Look<AxisState>) -> Style {
    let vertical = state.orientation == Orientation::Vertical;
    let (along, across) = THUMB;
    let s = if vertical {
        style().size(across, along)
    } else {
        style().size(along, across)
    };
    s.paint(move |d, size| {
        let (w, h) = (size.width, size.height);
        let thumb = if vertical {
            Path::new()
                .move_to(Point::new(0., 4.))
                .line_to(Point::new(0., h))
                .line_to(Point::new(w, h - 4.))
                .line_to(Point::new(w, 0.))
                .close()
        } else {
            Path::new()
                .move_to(Point::new(4., 0.))
                .line_to(Point::new(w, 0.))
                .line_to(Point::new(w - 4., h))
                .line_to(Point::new(0., h))
                .close()
        };
        d.fill_path(&thumb.clone(), d.theme().colors().surface);
        d.stroke_path(
            &thumb,
            &Stroke::new(1.5).cap(LineCap::Round).join(LineJoin::Round),
            ColorToken::Accent,
        );
    })
}

/// The color of a toast's tone.
fn tone_color(t: &Theme, tone: Tone) -> Color {
    match tone {
        Tone::Neutral | Tone::Info => t.colors().accent,
        Tone::Success => t.colors().success,
        Tone::Warning => t.colors().warning,
        Tone::Danger => t.colors().danger,
        Tone::Accent => t.ext::<Tokens>().yellow,
    }
}

/// A stack at the window's bottom right, above the page.
pub(crate) fn toast_region(_: &Theme, _: &Look<PartState>) -> Style {
    style()
        .position(Position::Fixed)
        .z_index(100)
        .right(16)
        .bottom(16)
        .max_width(pct(100) - 16)
        .align_items(Align::End)
        .gap(8)
}

/// A navy card with its tone's edge and a diamond of its tone before the
/// text. It slides in from the right and back out when it is dismissed.
pub(crate) fn toast_card(t: &Theme, state: &Look<ToastState>) -> Style {
    use kivact::motion::{Easing, Transition};
    use kivact::style::TransitionProp;
    let tone = state.tone;
    style()
        .enter(style().translate_x(40))
        .exit(style().translate_x(40).opacity(0.))
        .transition(TransitionProp::All, Transition::tween(260, Easing::EaseOut))
        .direction(Direction::Row)
        .width(360)
        .padding_left(14)
        .padding_right(12)
        .padding_top(12)
        .padding_bottom(14)
        .gap(12)
        .align_items(Align::Start)
        .background(t.ext::<Tokens>().navy)
        .color(t.ext::<Tokens>().on_dark)
        .border_left(4, tone_color(t, tone))
}

/// The diamond of the tone, in the tone's color.
pub(crate) fn toast_icon(t: &Theme, state: &Look<ToastState>) -> Style {
    style()
        .size(18, 18)
        .shrink(0)
        .margin_top(1)
        .color(tone_color(t, state.tone))
}

pub(crate) fn toast_title(t: &Theme, _: &Look<ToastState>) -> Style {
    style()
        .weight(700)
        .font_size(13)
        .color(t.ext::<Tokens>().on_dark)
}

pub(crate) fn toast_body(t: &Theme, _: &Look<ToastState>) -> Style {
    style().font_size(12).color(t.ext::<Tokens>().toast_text)
}

/// The close button around the kit's `x`.
pub(crate) fn toast_close(t: &Theme, _: &Look<ToastState>) -> Style {
    style()
        .size(22, 22)
        .shrink(0)
        .padding(3)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(t.ext::<Tokens>().toast_text)
        .hover(style().color(t.ext::<Tokens>().on_dark))
}

/// A 5 px bar between the panes that lights up under the pointer and while
/// dragged.
pub(crate) fn splitter_handle(_: &Theme, state: &Look<AxisState>) -> Style {
    let s = style()
        .shrink(0)
        .align_self(Align::Stretch)
        .background(ColorToken::Border)
        .hover(style().background(ColorToken::Accent))
        .pressed(style().background(ColorToken::Accent));
    match state.orientation {
        Orientation::Vertical => s.height(5).cursor(Cursor::NsResize),
        _ => s.width(5).cursor(Cursor::EwResize),
    }
}

/// The slider thumb dimensions along and across its axis.
const THUMB: (f64, f64) = (14., 18.);
