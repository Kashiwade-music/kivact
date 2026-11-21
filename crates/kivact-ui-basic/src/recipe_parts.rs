//! Role: The helpers the basic kit's recipes share: focus rings, popover and list surfaces, the button look, Lucide strokes and pressable parts.

use crate::color::{
    ACCENT, ACCENT_FOREGROUND, BACKGROUND, BORDER, DESTRUCTIVE, FOREGROUND, INPUT, MUTED,
    MUTED_FOREGROUND, POPOVER, POPOVER_FOREGROUND, PRIMARY, PRIMARY_FOREGROUND, RING, SECONDARY,
    SECONDARY_FOREGROUND,
};
use kivact::custom::{PartState, part};
use kivact::draw::{Draw, LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::prelude::Element;
use kivact::state::{Callback, Value};
use kivact::style::{
    Align, Color, FlexDirection, Justify, Shadow, Style, TextDecorationLine, Theme, ThemeMode,
    color, style,
};
use kivact::ui::ControlSize;
use kivact::view::{IntoView, Node, Role, node};

/// A Lucide glyph as polylines in its 24-unit view box.
pub(crate) type Glyph = &'static [&'static [(f64, f64)]];

/// Lucide `check`.
pub(crate) const CHECK: Glyph = &[&[(20.0, 6.0), (9.0, 17.0), (4.0, 12.0)]];
/// Lucide `minus`.
pub(crate) const MINUS: Glyph = &[&[(5.0, 12.0), (19.0, 12.0)]];
/// Lucide `chevron-down`.
pub(crate) const CHEVRON_DOWN: Glyph = &[&[(6.0, 9.0), (12.0, 15.0), (18.0, 9.0)]];

/// The monospace family of inline code and code blocks; the application registers the font.
pub(crate) const MONO: &str = "\"Geist Mono\", monospace";

/// The theme's focus ring: the focus outline of the theme's width in
/// `--ring` at half opacity, for a part without a border of its own.
pub(crate) fn theme_ring(t: &Theme) -> Style {
    style().outline(t.focus_ring(), color::FOCUS)
}

/// `focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50`:
/// the theme's focus ring with the border in `--ring`.
pub(crate) fn focus_ring(t: &Theme) -> Style {
    theme_ring(t).border_color(RING)
}

/// `focus-visible:border-ring focus-visible:ring-[3px] ring-ring/50`: the
/// wide ring of tabs, toggles and badges.
pub(crate) fn wide_focus_ring() -> Style {
    style()
        .border_color(RING)
        .outline(3.0, RING.with_alpha(0.5))
}

/// Apply [`focus_ring`] while the focus is visible (`focus-visible:`).
pub(crate) fn with_focus(s: Style, t: &Theme) -> Style {
    s.focus_visible(focus_ring(t))
}

/// `bg-transparent dark:bg-input/30`: the fill of fields and outline controls.
pub(crate) fn field_fill(dark: bool) -> Color {
    if dark {
        INPUT.with_alpha(0.3)
    } else {
        Color::TRANSPARENT
    }
}

/// The `aria-invalid` border and ring: `border-destructive ring-1
/// ring-destructive/20` (`/50` and `/40` in the dark).
pub(crate) fn invalid_edge(dark: bool) -> (Color, Color) {
    if dark {
        (DESTRUCTIVE.with_alpha(0.5), DESTRUCTIVE.with_alpha(0.4))
    } else {
        (DESTRUCTIVE, DESTRUCTIVE.with_alpha(0.2))
    }
}

/// Tailwind `shadow-md`.
pub(crate) fn shadow_md(s: Style) -> Style {
    s.shadow(Shadow::new(0.0, 4.0, 6.0, color::SHADOW).spread(-1.0))
        .shadow(Shadow::new(0.0, 2.0, 4.0, color::SHADOW).spread(-2.0))
}

/// Tailwind `shadow-lg`.
pub(crate) fn shadow_lg(s: Style) -> Style {
    s.shadow(Shadow::new(0.0, 10.0, 15.0, color::SHADOW).spread(-3.0))
        .shadow(Shadow::new(0.0, 4.0, 6.0, color::SHADOW).spread(-4.0))
}

/// A floating surface: `bg-popover text-popover-foreground ring-1 ring-foreground/10`.
pub(crate) fn popover_surface() -> Style {
    style()
        .background(POPOVER)
        .color(POPOVER_FOREGROUND)
        .outline(1.0, FOREGROUND.with_alpha(0.1))
        .border_radius(0.0)
        .font_size(12.0)
}

/// A row of a popup list: `gap-2 py-2 px-2 text-xs`, `bg-accent
/// text-accent-foreground` while `lit` (pointed at, focused or active).
pub(crate) fn list_row(lit: bool) -> Style {
    let s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .gap(8.0)
        .min_height(32.0)
        .padding_y(8.0)
        .padding_x(8.0)
        .border_radius(0.0)
        .font_size(12.0)
        .text_wrap(false)
        .color(POPOVER_FOREGROUND);
    if lit {
        s.background(ACCENT).color(ACCENT_FOREGROUND)
    } else {
        s
    }
}

/// `bg-accent text-accent-foreground`: a popup row pointed at, focused or
/// active.
pub(crate) fn lit_row() -> Style {
    style().background(ACCENT).color(ACCENT_FOREGROUND)
}

/// A menu row's shortcut: `ml-auto text-xs tracking-widest`, muted unless
/// its row is `lit`.
pub(crate) fn menu_shortcut(lit: bool) -> Style {
    style()
        .margin_left(16.0)
        .font_size(12.0)
        .letter_spacing(1.2)
        .color(if lit {
            ACCENT_FOREGROUND
        } else {
            MUTED_FOREGROUND
        })
}

/// Stroke `glyph` into the square at `origin` with side `side`, as Lucide
/// draws it (2 units wide, round caps and joins).
pub(crate) fn stroke_glyph(d: &mut Draw, origin: Point, side: f64, glyph: Glyph, color: Color) {
    let k = side / 24.0;
    let stroke = Stroke::new(2.0 * k)
        .cap(LineCap::Round)
        .join(LineJoin::Round);
    for line in glyph {
        let mut path = Path::new();
        for (i, (x, y)) in line.iter().enumerate() {
            let at = Point::new(origin.x + x * k, origin.y + y * k);
            path = if i == 0 {
                path.move_to(at)
            } else {
                path.line_to(at)
            };
        }
        d.stroke_path(&path, &stroke, color);
    }
}

/// Lucide `loader-circle` turned by `phase` (0 to 1) in the square at `origin`.
pub(crate) fn stroke_loader(d: &mut Draw, origin: Point, side: f64, phase: f64, color: Color) {
    let k = side / 24.0;
    let start = phase * 360.0;
    let path = Path::new().arc(
        Point::new(origin.x + 12.0 * k, origin.y + 12.0 * k),
        9.0 * k,
        start,
        start + 288.0,
        false,
    );
    d.stroke_path(
        &path,
        &Stroke::new(2.0 * k)
            .cap(LineCap::Round)
            .join(LineJoin::Round),
        color,
    );
}

/// The look of a button: one per core variant, the six of button.tsx.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ButtonLook {
    Default,
    Outline,
    Secondary,
    Ghost,
    Destructive,
    Link,
}

impl ButtonLook {
    pub(crate) fn of(variant: kivact::ui::ButtonVariant) -> Self {
        match variant {
            kivact::ui::ButtonVariant::Primary => ButtonLook::Default,
            kivact::ui::ButtonVariant::Outline => ButtonLook::Outline,
            kivact::ui::ButtonVariant::Secondary => ButtonLook::Secondary,
            kivact::ui::ButtonVariant::Ghost => ButtonLook::Ghost,
            kivact::ui::ButtonVariant::Danger => ButtonLook::Destructive,
            kivact::ui::ButtonVariant::Link => ButtonLook::Link,
        }
    }

    /// The text (and icon) color of the look.
    pub(crate) fn foreground(self) -> Color {
        match self {
            ButtonLook::Default => PRIMARY_FOREGROUND,
            ButtonLook::Outline | ButtonLook::Ghost => FOREGROUND,
            ButtonLook::Secondary => SECONDARY_FOREGROUND,
            ButtonLook::Destructive => DESTRUCTIVE,
            ButtonLook::Link => PRIMARY,
        }
    }
}

/// Height, horizontal padding, gap and icon side of a button size.
pub(crate) fn metrics(size: ControlSize) -> (f64, f64, f64, f64) {
    match size {
        ControlSize::ExtraSmall => (24.0, 8.0, 4.0, 12.0),
        ControlSize::Small => (28.0, 10.0, 4.0, 14.0),
        ControlSize::Medium => (32.0, 10.0, 6.0, 16.0),
        ControlSize::Large => (36.0, 10.0, 6.0, 16.0),
    }
}

/// [`trigger_style`] for a plain button (not a popup trigger).
pub(crate) fn button_style(
    t: &Theme,
    look: ButtonLook,
    size: ControlSize,
    input: &PartState,
) -> Style {
    trigger_style(t, look, size, input, None)
}

/// The complete button style of `look` at `size` for the actual input state.
/// `expanded` is `Some(open)` for a popup or disclosure trigger, which does not
/// move while pressed and, open, takes the muted fill of an outline or ghost
/// button (`aria-expanded:bg-muted`).
pub(crate) fn trigger_style(
    t: &Theme,
    look: ButtonLook,
    size: ControlSize,
    input: &PartState,
    expanded: Option<bool>,
) -> Style {
    let (height, pad, gap, _) = metrics(size);
    let dark = t.mode() == ThemeMode::Dark;
    let s = style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .flex_shrink(0.0)
        .gap(gap)
        .height(height)
        .padding_x(pad)
        .border_radius(0.0)
        .border(1.0, Color::TRANSPARENT)
        // `text-xs`: 12 px on a 16 px line, whatever line height the
        // button's parent sets (a card body's `text-xs/relaxed`).
        .font_size(12.0)
        .line_height(16.0 / 12.0)
        .font_weight(500)
        .text_wrap(false)
        .color(look.foreground());
    let s = if expanded.is_none() {
        s.active(style().translate_y(1.0))
    } else {
        s
    };
    let open = expanded == Some(true);
    let s = match look {
        ButtonLook::Default => s
            .background(PRIMARY)
            .hover(style().background(PRIMARY.with_alpha(0.8))),
        ButtonLook::Outline => {
            let (fill, hover) = if dark {
                (INPUT.with_alpha(0.3), INPUT.with_alpha(0.5))
            } else {
                (BACKGROUND, MUTED)
            };
            s.background(if open { MUTED } else { fill })
                .border_color(if dark { INPUT } else { BORDER })
                .hover(style().background(hover))
        }
        ButtonLook::Secondary => s
            .background(SECONDARY)
            .hover(style().background(SECONDARY.mix(FOREGROUND, 0.05, t))),
        ButtonLook::Ghost => s
            .background(if open { MUTED } else { Color::TRANSPARENT })
            .hover(style().background(if dark { MUTED.with_alpha(0.5) } else { MUTED })),
        ButtonLook::Destructive => {
            let (fill, hover) = if dark { (0.2, 0.3) } else { (0.1, 0.2) };
            s.background(DESTRUCTIVE.with_alpha(fill))
                .hover(style().background(DESTRUCTIVE.with_alpha(hover)))
        }
        ButtonLook::Link => {
            let s = s.background(Color::TRANSPARENT).text_underline_offset(4.0);
            if input.is_hovered() && !input.is_disabled() {
                s.text_decoration(TextDecorationLine::UNDERLINE)
            } else {
                s
            }
        }
    };
    if look == ButtonLook::Destructive {
        let ring = DESTRUCTIVE.with_alpha(if dark { 0.4 } else { 0.2 });
        s.focus_visible(
            style()
                .border_color(DESTRUCTIVE.with_alpha(0.4))
                .outline(1.0, ring),
        )
    } else {
        with_focus(s, t)
    }
}

/// The layer behind a modal: `fixed inset-0 bg-black/10 backdrop-blur-xs`.
pub(crate) fn scrim_style() -> Style {
    style()
        .position(kivact::style::Position::Fixed)
        .inset(0.0)
        .background(color::OVERLAY)
        .backdrop_blur(4.0)
}

/// A title: `text-sm font-medium` in the popover foreground.
pub(crate) fn title_style() -> Style {
    style()
        .font_size(14.0)
        .font_weight(500)
        .line_height(20.0 / 14.0)
        .color(POPOVER_FOREGROUND)
}

/// A description: `text-xs/relaxed text-muted-foreground`.
pub(crate) fn description_style() -> Style {
    style()
        .font_size(12.0)
        .line_height(1.625)
        .color(MUTED_FOREGROUND)
}

/// The ghost `icon-sm` close button's box, `top` and `right` px inside the
/// corner of its panel, for the input state `input`.
pub(crate) fn close_box(t: &Theme, input: &PartState, top: f64, right: f64) -> Style {
    button_style(t, ButtonLook::Ghost, ControlSize::Small, input)
        .position(kivact::style::Position::Absolute)
        .top(top)
        .right(right)
        .size(28.0, 28.0)
        .padding(0.0)
}

/// toggle.tsx: `inline-flex center gap-1 text-xs font-medium`, muted when on,
/// with the wide ring while the focus is visible.
pub(crate) fn toggle_style(
    outline: bool,
    size: ControlSize,
    pressed: bool,
    icon_start: bool,
    joined: bool,
) -> Style {
    let (height, pad_left) = match size {
        ControlSize::ExtraSmall | ControlSize::Small => (28.0, if icon_start { 6.0 } else { 10.0 }),
        ControlSize::Medium => (32.0, if icon_start { 8.0 } else { 10.0 }),
        ControlSize::Large => (36.0, if icon_start { 8.0 } else { 10.0 }),
    };
    // `group-data-[spacing=0]/toggle-group:px-2` and `has-data-[icon=inline-start]:pl-1.5`.
    let (pad_left, pad_right) = if joined {
        (if icon_start { 6.0 } else { 8.0 }, 8.0)
    } else {
        (pad_left, 10.0)
    };
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(4.0)
        .flex_shrink(0.0)
        .height(height)
        .min_width(height)
        .padding_left(pad_left)
        .padding_right(pad_right)
        .border_radius(0.0)
        .border(1.0, if outline { INPUT } else { Color::TRANSPARENT })
        .font_size(12.0)
        .font_weight(500)
        .text_wrap(false)
        .color(FOREGROUND)
        .background(if pressed { MUTED } else { Color::TRANSPARENT })
        .hover(style().background(MUTED))
        // The focused item of a group draws its ring over its neighbours.
        .focus_visible(wide_focus_ring().z_index(1))
}

/// Attach hover, focus and press tracking to `root`.
///
/// With `on_press` the node becomes a focusable `role` that reports
/// activation (click, Enter, Space, assistive technology); without it the
/// state still follows the pointer, but the node takes no focus.
#[track_caller]
pub(crate) fn pressable(
    root: Node,
    role: Role,
    on_press: Option<Callback<()>>,
) -> (Node, Value<PartState>) {
    let attachment = part(|input| input);
    let state = attachment.state();
    let root = root.with_node(|root| root.part(attachment));
    let root = match on_press {
        Some(callback) => root
            .with_node(|root| root.role(role))
            .focusable(true)
            .on_press(callback),
        None => root,
    };
    (root, state)
}

/// Show `view` (a 16 px icon) at `to` pixels: the layout box is `to` wide and the glyph is scaled.
#[track_caller]
pub(crate) fn sized_icon(view: impl IntoView, to: impl kivact::state::IntoValue<f64>) -> Node {
    let to = to.into_value();
    let from = 16.0;
    node()
        .style(move || {
            style()
                .size(from, from)
                .margin((to.get() - from) / 2.0)
                .flex_shrink(0.0)
                .scale(to.get() / from)
        })
        .child(view)
}

/// The popup list of select, menus, comboboxes and native selects:
/// [`popover_surface`] with `shadow-md` and no padding.
pub(crate) fn surface_style() -> Style {
    shadow_md(popover_surface()).flex_direction(FlexDirection::Column)
}

/// An option row of a list popup: [`list_row`] with `pr-8`, leaving room
/// for the `check` at the right, lit while active, pointed at or focused.
pub(crate) fn option_row(active: bool) -> Style {
    list_row(active)
        .padding_right(32.0)
        .hover(lit_row())
        .focus(lit_row())
}

/// [`option_row`] with the `check` painted at the right while selected,
/// for the lists this kit builds itself.
pub(crate) fn row_style(state: &kivact::interaction::ListItemState) -> Style {
    // The check is local paint, which a state style cannot recolor, so it
    // follows the row's state.
    let input = &state.input;
    let lit = (input.is_hovered() || input.is_focused() || state.active) && !input.is_disabled();
    let check = if lit {
        ACCENT_FOREGROUND
    } else {
        POPOVER_FOREGROUND
    };
    let selected = state.selected;
    option_row(state.active).paint(move |d, size| {
        if selected {
            let at = Point::new(size.width - 24.0, (size.height - 16.0) / 2.0);
            stroke_glyph(d, at, 16.0, CHECK, check);
        }
    })
}
