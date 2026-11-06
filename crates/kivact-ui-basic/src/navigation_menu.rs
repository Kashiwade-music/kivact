//! Role: Lyra appearance for the canonical navigation menu and its actual links.
use crate::Tokens;
use kivact::interaction::NavOpen;
use kivact::style::{Align, Color, Direction, Justify, Style, Theme, style};
use kivact::ui::{Look, NavigationLinkState, NavigationMenuRecipes, NavigationTriggerState};

pub(crate) fn recipes(r: NavigationMenuRecipes) -> NavigationMenuRecipes {
    r.root(|_| root)
        .trigger(|_| trigger)
        .action(|_| action)
        .panel(|_| panel)
        .link(|_| link)
        .link_title(|_| title)
        .link_description(|_| description)
        .link_icon(|_| icon)
}
fn root(_: &Theme, _: &Look<NavOpen>) -> Style {
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .gap(0)
}
fn trigger(t: &Theme, s: &Look<NavigationTriggerState>) -> Style {
    trigger_style(t, s.open)
}
fn action(t: &Theme, _: &Look<NavOpen>) -> Style {
    trigger_style(t, false)
}
fn trigger_style(t: &Theme, open: bool) -> Style {
    let p = t.ext::<Tokens>();
    let hot = style().background(p.muted);
    style()
        .direction(Direction::Row)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .shrink(0)
        .height(36)
        .padding_x(10)
        .padding_y(6)
        .radius(0)
        .font_size(12)
        .weight(500)
        .text_wrap(false)
        .color(p.foreground)
        .background(if open {
            p.muted.with_alpha(0.5)
        } else {
            Color::TRANSPARENT
        })
        .hover(hot.clone())
        .focus(hot)
        .focus_visible(crate::theme_access::theme_ring())
}
fn panel(t: &Theme, _: &Look<NavOpen>) -> Style {
    let p = t.ext::<Tokens>();
    crate::theme_access::shadow_md(crate::theme_access::popover_surface(&p))
        .direction(Direction::Column)
        .padding(4)
        .margin_top(8)
}
fn link(t: &Theme, s: &Look<NavigationLinkState>) -> Style {
    let p = t.ext::<Tokens>();
    let hot = style().background(p.muted);
    style()
        .direction(Direction::Row)
        .align_items(if s.has_description {
            Align::Start
        } else {
            Align::Center
        })
        .gap(8)
        .padding(8)
        .radius(0)
        .font_size(12)
        .color(p.foreground)
        .hover(hot.clone())
        .focus(hot)
        .focus_visible(crate::theme_access::theme_ring())
}
fn title(_: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style().weight(500)
}
fn description(t: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style()
        .line_height(1.375)
        .line_clamp(2)
        .color(t.ext::<Tokens>().muted_foreground)
}
fn icon(_: &Theme, _: &Look<NavigationLinkState>) -> Style {
    style().shrink(0)
}
