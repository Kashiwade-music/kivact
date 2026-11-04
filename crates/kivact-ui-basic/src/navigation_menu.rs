//! Role: The Lyra navigation menu: links and triggers that open content panels.

use crate::parts::{pressable, sized_icon};
use crate::theme_access::tokens;
use crate::tokens::Tokens;
use kivact::control::icon;
use kivact::control::{Node, NodeBuilder, column, node, row, text};
use kivact::i18n::message;
use kivact::interaction::{
    List, ListBoundary, ListMode, ListPolicy, ListSelection, use_list, use_popup,
};
use kivact::motion::{Easing, Transition};
use kivact::primitive::{NodeExt, part};
use kivact::style::{Align, Color, Direction, Justify, Style, TransitionProp, style};
use kivact::ui::{ListActivation, ListItem};
use kivact::view::HandlerResult;
use kivact::view::{
    Handled, IntoText, IntoValue, IntoView, Key, Orientation, OverlayAlign, Placement,
    PointerPhase, Role, View, callback, node_ref, overlay, provide, show, try_context,
};
use kivact::view::{ReadingDirection, current_direction};
use kivact::{Callback, Signal, Value, binding, effect_on, memo, signal};
use std::rc::Rc;

/// One entry of a [`navigation_menu`]: a plain link or a trigger that opens a panel.
#[derive(Clone)]
pub struct NavItem {
    label: String,
    kind: NavKind,
}

#[derive(Clone)]
enum NavKind {
    Link(Callback<()>),
    Menu(Rc<dyn Fn() -> View>),
}

/// Items are equal when their labels and kinds are. A link's callback is
/// read once, when the menu is built, so it does not take part.
impl PartialEq for NavItem {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
            && match (&self.kind, &other.kind) {
                (NavKind::Link(_), NavKind::Link(_)) => true,
                (NavKind::Menu(a), NavKind::Menu(b)) => Rc::ptr_eq(a, b),
                _ => false,
            }
    }
}

impl NavItem {
    /// A link that calls `on_press` (a click, Enter or Space). Setup only.
    #[track_caller]
    pub fn link<R: HandlerResult>(
        label: impl Into<String>,
        on_press: impl Fn() -> R + 'static,
    ) -> Self {
        NavItem {
            label: label.into(),
            kind: NavKind::Link(callback(move |()| on_press())),
        }
    }

    /// A trigger that opens the panel `content` builds: on hover, a click,
    /// Enter, Space and ArrowDown. `content` runs again for each opening.
    pub fn menu<V: IntoView>(label: impl Into<String>, content: impl Fn() -> V + 'static) -> Self {
        NavItem {
            label: label.into(),
            kind: NavKind::Menu(Rc::new(move || content().into_view())),
        }
    }
}

/// Which panel of a menu is open, and why: hovered (closed when the pointer
/// leaves) or pinned (by a press, a key or an assistive action).
#[derive(Clone, Copy, PartialEq, Eq)]
enum NavOpen {
    Closed,
    Hover(usize),
    Pinned(usize),
}

/// What a panel provides to the links inside it.
#[derive(Clone, Copy)]
struct PanelCx {
    close: Callback<()>,
}

/// `h-9 px-2.5 text-xs font-medium`: `bg-muted` while hovered or focused,
/// `bg-muted/50` while its panel is open, with the theme's focus ring while
/// the focus is visible.
fn trigger_style(p: &Tokens, open: bool) -> Style {
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

/// A navigation menu (navigation-menu.tsx): a row of links and panel triggers.
///
/// Hovering a trigger opens its panel and leaving the trigger and the panel
/// closes it; a click, Enter, Space or ArrowDown pins it open until Escape,
/// an outside press, or choosing a [`navigation_menu_link`]. One panel is
/// open at a time. Left and Right move the focus along the row. Every panel
/// has its own surface under its trigger; the shared animated viewport of
/// the web component is not reproduced. The number of entries is read once,
/// when the menu is built; labels follow the value. The links and triggers
/// are a horizontal list (`kivact::interaction::use_list`), so Home and End
/// go to the first and last, and the arrows run from the right in a
/// right-to-left boundary.
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn nav() -> impl IntoView {
///     let panel = || {
///         column((
///             basic::navigation_menu_link("Introduction").description("Re-usable components."),
///             basic::navigation_menu_link("Installation"),
///         ))
///         .style(style().width(280))
///     };
///     basic::navigation_menu(vec![
///         basic::NavItem::menu("Getting started", panel),
///         basic::NavItem::link("Docs", || {}),
///     ])
/// }
/// ```
#[track_caller]
pub fn navigation_menu(items: impl IntoValue<Vec<NavItem>>) -> Node {
    let items = items.into_value();
    let rtl = current_direction() == ReadingDirection::Rtl;
    let count = items.with_untracked(Vec::len);
    let open = signal(NavOpen::Closed);
    // The links and triggers are a horizontal list: the arrows move the focus along it.
    let labels = memo(move || {
        items.with(|all| {
            all.iter()
                .enumerate()
                .map(|(at, item)| ListItem::new(at, item.label.clone()))
                .collect::<Vec<_>>()
        })
    });
    let focus = signal(None::<usize>);
    let row_list = use_list(
        labels,
        ListSelection::Single(focus.into()),
        ListPolicy {
            axis: Some(Orientation::Horizontal),
            boundary: ListBoundary::Wrap,
            activation: ListActivation::Manual,
            mode: ListMode::Activate,
            ..ListPolicy::default()
        },
    );
    let mut children: Vec<View> = Vec::with_capacity(count);
    for at in 0..count {
        let label = memo(move || {
            items.with(|all| {
                all.get(at)
                    .map(|item| item.label.clone())
                    .unwrap_or_default()
            })
        });
        let kind = items.with_untracked(|all| all.get(at).map(|item| item.kind.clone()));
        match kind {
            Some(NavKind::Link(on_press)) => {
                let (link, _) = pressable(node(), Role::Link, Some(on_press));
                children.push(
                    // Every link and trigger stays a tab stop, as in the web
                    // component; the list only moves the focus by the arrows.
                    link.part(row_list.item(at))
                        .role(Role::Link)
                        .selected(false)
                        .focusable(true)
                        .name(label)
                        .style(move || trigger_style(&tokens(), false))
                        .child(text(label))
                        .into_view(),
                );
            }
            Some(NavKind::Menu(content)) => {
                let (trigger, panel) = menu_item(at, label, content, open, row_list, rtl);
                children.push(trigger.into_view());
                children.push(panel);
            }
            None => {}
        }
    }
    row(children)
        .role(Role::Group)
        .name(message("basic.navigation_menu.label", "Navigation"))
        .style(
            style()
                .direction(if rtl {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .gap(0),
        )
}

/// The trigger and the panel overlay of item `at` of `row_list`; `open` is
/// the panel of the menu that is open.
fn menu_item(
    at: usize,
    label: Value<String>,
    content: Rc<dyn Fn() -> View>,
    open: Signal<NavOpen>,
    row_list: List<usize>,
    rtl: bool,
) -> (Node, View) {
    let shown =
        memo(move || matches!(open.get(), NavOpen::Hover(i) | NavOpen::Pinned(i) if i == at));
    // The popup's binding is only the pinned state: a press on a panel that
    // only hover opened pins it, a press on a pinned one closes it, and the
    // popup's dismissals (Escape, an outside press, Tab away) close it.
    let pinned = binding(
        memo(move || open.get() == NavOpen::Pinned(at)),
        move |pin: bool| {
            if pin {
                open.set(NavOpen::Pinned(at));
            } else if open.get_untracked() == NavOpen::Pinned(at) {
                open.set(NavOpen::Closed);
            }
        },
    );
    let pop = use_popup(pinned);
    let attachment = pop.trigger();
    let state = attachment.state();
    let anchor = attachment.node_ref();
    let panel_ref = node_ref();
    let hot = {
        let on_trigger = anchor.hovered();
        let on_panel = panel_ref.hovered();
        memo(move || on_trigger.get() || on_panel.get())
    };
    // The pointer ending its visit closes a panel that only hover opened.
    effect_on(hot, move |&hot| {
        if !hot && open.get_untracked() == NavOpen::Hover(at) {
            open.set(NavOpen::Closed);
        }
    });
    let close = callback(move |()| open.set(NavOpen::Closed));
    let chevron_state = state;
    let chevron = sized_icon(icon("chevron-down"), 12.0).style(move || {
        style()
            .margin_left(4)
            .translate(0, 1)
            .rotate(if shown.get() || chevron_state.get().open {
                180.0
            } else {
                0.0
            })
            .transition(
                TransitionProp::Paint,
                Transition::tween(300, Easing::CubicBezier(0.4, 0.0, 0.2, 1.0)),
            )
    });
    let trigger = node()
        .part(attachment)
        // The trigger reports what is drawn: expanded while its panel is
        // shown, hovered or pinned (a later part's semantics replace an
        // earlier one's).
        .part(part(|input| input).expanded(shown))
        .part(row_list.item(at))
        .role(Role::Button)
        .selected(false)
        .focusable(true)
        .name(label)
        .style(move || trigger_style(&tokens(), shown.get()))
        .on_pointer(move |event| {
            if event.phase() == PointerPhase::Enter {
                open.update(|state| {
                    if *state != NavOpen::Pinned(at) {
                        *state = NavOpen::Hover(at);
                    }
                });
            }
            Handled::No
        })
        .on_key(move |event| {
            if event.key() == Key::Escape && shown.get_untracked() {
                open.set(NavOpen::Closed);
                return Handled::Yes;
            }
            if event.key() == Key::ArrowDown
                && !event.modifiers().alt
                && open.get_untracked() != NavOpen::Pinned(at)
            {
                open.set(NavOpen::Pinned(at));
                return Handled::Yes;
            }
            Handled::No
        })
        .child(row((text(label), chevron)).style(style().align_items(Align::Center)));
    let surface = show(shown, move || {
        let attachment = pop.content();
        let body = provide(PanelCx { close }, {
            let content = content.clone();
            move || content()
        });
        let card = node()
            .style(move || {
                let p = tokens();
                crate::theme_access::shadow_md(crate::theme_access::popover_surface(&p))
                    .direction(Direction::Column)
                    .padding(4)
            })
            .child(body);
        // The transparent 8 px above the card keeps the pointer inside the panel on its way down.
        let content = column((node().style(style().height(8)), card)).node_ref(panel_ref);
        overlay(anchor, Placement::Below, content)
            .align(if rtl {
                OverlayAlign::End
            } else {
                OverlayAlign::Start
            })
            .offset(0)
            .part(attachment)
            .role(Role::Group)
            .name(label)
    })
    .into_view();
    (trigger, surface)
}

/// A link for the panels of a [`navigation_menu`] (`NavigationMenuLink`).
///
/// Created by [`navigation_menu_link`]. Pressing it calls the callback and
/// closes the panel it is in.
pub struct NavLink {
    root: Node,
    title: Value<String>,
    description: Option<Value<String>>,
    icon: Option<View>,
    on_press: Option<Callback<()>>,
}

/// A panel link titled `title`.
#[track_caller]
pub fn navigation_menu_link(title: impl IntoText) -> NavLink {
    let title = title.into_text_value();
    NavLink {
        root: node().name(title),
        title,
        description: None,
        icon: None,
        on_press: None,
    }
}

impl NavLink {
    /// Show `text` under the title, muted (at most two lines).
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// Show `icon` before the text.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Call `f` when the link is pressed (click, Enter, Space).
    pub fn on_press<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.on_press = Some(callback(move |_| f()));
        self
    }
}

impl NodeBuilder for NavLink {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for NavLink {
    fn into_view(self) -> View {
        let panel = try_context::<PanelCx>();
        let user = self.on_press;
        let press = callback(move |()| {
            if let Some(user) = user {
                user.call(());
            }
            if let Some(panel) = panel {
                panel.close.call(());
            }
        });
        let (link, _) = pressable(self.root, Role::Link, Some(press));
        let has_description = self.description.is_some();
        let link = link.base_style(move || {
            let p = tokens();
            let hot = style().background(p.muted);
            style()
                .direction(Direction::Row)
                .align_items(if has_description {
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
        });
        let title = self.title;
        let text_column = column((
            text(title).style(style().weight(500)),
            self.description.map(|description| {
                text(description).style(
                    style()
                        .line_height(1.375)
                        .line_clamp(2)
                        .color(crate::Color::MutedForeground),
                )
            }),
        ))
        .style(style().gap(4).min_width(0));
        link.child(self.icon).child(text_column).into_view()
    }
}
