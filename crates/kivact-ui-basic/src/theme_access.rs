//! Role: Shared Lyra style fragments and Lucide strokes for recipes and parts.

use crate::tokens::Tokens;
use kivact::draw::{Draw, LineCap, LineJoin, Path, Point, Shadow, Stroke};
use kivact::style::{Align, Color, ColorToken, Direction, Style, current_theme, style};

/// The tokens of the theme in effect; call it inside a style closure.
pub(crate) fn tokens() -> std::sync::Arc<Tokens> {
    current_theme().get::<Tokens>()
}

/// The theme's focus ring (`Theme::focus_ring`, 1 px, in `ColorToken::Focus`,
/// `--ring` at half opacity): what every focus stop draws by default while
/// the focus is visible. A style with a `focus` look of its own turns that
/// default off, so it adds this to its `focus_visible` look. Call it inside
/// a style closure or a recipe.
pub(crate) fn theme_ring() -> Style {
    style().ring(current_theme().focus_ring(), ColorToken::Focus)
}

/// `focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50`:
/// the theme's focus ring with the border in `--ring`, for a part with a
/// border (a part without one draws the theme's default ring, which is the
/// same).
pub(crate) fn focus_ring(p: &Tokens) -> Style {
    theme_ring().border_color(p.ring)
}

/// `focus-visible:border-ring focus-visible:ring-[3px] ring-ring/50`: the
/// wide ring of tabs, toggles and badges.
pub(crate) fn wide_focus_ring(p: &Tokens) -> Style {
    style().border_color(p.ring).ring(3, p.ring.with_alpha(0.5))
}

/// Apply [`focus_ring`] while the focus is visible (`focus-visible:`).
pub(crate) fn with_focus(s: Style, p: &Tokens) -> Style {
    s.focus_visible(focus_ring(p))
}

/// `disabled:opacity-50`: the theme's `disabled_opacity`, for a node that
/// looks disabled without being an interaction part (a label, a field's box
/// around a disabled input); a focus stop and a part's item get it from the
/// theme by default. Input is suppressed by the
/// disabled state itself. Call it inside a style closure or a recipe.
pub(crate) fn disabled() -> Style {
    style().opacity(current_theme().disabled_opacity())
}

/// `bg-accent text-accent-foreground`: a popup row pointed at, focused or
/// active.
pub(crate) fn lit_row(p: &Tokens) -> Style {
    style().background(p.accent).color(p.accent_foreground)
}

/// `bg-transparent dark:bg-input/30`: the fill of fields and outline controls.
pub(crate) fn field_fill(p: &Tokens) -> Color {
    if p.is_dark() {
        p.input.with_alpha(0.3)
    } else {
        Color::TRANSPARENT
    }
}

/// Tailwind `shadow-md`.
pub(crate) fn shadow_md(s: Style) -> Style {
    let c = Color::BLACK.with_alpha(0.1);
    s.shadow(Shadow::new(0, 4, 6, c).spread(-1))
        .shadow(Shadow::new(0, 2, 4, c).spread(-2))
}

/// Tailwind `shadow-lg`.
pub(crate) fn shadow_lg(s: Style) -> Style {
    let c = Color::BLACK.with_alpha(0.1);
    s.shadow(Shadow::new(0, 10, 15, c).spread(-3))
        .shadow(Shadow::new(0, 4, 6, c).spread(-4))
}

/// A floating surface: `bg-popover text-popover-foreground ring-1 ring-foreground/10`.
pub(crate) fn popover_surface(p: &Tokens) -> Style {
    style()
        .background(p.popover)
        .color(p.popover_foreground)
        .ring(1, p.foreground.with_alpha(0.1))
        .radius(0)
        .font_size(12)
}

/// A row of a popup list: `gap-2 py-2 px-2 text-xs`, `bg-accent
/// text-accent-foreground` while `lit` (pointed at, focused or active).
pub(crate) fn list_row(p: &Tokens, lit: bool) -> Style {
    let s = style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .gap(8)
        .min_height(32)
        .padding_y(8)
        .padding_x(8)
        .radius(0)
        .font_size(12)
        .text_wrap(false)
        .color(p.popover_foreground);
    if lit {
        s.background(p.accent).color(p.accent_foreground)
    } else {
        s
    }
}

/// A menu row's shortcut: `ml-auto text-xs tracking-widest`, muted unless
/// its row is `lit`.
pub(crate) fn menu_shortcut(p: &Tokens, lit: bool) -> Style {
    style()
        .margin_left(16)
        .font_size(12)
        .letter_spacing(1.2)
        .color(if lit {
            p.accent_foreground
        } else {
            p.muted_foreground
        })
}

/// A Lucide glyph as polylines in its 24-unit view box.
pub(crate) type Glyph = &'static [&'static [(f64, f64)]];

/// Lucide `check`.
pub(crate) const CHECK: Glyph = &[&[(20.0, 6.0), (9.0, 17.0), (4.0, 12.0)]];
/// Lucide `minus`.
pub(crate) const MINUS: Glyph = &[&[(5.0, 12.0), (19.0, 12.0)]];
/// Lucide `chevron-down`.
pub(crate) const CHEVRON_DOWN: Glyph = &[&[(6.0, 9.0), (12.0, 15.0), (18.0, 9.0)]];

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
