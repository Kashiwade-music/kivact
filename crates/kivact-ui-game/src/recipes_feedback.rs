//! Role: Draw console empty panels, alerts and toasts through the standard recipes.
use crate::color::{NAVY, ON_DARK, TAPE_STRIPE, TOAST_TEXT, YELLOW};
use crate::fonts::DISPLAY_FAMILY_VAR;
use crate::fonts::MONO_FAMILY_VAR;
use crate::tone::ToneExt;
use kivact::draw::{Layer, Path, Stroke};
use kivact::geometry::{Point, Rect};
use kivact::interaction::{ToastState, Tone};
use kivact::style::color::{ACCENT, ACCENT_TEXT, SURFACE, TEXT_MUTED};
use kivact::style::{
    Align, BorderStyle, Color, Display, FlexDirection, Justify, Position, Style, Theme, Track,
    auto, pct, style,
};
use kivact::ui::AlertVariant;
use kivact::ui::theme::{
    AlertState, AlertTheme, EmptyStateState, EmptyStateTheme, Look, ToastTheme,
};

/// Install the empty state, alert, toast and indicator recipes.
pub(crate) fn install(theme: Theme) -> Theme {
    let theme = theme
        .with(
            EmptyStateTheme::default()
                .root(empty_root)
                .title(empty_title)
                .description(empty_description)
                .icon(empty_icon)
                .actions(empty_actions),
        )
        .with(
            AlertTheme::default()
                .root(alert_root)
                .title(alert_title)
                .description(alert_description)
                .icon(alert_icon)
                .actions(alert_actions)
                .content(alert_content)
                .dismiss(alert_dismiss)
                .toggle(alert_toggle),
        )
        .with(
            ToastTheme::default()
                .region(toast_region)
                .toast(toast_card)
                .title(toast_title)
                .body(toast_body)
                .icon(toast_icon)
                .close(toast_close),
        );
    crate::recipes_feedback_indicators::install(theme)
}

fn empty_root(_: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style()
        .flex_direction(FlexDirection::Column)
        .width(pct(100))
        .padding_y(28)
        .padding_x(16)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(8)
        .border(1.5, crate::color::LINE_STRONG)
        .border_style(BorderStyle::Dashed)
}

fn empty_title(t: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style()
        .font_family(t.get(&DISPLAY_FAMILY_VAR).as_str().unwrap_or_default())
        .font_size(20)
        .font_weight(700)
        .font_style(kivact::style::FontStyle::Italic)
}

fn empty_description(_: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style().font_size(12.5).max_width(300).color(TEXT_MUTED)
}

fn empty_icon(_: &Theme, _: &Look<EmptyStateState>, _base: Style) -> Style {
    style().size(64, 64).paint(|d, _| {
        let color = crate::color::TEXT_3;
        d.stroke_path(
            &Path::new().arc(Point::new(32., 32.), 28., 0., 359.9, false),
            &Stroke::new(2.).dash([10., 4., 3., 4.]),
            color,
        );
        d.stroke_rect(Rect::new(22., 26., 20., 16.), 2., color);
        d.polyline(
            [
                Point::new(22., 26.),
                Point::new(26., 21.),
                Point::new(38., 21.),
                Point::new(42., 26.),
            ],
            &Stroke::new(2.),
            color,
        );
        d.line(Point::new(28., 33.), Point::new(36., 33.), 2.5, ACCENT);
    })
}

fn empty_actions(_: &Theme, _: &Look<EmptyStateState>, base: Style) -> Style {
    base.align_items(Align::Center)
}

fn banner(s: &Look<AlertState>) -> bool {
    s.variant::<AlertVariant>() == Some(AlertVariant::Banner)
}

fn alert_root(t: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    let base = style().display(Display::Grid).border_radius(0);
    if banner(s) {
        base.width(pct(100))
            .height(44)
            .padding_right(14)
            .gap(14)
            .grid_template_columns((auto(), Track::fr(1), auto()))
            .align_items(Align::Center)
            .background(NAVY)
            .color(ACCENT_TEXT)
    } else {
        let base = if s.dismissible {
            base.grid_template_columns((18, Track::fr(1), 32))
        } else {
            base.grid_template_columns((18, Track::fr(1)))
        };
        base.padding_y(12)
            .padding_x(14)
            .column_gap(12)
            .row_gap(2)
            .align_items(Align::Start)
            .background(s.tone.weak(t))
            .border(1, s.tone.color(t).mix(SURFACE, 0.65, t))
            .border_left(4, s.tone.color(t))
    }
}

fn alert_title(t: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    if banner(s) {
        banner_tape(t, s)
            .grid_column(1)
            .grid_row(1)
            .font_family(t.get(&MONO_FAMILY_VAR).as_str().unwrap_or_default())
            .font_size(10)
            .font_weight(800)
            .text_wrap(false)
            .padding_x(18)
            .padding_y(14.25)
    } else {
        style()
            .grid_column(2)
            .grid_row(1)
            .font_size(13)
            .font_weight(700)
    }
}

fn alert_description(_: &Theme, _: &Look<AlertState>, _base: Style) -> Style {
    style()
        .grid_column(2)
        .grid_row(2)
        .font_size(12.5)
        .color(TEXT_MUTED)
}

fn alert_icon(t: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    style()
        .grid_column(1)
        .grid_row((1, 3))
        .size(18, 18)
        .color(s.tone.color(t))
}

fn alert_actions(_: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    if banner(s) {
        style()
            .grid_column(3)
            .grid_row(1)
            .align_items(Align::Center)
    } else {
        style().grid_column(2).grid_row(3)
    }
}

fn alert_content(_: &Theme, s: &Look<AlertState>, _base: Style) -> Style {
    if banner(s) {
        style()
            .grid_column(2)
            .grid_row(1)
            .font_size(12.5)
            .color(ACCENT_TEXT)
            .flex_grow(1.)
    } else {
        style().grid_column(2).grid_row(4)
    }
}

fn alert_dismiss(_: &Theme, _: &Look<AlertState>, _base: Style) -> Style {
    style()
        .grid_column(3)
        .grid_row((1, 3))
        .align_self(Align::Start)
        .size(32, 32)
        .padding(0)
        .background(Color::TRANSPARENT)
        .color(TEXT_MUTED)
        .border_width(0)
        .shadow(kivact::style::Shadow::new(0, 0, 0, Color::TRANSPARENT))
}

fn alert_toggle(_: &Theme, _: &Look<AlertState>, _base: Style) -> Style {
    style().grid_column(2).grid_row(4)
}

fn banner_tape(t: &Theme, s: &Look<AlertState>) -> Style {
    let stripe = TAPE_STRIPE;
    let color = if s.tone == Tone::Warning {
        YELLOW
    } else {
        s.tone.color(t)
    };
    style()
        .grid_column(1)
        .grid_row(1)
        .height(pct(100))
        .padding_x(12)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(NAVY)
        .paint(move |d, s| {
            d.layer(Layer::new().clip_rect(s.to_rect()), |d| {
                d.fill_rect(s.to_rect(), color);
                let mut x = -44.;
                while x < s.width + 44. {
                    d.fill_path(
                        &Path::new()
                            .move_to(Point::new(x, s.height))
                            .line_to(Point::new(x + s.height, 0.))
                            .line_to(Point::new(x + s.height + 8., 0.))
                            .line_to(Point::new(x + 8., s.height))
                            .close(),
                        stripe,
                    );
                    x += 16.;
                }
                d.fill_rect(
                    Rect::new(12., (s.height - 23.5) / 2., s.width - 24., 23.5),
                    color,
                );
            })
        })
}

fn toast_region(_: &Theme, _: &Look<kivact::custom::PartState>, _base: Style) -> Style {
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
fn toast_card(t: &Theme, state: &Look<ToastState>, _base: Style) -> Style {
    use kivact::motion::{Easing, Transition};
    use kivact::style::TransitionProp;
    style()
        .enter(style().translate_x(40))
        .exit(style().translate_x(40).opacity(0.))
        .transition(TransitionProp::All, Transition::tween(260, Easing::EaseOut))
        .flex_direction(FlexDirection::Row)
        .width(360)
        .padding_left(14)
        .padding_right(12)
        .padding_top(12)
        .padding_bottom(14)
        .gap(12)
        .align_items(Align::Start)
        .background(NAVY)
        .color(ON_DARK)
        .border_left(4, state.tone.color(t))
}

/// The diamond of the tone, in the tone's color.
fn toast_icon(t: &Theme, state: &Look<ToastState>, _base: Style) -> Style {
    style()
        .size(18, 18)
        .flex_shrink(0.)
        .margin_top(1)
        .color(state.tone.color(t))
}

fn toast_title(_: &Theme, _: &Look<ToastState>, _base: Style) -> Style {
    style().font_weight(700).font_size(13).color(ON_DARK)
}

fn toast_body(_: &Theme, _: &Look<ToastState>, _base: Style) -> Style {
    style().font_size(12).color(TOAST_TEXT)
}

/// The close button around the kit's `x`.
fn toast_close(_: &Theme, _: &Look<ToastState>, _base: Style) -> Style {
    style()
        .size(22, 22)
        .flex_shrink(0.)
        .padding(3)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .color(TOAST_TEXT)
        .hover(style().color(ON_DARK))
}
