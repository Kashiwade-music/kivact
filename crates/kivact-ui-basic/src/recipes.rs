//! Role: Lyra recipes for the standard button, toggles and text fields.
//!
//! What [`theme`](crate::theme) installs so that `kivact::ui::button`,
//! `checkbox`, `switch`, `text_input` and `text_area` follow the shadcn/ui
//! Lyra sources (button.tsx, checkbox.tsx, switch.tsx, input.tsx and
//! textarea.tsx); the other standard components' tables are in
//! `recipes_popup`, `recipes_dialog` and `recipes_controls`.

use crate::button::{Look, metrics, trigger_style};
use crate::theme_access::{CHECK, MINUS, field_fill, stroke_glyph, stroke_loader};
use crate::tokens::Tokens;
use kivact::custom::PartState;
use kivact::draw::{CornerRadius, Draw, Path, Stroke};
use kivact::geometry::{Point, Rect, Size};
use kivact::style::{Align, Color, Direction, Justify, Position, Style, Theme, pct, style};
use kivact::ui::{
    ButtonState, ButtonVariant, CheckableState, ControlSize, TextInputState, UiRecipes,
};
use kivact::view::CheckedState;

/// The recipes of the kit's look.
///
/// The table is built once and shared, so two themes of the kit compare
/// their recipes by identity and installing the kit's theme again never
/// restyles.
pub(crate) fn recipes() -> UiRecipes {
    static RECIPES: std::sync::OnceLock<UiRecipes> = std::sync::OnceLock::new();
    RECIPES.get_or_init(build).clone()
}

fn build() -> UiRecipes {
    UiRecipes::default()
        .field(crate::recipes_field::field)
        .button(|_| button)
        .button_content(|_| button_content)
        .button_busy(|_| button_busy)
        .button_icon(|_| button_icon)
        .toggle(|_| crate::recipes_controls::toggle)
        .checkbox(|_| checkbox)
        .switch(|_| switch)
        .text_input(|_| text_input)
        .text_area(|_| text_area)
        .select(crate::recipes_popup::select)
        .menu(crate::recipes_popup::menu)
        .dialog(crate::recipes_dialog::dialog)
        .tooltip(|tooltip| {
            tooltip
                .surface(|_| crate::recipes_popup::tooltip)
                .arrow(|_| crate::recipes_popup::tooltip_arrow)
        })
        .tabs(crate::recipes_popup::tabs)
        .radio_group(crate::recipes_controls::radio_group)
        .slider(crate::recipes_controls::slider)
        .toast(crate::recipes_controls::toast)
        .splitter(crate::recipes_controls::splitter)
        .list(crate::recipes_collections::list)
        .data_table(crate::recipes_collections::data_table)
        .split(crate::recipes_controls::split)
        .progress(crate::recipes_controls::progress)
        .disclosure(crate::recipes_controls::disclosure)
        .command_palette(crate::recipes_command::command_palette)
        .combobox(crate::recipes_combobox::combobox)
        .toggle_group(crate::recipes_groups::toggle_group)
        .button_group(crate::recipes_groups::button_group)
        .skeleton(|_| skeleton)
        .spinner(|_| spinner)
        .kbd(|_| kbd)
        .badge(|_| crate::badge::recipe)
        .calendar(crate::recipes_calendar::calendar)
        .empty_state(crate::empty::recipes)
        .alert(crate::alert::recipes)
        .avatar(|_| crate::avatar::avatar)
        .avatar_image(|_| crate::avatar::image)
        .avatar_group(|_| crate::avatar::group)
        .avatar_group_member(|_| crate::avatar::member)
        .avatar_status(|_| crate::avatar::status)
        .table(crate::table::recipes)
        .item_list(crate::item::recipes)
        .card(crate::card::recipes)
        .hover_card(|parts| parts.surface(|_| hover_card_surface))
        .carousel(|parts| {
            parts
                .root(|_| carousel_root)
                .track(|_| carousel_track)
                .viewport(|_| carousel_viewport)
                .slide(|_| carousel_slide)
                .previous(|_| carousel_previous)
                .next(|_| carousel_next)
        })
}

fn button(t: &Theme, state: &kivact::ui::Look<ButtonState>) -> Style {
    let p = t.ext::<Tokens>();
    let look = Look::of(
        state
            .variant::<ButtonVariant>()
            .copied()
            .unwrap_or_default(),
    );
    let s = trigger_style(&p, look, state.size, &state.input, state.expanded);

    // `has-data-[icon=inline-start]:pl-2` (`pl-1.5` in `sm` and `xs`), and
    // `pr` likewise beside a trailing icon.
    let edge = match state.size {
        ControlSize::ExtraSmall | ControlSize::Small => 6.0,
        ControlSize::Medium | ControlSize::Large => 8.0,
    };
    let s = if state.leading_icon {
        s.padding_left(edge)
    } else {
        s
    };
    let s = if state.trailing_icon {
        s.padding_right(edge)
    } else {
        s
    };
    let s = if state.busy {
        // Room for the leading spinner that `button_busy` places absolutely.
        let (_, pad, gap, icon) = metrics(state.size);
        s.padding_left(pad + icon + gap)
    } else {
        s
    };
    // button.tsx `size="icon"`: a square, as wide as the button is high.
    if state.icon_only {
        s.aspect_ratio(1.0).padding_x(0)
    } else {
        s
    }
}

fn button_content(_: &Theme, state: &kivact::ui::Look<ButtonState>) -> Style {
    let (_, _, gap, _) = metrics(state.size);
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(gap)
}

/// The box of a button's icons: `size-4`, `size-3.5` in a small button.
fn button_icon(_: &Theme, state: &kivact::ui::Look<ButtonState>) -> Style {
    let (_, _, _, side) = metrics(state.size);
    style()
        .size(side, side)
        .shrink(0)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
}

fn button_busy(t: &Theme, state: &kivact::ui::Look<ButtonState>) -> Style {
    let (_, pad, _, side) = metrics(state.size);
    let color = Look::of(
        state
            .variant::<ButtonVariant>()
            .copied()
            .unwrap_or_default(),
    )
    .foreground(&t.ext::<Tokens>());
    // The button turns the indicator while busy.
    style()
        .position(Position::Absolute)
        .left(pad)
        .top(pct(50) - side / 2.0)
        .size(side, side)
        .paint(move |d, size| stroke_loader(d, Point::new(0.0, 0.0), size.width, 0.0, color))
}

/// The side of the check box and the gap before its label.
const CHECK_SIDE: f64 = 16.0;
const LABEL_GAP: f64 = 8.0;
/// The switch track (`h-[18.4px] w-[32px]`, `sm` and extra small: `h-[14px] w-[24px]`)
/// and its thumb (`size-4`, `sm`: `size-3`).
const TRACK: (f64, f64) = (32.0, 18.4);
const THUMB: f64 = 16.0;
const TRACK_SM: (f64, f64) = (24.0, 14.0);
const THUMB_SM: f64 = 12.0;

/// The track and thumb of a switch of `size` (`sm` for Small).
fn switch_metrics(size: ControlSize) -> ((f64, f64), f64) {
    match size {
        ControlSize::ExtraSmall | ControlSize::Small => (TRACK_SM, THUMB_SM),
        ControlSize::Medium | ControlSize::Large => (TRACK, THUMB),
    }
}
/// Room left of the drawn part so its 1 px focus ring is not cut.
const LEAD: f64 = 1.0;

/// A label row with the room its drawn part needs on the left. The drawn
/// box or track shows the focus ring itself, so the row draws none.
///
/// The box is local paint of the row, which a state style cannot carry, so
/// its border and ring follow the row's state in the paint.
fn toggle_row(p: &Tokens, drawn: f64) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .min_height(TRACK.1.ceil())
        .padding_left(LEAD + drawn + LABEL_GAP)
        .font_size(12)
        .line_height(1.0)
        .color(p.foreground)
        .focus_visible(style().ring_none())
}

fn checkbox(t: &Theme, state: &kivact::ui::Look<CheckableState>) -> Style {
    let (p, state) = (t.ext::<Tokens>(), **state);
    toggle_row(&p, CHECK_SIDE).paint(move |d, size| draw_checkbox(&p, &state, d, size))
}

fn switch(t: &Theme, state: &kivact::ui::Look<CheckableState>) -> Style {
    let (p, state) = (t.ext::<Tokens>(), **state);
    let (track, _) = switch_metrics(state.size);
    toggle_row(&p, track.0).paint(move |d, size| draw_switch(&p, &state, d, size))
}

/// The 1 px border inside `rect` and, when `halo` is given, the 1 px ring
/// outside it: stroked, so a transparent fill shows what is behind.
fn edge(d: &mut Draw, rect: Rect, radius: f64, border: Color, halo: Option<Color>) {
    let stroke = Stroke::new(1.0);
    let corners = |r: f64| [CornerRadius::from(r.max(0.0)); 4];
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

/// The `aria-invalid` border and ring: `border-destructive ring-1
/// ring-destructive/20` (`/50` and `/40` in the dark).
fn invalid_edge(p: &Tokens) -> (Color, Color) {
    if p.is_dark() {
        (p.destructive.with_alpha(0.5), p.destructive.with_alpha(0.4))
    } else {
        (p.destructive, p.destructive.with_alpha(0.2))
    }
}

/// The border and ring of a drawn toggle part in its state: invalid over
/// focus-visible over `rest`.
fn toggle_edge(
    p: &Tokens,
    input: &PartState,
    invalid: bool,
    rest: Color,
) -> (Color, Option<Color>) {
    if invalid {
        let (border, ring) = invalid_edge(p);
        (border, Some(ring))
    } else if input.focus_visible {
        (p.ring, Some(p.ring.with_alpha(0.5)))
    } else {
        (rest, None)
    }
}

fn draw_checkbox(p: &Tokens, state: &CheckableState, d: &mut Draw, size: Size) {
    let rect = Rect::new(
        LEAD,
        ((size.height - CHECK_SIDE) / 2.0).round(),
        CHECK_SIDE,
        CHECK_SIDE,
    );
    let on = state.checked != CheckedState::Unchecked;
    // `data-checked:bg-primary`; unchecked only `dark:bg-input/30`.
    d.fill_rect(rect, if on { p.primary } else { field_fill(p) });
    let (border, halo) = toggle_edge(p, &state.input, state.invalid, p.input);
    // `aria-invalid:aria-checked:border-primary`.
    let border = if on && !state.input.focus_visible {
        p.primary
    } else {
        border
    };
    edge(d, rect, 0.0, border, halo);
    if on {
        let glyph = if state.checked == CheckedState::Indeterminate {
            MINUS
        } else {
            CHECK
        };
        stroke_glyph(
            d,
            Point::new(rect.x + 1.0, rect.y + 1.0),
            14.0,
            glyph,
            p.primary_foreground,
        );
    }
}

fn draw_switch(p: &Tokens, state: &CheckableState, d: &mut Draw, size: Size) {
    let on = state.checked.is_checked();
    let ((width, height), side) = switch_metrics(state.size);
    let track = Rect::new(LEAD, (size.height - height) / 2.0, width, height);
    let radius = height / 2.0;
    let fill = match (on, p.is_dark()) {
        (true, _) => p.primary,
        (false, false) => p.input,
        (false, true) => p.input.with_alpha(0.8),
    };
    // The fill reaches under the transparent `border`.
    d.rounded_rect(track, radius, fill);
    let (border, halo) = toggle_edge(p, &state.input, state.invalid, Color::TRANSPARENT);
    if halo.is_some() {
        edge(d, track, radius, border, halo);
    }
    // `translate-x-[calc(100%-2px)]` inside the 1 px transparent border.
    let x = track.x + 1.0 + if on { side - 2.0 } else { 0.0 };
    let thumb = match (on, p.is_dark()) {
        (_, false) => p.background,
        (true, true) => p.primary_foreground,
        (false, true) => p.foreground,
    };
    d.rounded_rect(
        Rect::new(x, track.y + (height - side) / 2.0, side, side),
        side / 2.0,
        thumb,
    );
}

/// The field look of input.tsx and textarea.tsx.
fn field(t: &Theme, state: &kivact::ui::Look<TextInputState>) -> Style {
    let p = t.ext::<Tokens>();
    let s = style()
        .min_width(0)
        .padding_x(10)
        .radius(0)
        .border(1, p.input)
        .background(field_fill(&p))
        .color(p.foreground)
        .font_size(12)
        .placeholder_color(p.muted_foreground)
        .caret_color(p.foreground)
        .selection_color(p.primary.with_alpha(0.2))
        .disabled(style().background(p.input.with_alpha(if p.is_dark() { 0.8 } else { 0.5 })));
    if state.invalid {
        let (border, ring) = invalid_edge(&p);
        s.border_color(border).ring(1, ring)
    } else {
        // A field shows its focus however it got it.
        s.focus(crate::theme_access::focus_ring(&p))
    }
}

fn text_input(t: &Theme, state: &kivact::ui::Look<TextInputState>) -> Style {
    field(t, state)
        .height(match state.size {
            ControlSize::ExtraSmall => 24,
            ControlSize::Small => 28,
            ControlSize::Medium => 32,
            ControlSize::Large => 36,
        })
        .padding_y(4)
}

/// `min-h-16 field-sizing-content`: at least 64 px, growing with the lines
/// of the value (a native text area does not wrap, and it is never lower
/// than three lines).
fn text_area(t: &Theme, state: &kivact::ui::Look<TextInputState>) -> Style {
    field(t, state)
        .min_height(64)
        .padding_y(8)
        .line_height(16.0 / 12.0)
}

fn hover_card_surface(t: &Theme, _: &kivact::ui::Look<kivact::ui::HoverCardSurfaceState>) -> Style {
    crate::theme_access::shadow_md(crate::theme_access::popover_surface(&t.ext::<Tokens>()))
        .direction(Direction::Column)
        .line_height(1.625)
        .width(256)
        .max_width(pct(100))
        .padding(10)
}

fn carousel_root(_: &Theme, _: &kivact::ui::Look<kivact::ui::CarouselSurfaceState>) -> Style {
    style()
        .position(Position::Relative)
        .direction(Direction::Column)
        .focus(crate::theme_access::theme_ring())
}
fn carousel_track(_: &Theme, _: &kivact::ui::Look<kivact::ui::CarouselSurfaceState>) -> Style {
    style().gap(16).transition(
        kivact::style::TransitionProp::Paint,
        kivact::motion::Transition::tween(
            300,
            kivact::motion::Easing::CubicBezier(0.4, 0.0, 0.2, 1.0),
        ),
    )
}
fn carousel_viewport(_: &Theme, _: &kivact::ui::Look<kivact::ui::CarouselSurfaceState>) -> Style {
    style().direction(Direction::Column)
}
fn carousel_slide(_: &Theme, _: &kivact::ui::Look<kivact::ui::CarouselSurfaceState>) -> Style {
    style()
}
fn carousel_previous(t: &Theme, s: &kivact::ui::Look<kivact::ui::CarouselSurfaceState>) -> Style {
    carousel_control(t, s, true)
}
fn carousel_next(t: &Theme, s: &kivact::ui::Look<kivact::ui::CarouselSurfaceState>) -> Style {
    carousel_control(t, s, false)
}
fn carousel_control(
    t: &Theme,
    s: &kivact::ui::Look<kivact::ui::CarouselSurfaceState>,
    previous: bool,
) -> Style {
    let p = t.ext::<Tokens>();
    let base = style()
        .position(Position::Absolute)
        .size(28, 28)
        .min_width(28)
        .padding(0)
        .background(p.background)
        .color(p.foreground)
        .border(1, p.input)
        .hover(style().background(p.muted).color(p.foreground));
    if s.orientation == kivact::view::Orientation::Vertical {
        if previous {
            base.top(-48).left(pct(50) - 14)
        } else {
            base.bottom(-48).left(pct(50) - 14)
        }
    } else if previous != (s.direction == kivact::view::ReadingDirection::Rtl) {
        base.left(-48).top(pct(50) - 14)
    } else {
        base.right(-48).top(pct(50) - 14)
    }
}

fn skeleton(t: &Theme, s: &kivact::ui::Look<kivact::ui::SkeletonState>) -> Style {
    let base = style()
        .grow(1)
        .min_height(16)
        .width(pct(100))
        .background(t.ext::<Tokens>().muted)
        .opacity(s.pulse);
    match s.shape {
        kivact::ui::SkeletonShape::Circle => base.size(36, 36).radius(999),
        _ => base.radius(0),
    }
}
fn spinner(t: &Theme, s: &kivact::ui::Look<kivact::ui::SpinnerState>) -> Style {
    let side = match s.size {
        ControlSize::ExtraSmall => 12.0,
        ControlSize::Small => 14.0,
        ControlSize::Medium => 16.0,
        ControlSize::Large => 20.0,
    };
    let turn = s.turn / 360.0;
    let color = t.ext::<Tokens>().foreground;
    style().size(side, side).shrink(0).paint(move |d, _| {
        crate::theme_access::stroke_loader(d, Point::new(0.0, 0.0), side, turn, color)
    })
}
fn kbd(t: &Theme, _: &kivact::ui::Look<()>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(4)
        .height(20)
        .min_width(20)
        .padding_x(4)
        .radius(0)
        .background(p.muted)
        .color(p.muted_foreground)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
}
