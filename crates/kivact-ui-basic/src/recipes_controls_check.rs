//! Role: Lyra recipes of the checkbox and switch: a label beside a drawn box or track.
//!
//! The drawn part is local paint, which a state style cannot carry, so the
//! recipes read the state and draw the part with their own edge and fill.

use crate::color::{BACKGROUND, FOREGROUND, INPUT, PRIMARY, PRIMARY_FOREGROUND, RING};
use crate::recipe_parts::{CHECK, MINUS, field_fill, invalid_edge, stroke_glyph};
use kivact::custom::PartState;
use kivact::draw::{Draw, Path, Stroke};
use kivact::geometry::{Point, Rect, Size};
use kivact::style::{Align, Color, FlexDirection, Style, Theme, ThemeMode, style};
use kivact::ui::ControlSize;
use kivact::ui::theme::{CheckableState, CheckboxTheme, Look, SwitchTheme};
use kivact::view::CheckedState;

/// The side of the check box and the gap before its label.
const CHECK_SIDE: f64 = 16.0;
const LABEL_GAP: f64 = 8.0;
/// The switch track (`h-[18.4px] w-[32px]`, `sm` and extra small: `h-[14px] w-[24px]`)
/// and its thumb (`size-4`, `sm`: `size-3`).
const TRACK: (f64, f64) = (32.0, 18.4);
const THUMB: f64 = 16.0;
const TRACK_SM: (f64, f64) = (24.0, 14.0);
const THUMB_SM: f64 = 12.0;
/// Room left of the drawn part so its 1 px focus ring is not cut.
const LEAD: f64 = 1.0;

/// The track and thumb of a switch of `size` (`sm` for Small).
fn switch_metrics(size: ControlSize) -> ((f64, f64), f64) {
    match size {
        ControlSize::ExtraSmall | ControlSize::Small => (TRACK_SM, THUMB_SM),
        ControlSize::Medium | ControlSize::Large => (TRACK, THUMB),
    }
}

/// A label row with the room its drawn part needs on the left. The drawn
/// box or track shows the focus ring itself, so the row draws none.
fn toggle_row(drawn: f64) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .min_height(TRACK.1.ceil())
        .padding_left(LEAD + drawn + LABEL_GAP)
        .font_size(12.0)
        .line_height(1.0)
        .color(FOREGROUND)
        .focus_visible(style().outline_none())
}

/// The 1 px border inside `rect` and, when `halo` is given, the 1 px ring
/// outside it: stroked, so a transparent fill shows what is behind.
fn edge(d: &mut Draw, rect: Rect, radius: f64, border: Color, halo: Option<Color>) {
    let stroke = Stroke::new(1.0);
    let corners = |r: f64| [(r.max(0.0), r.max(0.0)); 4];
    if let Some(halo) = halo {
        let out = Rect::new(
            rect.x - 0.5,
            rect.y - 0.5,
            rect.width + 1.0,
            rect.height + 1.0,
        );
        let path = Path::new().rounded_rect(out, corners(radius + 0.5));
        d.stroke_path(&path, &stroke, halo);
    }
    let inner = Rect::new(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.width - 1.0,
        rect.height - 1.0,
    );
    let path = Path::new().rounded_rect(inner, corners(radius - 0.5));
    d.stroke_path(&path, &stroke, border);
}

/// The border and ring of a drawn toggle part in its state: invalid over
/// focus-visible over `rest`.
fn toggle_edge(
    input: &PartState,
    invalid: bool,
    dark: bool,
    rest: Color,
) -> (Color, Option<Color>) {
    if invalid {
        let (border, ring) = invalid_edge(dark);
        (border, Some(ring))
    } else if input.is_focus_visible() {
        (RING, Some(RING.with_alpha(0.5)))
    } else {
        (rest, None)
    }
}

fn draw_checkbox(state: &CheckableState, dark: bool, d: &mut Draw, size: Size) {
    let rect = Rect::new(
        LEAD,
        ((size.height - CHECK_SIDE) / 2.0).round(),
        CHECK_SIDE,
        CHECK_SIDE,
    );
    let on = state.checked() != CheckedState::Unchecked;
    // `data-checked:bg-primary`; unchecked only `dark:bg-input/30`.
    d.fill_rect(rect, if on { PRIMARY } else { field_fill(dark) });
    let (border, halo) = toggle_edge(&state.input, state.invalid, dark, INPUT);
    // `aria-invalid:aria-checked:border-primary`.
    let border = if on && !state.input.is_focus_visible() {
        PRIMARY
    } else {
        border
    };
    edge(d, rect, 0.0, border, halo);
    if on {
        let glyph = if state.is_mixed() { MINUS } else { CHECK };
        stroke_glyph(
            d,
            Point::new(rect.x + 1.0, rect.y + 1.0),
            14.0,
            glyph,
            PRIMARY_FOREGROUND,
        );
    }
}

fn draw_switch(state: &CheckableState, dark: bool, d: &mut Draw, size: Size) {
    let on = state.is_checked();
    let ((width, height), side) = switch_metrics(state.size);
    let track = Rect::new(LEAD, (size.height - height) / 2.0, width, height);
    let radius = height / 2.0;
    let fill = match (on, dark) {
        (true, _) => PRIMARY,
        (false, false) => INPUT,
        (false, true) => INPUT.with_alpha(0.8),
    };
    // The fill reaches under the transparent `border`.
    d.rounded_rect(track, radius, fill);
    let (border, halo) = toggle_edge(&state.input, state.invalid, dark, Color::TRANSPARENT);
    if halo.is_some() {
        edge(d, track, radius, border, halo);
    }
    // `translate-x-[calc(100%-2px)]` inside the 1 px transparent border.
    let x = track.x + 1.0 + if on { side - 2.0 } else { 0.0 };
    let thumb = match (on, dark) {
        (_, false) => BACKGROUND,
        (true, true) => PRIMARY_FOREGROUND,
        (false, true) => FOREGROUND,
    };
    d.rounded_rect(
        Rect::new(x, track.y + (height - side) / 2.0, side, side),
        side / 2.0,
        thumb,
    );
}

/// The checkbox: a 16 px box and its label.
fn checkbox(t: &Theme, state: &Look<CheckableState>, _base: Style) -> Style {
    let (dark, state) = (t.mode() == ThemeMode::Dark, **state);
    toggle_row(CHECK_SIDE).paint(move |d, size| draw_checkbox(&state, dark, d, size))
}

/// The switch: a track with a thumb and its label.
fn switch(t: &Theme, state: &Look<CheckableState>, _base: Style) -> Style {
    let (dark, state) = (t.mode() == ThemeMode::Dark, **state);
    let (track, _) = switch_metrics(state.size);
    toggle_row(track.0).paint(move |d, size| draw_switch(&state, dark, d, size))
}

/// Installs the checkbox and the switch.
pub(crate) fn install(theme: Theme) -> Theme {
    theme
        .with(CheckboxTheme::default().root(checkbox))
        .with(SwitchTheme::default().root(switch))
}
