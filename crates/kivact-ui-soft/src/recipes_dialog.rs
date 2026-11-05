//! Role: Style the standard dialog kinds and share overlay paint helpers.
use crate::Tokens;
use kivact::geometry::Edge;
use kivact::interaction::DialogMode;
use kivact::motion::Transition;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::Style;
use kivact::style::Theme;
use kivact::style::TransitionProp;
use kivact::style::auto;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::DialogRecipes;
use kivact::ui::DialogState;
use kivact::ui::Look;

const DIALOG_WIDTH: f64 = 400.0;
const DIALOG_MAX_HEIGHT_PERCENT: f64 = 75.0;
const LAYER_ENTER_SCALE: f64 = 0.95;

pub(crate) fn dialog(r: DialogRecipes) -> DialogRecipes {
    r.surface(|_| surface)
        .scrim(|_| {
            |t: &Theme, s: &Look<DialogState>| {
                let p = t.ext::<Tokens>();
                let base = scrim(&p);
                let hidden = style().opacity(0.0);
                if matches!(s.kind, DialogMode::Drawer(_)) {
                    motion(base, &p, p.motion.medium)
                        .enter(hidden.clone())
                        .exit(hidden)
                } else {
                    base
                }
            }
        })
        .header(|_| {
            |t: &Theme, s: &Look<DialogState>| {
                let p = t.ext::<Tokens>();
                let base = style().direction(Direction::Column);
                if matches!(s.kind, DialogMode::Drawer(_)) {
                    base.padding(p.spacing.dialog_padding)
                } else {
                    base
                }
            }
        })
        .title(|_| {
            |t: &Theme, _: &Look<DialogState>| {
                let p = t.ext::<Tokens>();
                let text = p.typography.heading2;
                style()
                    .font_size(text.size_px)
                    .line_height(text.line_height_px / text.size_px)
                    .weight(text.weight)
                    .color(p.colors.text_primary)
            }
        })
        .description(|_| {
            |t: &Theme, s: &Look<DialogState>| {
                let p = t.ext::<Tokens>();
                let base = style().color(p.colors.text_secondary);
                if s.kind == DialogMode::AlertModal {
                    base
                } else {
                    base.font_size(p.typography.size_sm)
                }
            }
        })
        .body(|_| {
            |t: &Theme, s: &Look<DialogState>| {
                let base = style().direction(Direction::Column).min_height(0);
                if matches!(s.kind, DialogMode::Drawer(_)) {
                    base.padding_x(t.ext::<Tokens>().spacing.dialog_padding)
                } else {
                    base
                }
            }
        })
        .footer(|_| {
            |t: &Theme, s: &Look<DialogState>| {
                let p = t.ext::<Tokens>();
                let base = style()
                    .direction(Direction::Row)
                    .justify_content(Justify::End)
                    .gap(p.spacing.s2);
                if matches!(s.kind, DialogMode::Drawer(_)) {
                    base.margin_top(auto()).padding(p.spacing.dialog_padding)
                } else {
                    base
                }
            }
        })
        .close(|_| close)
}

fn surface(t: &Theme, s: &Look<DialogState>) -> Style {
    let p = t.ext::<Tokens>();
    let gutter = p.spacing.layer_viewport_gutter;
    let base = body_text(&p)
        .direction(Direction::Column)
        .radius(p.radius.container)
        .background(p.colors.bg_surface)
        .focus_visible(focus(&p));
    match s.kind {
        DialogMode::Popover => anchored(
            shadows(base.background(p.colors.bg_popover), &p.shadows.low)
                .padding(p.spacing.section_padding)
                .max_width(pct(100) - gutter * 2.0)
                .max_height(pct(100) - gutter * 2.0),
            &p,
            0.0,
            -p.spacing.s2,
        ),
        DialogMode::Drawer(side) => drawer(shadows(base, &p.shadows.high), &p, side),
        DialogMode::Modal | DialogMode::AlertModal => modal(
            shadows(base, &p.shadows.high)
                .width(DIALOG_WIDTH)
                .max_width(pct(100) - gutter * 2.0)
                .max_height(pct(DIALOG_MAX_HEIGHT_PERCENT))
                .padding(p.spacing.dialog_padding)
                .gap(p.spacing.dialog_padding),
            &p,
        ),
    }
}

fn drawer(base: Style, p: &Tokens, side: Edge) -> Style {
    let width = p.sizes.border_width;
    let color = p.colors.border;
    let base = base.radius(p.radius.none);
    let (base, hidden) = match side {
        Edge::Right => (
            base.width(DIALOG_WIDTH)
                .max_width(pct(100))
                .border_left(width, color),
            style().translate_x(pct(100)),
        ),
        Edge::Left => (
            base.width(DIALOG_WIDTH)
                .max_width(pct(100))
                .border_right(width, color),
            style().translate_x(pct(-100)),
        ),
        Edge::Top => (
            base.max_height(pct(100)).border_bottom(width, color),
            style().translate_y(pct(-100)),
        ),
        Edge::Bottom => (
            base.max_height(pct(100)).border_top(width, color),
            style().translate_y(pct(100)),
        ),
    };
    // The drawer slides out through the standard dialog exit lifecycle.
    motion(base, p, p.motion.medium)
        .enter(hidden.clone())
        .exit(hidden)
}

fn close(t: &Theme, _: &Look<DialogState>) -> Style {
    let p = t.ext::<Tokens>();
    style()
        .position(Position::Absolute)
        .top(p.spacing.s2)
        .right(p.spacing.s2)
        .size(p.sizes.control_md, p.sizes.control_md)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .radius(p.radius.element)
        .color(p.colors.text_primary)
        .hover(style().background(p.colors.overlay_hover))
        .pressed(style().background(p.colors.overlay_pressed))
        .focus_visible(focus(&p))
}

pub(crate) fn focus(p: &Tokens) -> Style {
    style().ring_with_offset(
        p.sizes.focus_ring_width,
        p.sizes.focus_ring_offset,
        p.colors.focus_ring,
    )
}

pub(crate) fn scrim(p: &Tokens) -> Style {
    style()
        .position(Position::Fixed)
        .inset(0)
        .background(p.colors.overlay_scrim)
        .backdrop_blur(p.spacing.s0_5)
}

pub(crate) fn body_text(p: &Tokens) -> Style {
    let text = p.typography.body;
    style()
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .weight(text.weight)
        .color(p.colors.text_primary)
}

pub(crate) fn supporting_text(p: &Tokens) -> Style {
    let text = p.typography.supporting;
    style()
        .font_size(text.size_px)
        .line_height(text.line_height_px / text.size_px)
        .weight(text.weight)
        .color(p.colors.text_secondary)
}

pub(crate) fn shadows(base: Style, shadows: &[kivact::draw::Shadow]) -> Style {
    shadows.iter().copied().fold(base, Style::shadow)
}

pub(crate) fn motion(base: Style, p: &Tokens, duration: u64) -> Style {
    base.transition(
        TransitionProp::Paint,
        Transition::tween(duration as f64, p.motion.ease_standard),
    )
}

pub(crate) fn anchored(base: Style, p: &Tokens, x: f64, y: f64) -> Style {
    // Anchored layers scale in from their anchor; under reduced motion the
    // decorative transition completes at once, also for coarse pointers.
    motion(base, p, p.motion.fast_max).enter(
        style()
            .opacity(0.0)
            .translate(x, y)
            .scale_xy(LAYER_ENTER_SCALE, LAYER_ENTER_SCALE),
    )
}

pub(crate) fn modal(base: Style, p: &Tokens) -> Style {
    // The dialog recipe does not receive the trigger rectangle.
    motion(base, p, p.motion.medium_max).enter(
        style()
            .opacity(0.0)
            .translate_y(p.spacing.s4)
            .scale_xy(LAYER_ENTER_SCALE, LAYER_ENTER_SCALE),
    )
}
