//! Role: Install Astryx Neutral's recipes of the feedback group: the toast, the skeleton, the spinner and the key caps.
use crate::color as c;
use crate::metrics as m;
use crate::motion::FAST;
use crate::radius as r;
use crate::recipe_parts::{body_text, ring, shadowed, tween};
use crate::shadows;
use crate::space as s;
use crate::type_scale as t;
use crate::variants::{SkeletonRadius, SpinnerShade, SpinnerVariant};
use kivact::custom::PartState;
use kivact::draw::{LineCap, Path, Stroke};
use kivact::geometry::Point;
use kivact::interaction::{ToastState, Tone};
use kivact::style::{
    Align, Color, FlexDirection, Justify, Position, Style, Theme, ThemeMode, pct, style,
};
use kivact::ui::theme::{
    KbdKeyState, KbdTheme, Look, SkeletonState, SkeletonTheme, SpinnerState, SpinnerTheme,
    ToastGroupState, ToastTheme,
};
use kivact::ui::{ControlSize, SkeletonShape};

/// The width of the toast column, in logical px.
const TOAST_WIDTH: f64 = 400.0;

/// The toast region: the toasts stack at the window's bottom right, above the page.
fn region(theme: &Theme, _look: &Look<PartState>, _base: Style) -> Style {
    let gutter = theme.get(&s::SPACE_LAYER_VIEWPORT_GUTTER_VAR);
    style()
        .position(Position::Fixed)
        .bottom(gutter)
        .right(gutter)
        .width(TOAST_WIDTH)
        .max_width(pct(100) - gutter * 2.0)
        .flex_direction(FlexDirection::Column)
}

/// One live region's column of toasts.
fn group(_theme: &Theme, _look: &Look<ToastGroupState>, _base: Style) -> Style {
    style().flex_direction(FlexDirection::Column)
}

/// The text color of a toast: dark text on a light surface, light text on a dark one.
fn foreground(theme: &Theme, look: &Look<ToastState>) -> Color {
    if theme.mode() == ThemeMode::Dark && look.tone != Tone::Danger {
        c::ON_LIGHT
    } else {
        c::ON_DARK
    }
}

/// A toast on the inverted surface, its edge in the tone's danger color or the inverted fill.
fn card(theme: &Theme, look: &Look<ToastState>, _base: Style) -> Style {
    let dark = theme.mode() == ThemeMode::Dark;
    let fill = if look.tone == Tone::Danger {
        c::BG_ERROR_INVERTED
    } else {
        c::BG_INVERTED
    };
    let hidden = style().opacity(0.0).translate_y(s::SPACE_S2);
    let base = shadowed(body_text(style()), shadows::med(dark))
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Start)
        .gap(s::SPACE_S3)
        .padding(s::SPACE_DIALOG_PADDING)
        .border_radius(r::RADIUS_CONTAINER)
        .background(fill)
        .color(foreground(theme, look));
    tween(base, FAST).enter(hidden.clone()).exit(hidden)
}

/// The text of a toast's title and body.
fn text(theme: &Theme, look: &Look<ToastState>, _base: Style) -> Style {
    body_text(style()).color(foreground(theme, look))
}

/// The close button: the inverted surface's hover, pressed and focus colors, around the `x` icon.
fn close(theme: &Theme, look: &Look<ToastState>, _base: Style) -> Style {
    let inverted = if theme.mode() == ThemeMode::Dark && look.tone != Tone::Danger {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    };
    let media = crate::theme::theme(inverted);
    let overhang = (m::CONTROL_SM - t::BODY.line_height_px) / 2.0;
    style()
        .size(m::CONTROL_SM, m::CONTROL_SM)
        .flex_shrink(0)
        .margin_top(-overhang)
        .margin_right(-overhang)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .border_radius(r::RADIUS_ELEMENT)
        .color(foreground(theme, look))
        .hover(style().background(media.get(&c::OVERLAY_HOVER_VAR)))
        .active(style().background(media.get(&c::OVERLAY_PRESSED_VAR)))
        .focus_visible(ring(media.get(&c::FOCUS_RING_VAR)))
}

/// The toast icon is hidden: a toast draws its own icon view when it has one.
fn icon(_theme: &Theme, _look: &Look<ToastState>, _base: Style) -> Style {
    style().size(0.0, 0.0)
}

/// The skeleton: a muted block with the corner profile of its shape or variant, pulsing.
fn skeleton(_theme: &Theme, look: &Look<SkeletonState>, _base: Style) -> Style {
    let radius = match look.variant::<SkeletonRadius>() {
        Some(SkeletonRadius::None | SkeletonRadius::R0) => r::RADIUS_NONE,
        Some(SkeletonRadius::R1) => r::RADIUS_INNER,
        Some(SkeletonRadius::R2) => r::RADIUS_ELEMENT,
        Some(SkeletonRadius::Rounded) => r::RADIUS_FULL,
        Some(SkeletonRadius::R3 | SkeletonRadius::R4) => r::RADIUS_CONTAINER,
        None => match look.shape {
            SkeletonShape::Circle => r::RADIUS_FULL,
            SkeletonShape::Rounded | SkeletonShape::Rect => r::RADIUS_CONTAINER,
            SkeletonShape::Text => r::RADIUS_INNER,
        },
    };
    let base = style()
        .size(pct(100), pct(100))
        .border_radius(radius)
        .background(c::SKELETON)
        .opacity((1.75 - 1.5 * look.pulse).min(0.925));
    match look.shape {
        SkeletonShape::Circle => base.size(36.0, 36.0),
        SkeletonShape::Text => base.height(t::BODY.line_height_px),
        _ => base,
    }
}

/// The spinner: a track ring with an arc in the shade's ink, turning with the spinner's angle.
fn spinner(_theme: &Theme, look: &Look<SpinnerState>, _base: Style) -> Style {
    let (diameter, width) = match look.size {
        ControlSize::ExtraSmall => (10.0, 2.0),
        ControlSize::Small => (14.0, 3.0),
        ControlSize::Medium => (18.0, 3.0),
        ControlSize::Large => (28.0, 4.0),
    };
    let variant = look.variant::<SpinnerVariant>().unwrap_or_default();
    let (ink, track) = match variant.shade {
        SpinnerShade::Default => (c::ACCENT, c::TRACK),
        SpinnerShade::Subtle => (c::TEXT_SECONDARY, c::TRACK),
        SpinnerShade::OnMedia => (c::ON_DARK, c::ON_DARK.with_alpha(77.0 / 255.0)),
        SpinnerShade::Inherit => {
            let ink = variant.ink.unwrap_or(c::TEXT_PRIMARY);
            (ink, ink.with_alpha(0.3))
        }
    };
    let start = -90.0 + look.turn;
    style()
        .size(diameter + width * 2.0, diameter + width * 2.0)
        .flex_shrink(0)
        .paint(move |draw, size| {
            let center = Point::new(size.width / 2.0, size.height / 2.0);
            let stroke = Stroke::new(width).cap(LineCap::Round);
            draw.stroke_path(
                &Path::new().arc(center, diameter / 2.0, 0.0, 360.0, false),
                &stroke,
                track,
            );
            draw.stroke_path(
                &Path::new().arc(center, diameter / 2.0, start, start + 135.0, false),
                &stroke,
                ink,
            );
        })
}

/// The row of key caps: the caps sit in a centered row.
fn kbd_root(_theme: &Theme, _look: &Look<()>, _base: Style) -> Style {
    style().align_items(Align::Center).gap(4.0).flex_shrink(0)
}

/// One key cap: a small raised box with the shortcut's text.
fn key(_theme: &Theme, _look: &Look<KbdKeyState>, _base: Style) -> Style {
    style()
        .height(20.0)
        .min_width(20.0)
        .padding_x(s::SPACE_S1)
        .border_radius(r::RADIUS_INNER)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .background(c::NEUTRAL)
        .border_bottom(2.0, c::BORDER_EMPHASIZED)
        .color(c::TEXT_PRIMARY)
        .font_size(t::SIZE_SM)
        .line_height(20.0 / t::SIZE_SM)
        .font_weight(t::WEIGHT_MEDIUM)
        .text_wrap(false)
}

/// Install the kit's recipes of the feedback group.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(
            ToastTheme::default()
                .region(region)
                .group(group)
                .toast(card)
                .title(text)
                .body(text)
                .close(close)
                .icon(icon),
        )
        .with(SkeletonTheme::default().root(skeleton))
        .with(SpinnerTheme::default().root(spinner))
        .with(KbdTheme::default().root(kbd_root).key(key))
}
