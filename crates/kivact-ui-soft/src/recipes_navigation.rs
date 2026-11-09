//! Role: Style canonical navigation landmarks and rich panel links.
use kivact::interaction::NavOpen;
use kivact::style::{Align, Color, Direction, Style, Theme, style};
use kivact::ui::{Look, NavigationLinkState, NavigationMenuRecipes, NavigationTriggerState};
fn direction() -> Direction {
    if crate::stack::rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}
pub(crate) fn navigation_menu(r: NavigationMenuRecipes) -> NavigationMenuRecipes {
    r.root(|_| {
        |_: &Theme, _: &Look<NavOpen>| {
            style()
                .direction(direction())
                .align_items(Align::Center)
                .gap(4)
                .padding(0)
        }
    })
    .trigger(|_| trigger)
    .action(|_| |t: &Theme, _: &Look<NavOpen>| item(t, false))
    .panel(|_| panel)
    .link(|_| link)
    .link_title(|_| {
        |t: &Theme, _: &Look<NavigationLinkState>| {
            let t = t.ext::<crate::Tokens>();
            let f = t.typography.body;
            style()
                .font_size(f.size_px)
                .line_height(f.line_height_px / f.size_px)
                .weight(t.typography.weight_semibold)
                .color(t.colors.text_primary)
        }
    })
    .link_description(|_| {
        |t: &Theme, _: &Look<NavigationLinkState>| {
            let t = t.ext::<crate::Tokens>();
            let f = t.typography.supporting;
            style()
                .font_size(f.size_px)
                .line_height(f.line_height_px / f.size_px)
                .color(t.colors.text_secondary)
        }
    })
    .link_icon(|_| {
        |t: &Theme, _: &Look<NavigationLinkState>| {
            let t = t.ext::<crate::Tokens>();
            style()
                .size(40, 40)
                .shrink(0)
                .radius(t.radius.element)
                .background(t.colors.neutral)
                .color(t.colors.icon_secondary)
                .align_items(Align::Center)
                .justify_content(kivact::style::Justify::Center)
        }
    })
}
fn item(theme: &Theme, open: bool) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let font = t.typography.label;
    style()
        .direction(direction())
        .align_items(Align::Center)
        .gap(8)
        .padding_x(12)
        .padding_y(6)
        .height(kivact::style::auto())
        .min_height(0)
        .min_width(0)
        .border(0, Color::TRANSPARENT)
        .radius(t.radius.element)
        .font_size(font.size_px)
        .line_height(font.line_height_px / font.size_px)
        .weight(font.weight)
        .color(if open {
            t.colors.text_primary
        } else {
            t.colors.text_secondary
        })
        .background(if open {
            t.colors.overlay_hover
        } else {
            Color::TRANSPARENT
        })
        .hover(style().background(t.colors.overlay_hover))
        .pressed(style().background(t.colors.overlay_pressed))
        .disabled(style().opacity(t.sizes.disabled_opacity))
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
}
fn trigger(theme: &Theme, s: &Look<NavigationTriggerState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let ink = t.colors.text_secondary;
    let open = s.open;
    let rtl = crate::stack::rtl();
    let base = item(theme, open);
    let base = if rtl {
        base.padding_left(36)
    } else {
        base.padding_right(36)
    };
    base.paint(move |draw, size| {
        use kivact::draw::{Path, Stroke};
        use kivact::geometry::Point;
        let x = if rtl { 20.0 } else { size.width - 20.0 };
        let y = size.height / 2.0;
        let flip = if open { -1.0 } else { 1.0 };
        draw.stroke_path(
            &Path::new()
                .move_to(Point::new(x - 4.0, y - 2.0 * flip))
                .line_to(Point::new(x, y + 2.0 * flip))
                .line_to(Point::new(x + 4.0, y - 2.0 * flip)),
            &Stroke::new(1.5),
            ink,
        );
    })
}
fn panel(theme: &Theme, _: &Look<NavOpen>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    let hidden = style()
        .opacity(0.0)
        .translate_y(-t.spacing.s2)
        .scale_xy(0.95, 0.95);
    let base = style()
        .direction(Direction::Column)
        .padding(t.spacing.s1)
        .gap(t.spacing.s1)
        .min_width(280)
        .background(t.colors.bg_popover)
        .radius(t.radius.container)
        .border(0, Color::TRANSPARENT)
        .enter(hidden)
        .transition(
            kivact::style::TransitionProp::All,
            kivact::motion::Transition::tween(
                t.motion.fast_max as f64,
                t.motion.ease_standard.clone(),
            ),
        );
    t.shadows.low.iter().copied().fold(base, Style::shadow)
}
fn link(theme: &Theme, s: &Look<NavigationLinkState>) -> Style {
    let t = theme.ext::<crate::Tokens>();
    style()
        .direction(direction())
        .align_items(if s.has_description {
            Align::Start
        } else {
            Align::Center
        })
        .gap(t.spacing.s3)
        .padding(t.spacing.s3)
        .min_width(0)
        .radius(t.radius.element)
        .background(
            if !s.input.disabled && (s.input.hovered || s.input.is_focused()) {
                t.colors.overlay_hover
            } else {
                Color::TRANSPARENT
            },
        )
        .disabled(style().opacity(t.sizes.disabled_opacity))
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
}
