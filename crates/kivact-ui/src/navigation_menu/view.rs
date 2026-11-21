//! Role: The navigation menu builder, its panel and trigger rendering.

use crate::navigation_menu::look::NavigationMenuTheme;
use kivact_core::state::{Callback, IntoText, IntoValue, Value, binding, callback, memo};
use kivact_core::view::{
    Element, Handled, IntoView, Key, Node, OverlayAlign, Placement, PointerPhase, Role, View, node,
    node_ref, overlay, show,
};
use kivact_interaction::{NavOpen, use_navigation_menu, use_popup};
use std::rc::Rc;
use std::time::Duration;
enum Entry {
    Panel(Link),
    Action {
        label: Value<String>,
        press: kivact_core::state::Callback<()>,
    },
}
struct Link {
    label: Value<String>,
    content: Rc<dyn Fn() -> View>,
}
/// A row of panel triggers with shared exclusive opening state.
pub struct NavigationMenu {
    root: Node,
    links: Vec<Entry>,
    open_delay: Duration,
    close_delay: Duration,
    placement: Placement,
    align: OverlayAlign,
    changed: Option<Callback<bool>>,
}
/// Construct a navigation landmark under an accessible label.
#[track_caller]
pub fn navigation_menu(label: impl IntoText) -> NavigationMenu {
    NavigationMenu {
        root: node().role(Role::Navigation).name(label),
        links: Vec::new(),
        open_delay: Duration::ZERO,
        close_delay: Duration::from_millis(150),
        placement: Placement::Below,
        align: OverlayAlign::Start,
        changed: None,
    }
}
impl NavigationMenu {
    /// Report accepted opening and closing transitions.
    pub fn on_open_change<R: kivact_core::view::HandlerResult>(
        mut self,
        f: impl Fn(bool) -> R + 'static,
    ) -> Self {
        self.changed = Some(callback(f));
        self
    }
    /// Add a panel trigger. Keys belong to the application's navigation data.
    #[track_caller]
    pub fn link<K: 'static, V: IntoView>(
        mut self,
        _key: K,
        label: impl IntoText,
        content: impl Fn() -> V + 'static,
    ) -> Self {
        self.links.push(Entry::Panel(Link {
            label: label.into_text_value(),
            content: Rc::new(move || content().into_view()),
        }));
        self
    }
    /// Add a direct navigation action without constructing or opening a panel.
    #[track_caller]
    pub fn action<K: 'static, M>(
        mut self,
        _key: K,
        label: impl IntoText,
        press: impl kivact_core::state::IntoHandler<(), M>,
    ) -> Self {
        self.links.push(Entry::Action {
            label: label.into_text_value(),
            press: kivact_core::state::Callback::from_handler(
                kivact_core::state::IntoHandler::into_handler(press),
            ),
        });
        self
    }
    /// Delay pointer opening; keyboard and press opening stay immediate.
    pub fn open_delay(mut self, delay: Duration) -> Self {
        self.open_delay = delay;
        self
    }
    /// Keep a hovered panel open briefly during pointer travel.
    pub fn close_delay(mut self, delay: Duration) -> Self {
        self.close_delay = delay;
        self
    }
    /// Place panels on this side of the trigger.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
    /// Align each panel with its trigger.
    pub fn align(mut self, align: OverlayAlign) -> Self {
        self.align = align;
        self
    }
}
impl Element for NavigationMenu {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for NavigationMenu {
    fn into_view(self) -> View {
        let Self {
            mut root,
            links,
            open_delay,
            close_delay,
            placement,
            align,
            changed,
        } = self;
        let variants = root.authored().variants();
        let api = use_navigation_menu();
        let api = if let Some(changed) = changed {
            api.on_open_change(move |open| changed.call(open))
        } else {
            api
        };
        let state = api.state();
        let disabled = root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let mut children = Vec::new();
        for entry in links {
            let Link { label, content } = match entry {
                Entry::Panel(link) => link,
                Entry::Action { label, press } => {
                    children.push(navigation_action(api, label, press, disabled, &variants));
                    continue;
                }
            };
            let at = api.panel_key();
            let shown = memo(
                move || matches!(state.get(), NavOpen::Hover(i) | NavOpen::Pinned(i) if i == at),
            );
            let pinned = pinned_binding(api, at);
            let pop = use_popup(pinned);
            let anchor = node_ref();
            let presentation = PanelPresentation {
                shown,
                variants: variants.clone(),
                placement,
                align,
                open_delay,
                close_delay,
            };
            let trigger = navigation_trigger(api, at, label, pop, anchor, disabled, &presentation);
            children.push(trigger.into_view());
            children.push(navigation_panel(
                api,
                at,
                Link { label, content },
                pop,
                anchor,
                presentation,
            ));
        }
        crate::layer::with_base_recipe_style(
            root.roving_focus(kivact_core::view::Orientation::Horizontal),
            |r: &NavigationMenuTheme| r.root_recipe(),
            &variants,
            move || state.get(),
        )
        .child(children)
        .into_view()
    }
}

/// Independent placement, appearance and actual visibility for one mounted panel.
struct PanelPresentation {
    shown: Value<bool>,
    variants: kivact_core::view::Variants,
    placement: Placement,
    align: OverlayAlign,
    open_delay: Duration,
    close_delay: Duration,
}
/// Render one exclusive navigation panel with its existing popup part.
fn navigation_panel(
    api: kivact_interaction::NavigationMenuApi,
    at: usize,
    link: Link,
    pop: kivact_interaction::PopupApi,
    anchor: kivact_core::view::NodeRef,
    presentation: PanelPresentation,
) -> View {
    let Link { label, content } = link;
    let PanelPresentation {
        shown,
        variants,
        placement,
        align,
        close_delay,
        ..
    } = presentation;
    let state = api.state();
    let surface = node_ref();
    show(shown, move || {
        let content = content.clone();
        let panel = crate::layer::with_recipe_style(
            node().role(Role::Group).name(label),
            |r: &NavigationMenuTheme| r.panel_recipe(),
            &variants,
            move || state.get(),
        )
        .child(crate::navigation_menu::context::provide_panel(
            kivact_core::state::callback(move |()| api.hide()),
            variants.clone(),
            move || content(),
        ));
        overlay(anchor, placement, panel)
            .align(align)
            .offset(0)
            .node_ref(surface)
            .with_node(|n| n.part(pop.content()))
            .on_pointer(move |event| {
                match event.phase() {
                    PointerPhase::Enter => api.hover_after(at, Duration::ZERO),
                    PointerPhase::Leave => api.leave_after(at, close_delay),
                    _ => (),
                }
                Handled::No
            })
            .on_key(move |event| {
                if event.key() == Key::Escape {
                    api.hide();
                    anchor.focus();
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
    })
    .into_view()
}

/// Bind a panel's pinned state to the shared exclusive navigation authority.
fn pinned_binding(
    api: kivact_interaction::NavigationMenuApi,
    at: usize,
) -> kivact_core::state::Signal<bool> {
    let state = api.state();
    binding(
        move || state.get() == NavOpen::Pinned(at),
        move |pin| {
            if pin {
                api.show(at);
            } else if state.get_untracked() == NavOpen::Pinned(at) {
                api.hide();
            }
        },
    )
}

/// Attach navigation trigger input to the canonical exclusive panel authority.
fn navigation_trigger(
    api: kivact_interaction::NavigationMenuApi,
    at: usize,
    label: Value<String>,
    pop: kivact_interaction::PopupApi,
    anchor: kivact_core::view::NodeRef,
    disabled: Value<bool>,
    presentation: &PanelPresentation,
) -> View {
    let shown = presentation.shown;
    let (open_delay, close_delay) = (presentation.open_delay, presentation.close_delay);
    let variants = &presentation.variants;
    crate::layer::with_recipe_style(
        crate::button(label)
            .disabled(disabled)
            .trailing(kivact_core::view::icon("chevron-down").style(move || {
                kivact_core::style::style()
                    .size(12, 12)
                    .rotate(if shown.get() { 180.0 } else { 0.0 })
            }))
            .with_node(|n| n.part(pop.trigger()))
            .node_ref(anchor)
            .with_node(|n| n.expanded(shown))
            .on_pointer(move |event| {
                match event.phase() {
                    PointerPhase::Enter if !disabled.get_untracked() => {
                        api.hover_after(at, open_delay)
                    }
                    PointerPhase::Leave => api.leave_after(at, close_delay),
                    _ => (),
                }
                Handled::No
            })
            .on_key(move |event| match event.key() {
                Key::Escape if shown.get_untracked() => {
                    api.hide();
                    Handled::Yes
                }
                Key::ArrowDown if !disabled.get_untracked() => {
                    api.show(at);
                    Handled::Yes
                }
                _ => Handled::No,
            }),
        |r: &NavigationMenuTheme| r.trigger_recipe(),
        variants,
        move || crate::theme::NavigationTriggerState { open: shown.get() },
    )
    .into_view()
}

fn navigation_action(
    api: kivact_interaction::NavigationMenuApi,
    label: Value<String>,
    press: kivact_core::state::Callback<()>,
    disabled: Value<bool>,
    variants: &kivact_core::view::Variants,
) -> View {
    let state = api.state();
    crate::layer::with_recipe_style(
        crate::button(label)
            .disabled(disabled)
            .with_node(|node| node.role(Role::Link))
            .on_press(move || {
                api.hide();
                press.call(());
            }),
        |r: &NavigationMenuTheme| r.action_recipe(),
        variants,
        move || state.get(),
    )
    .into_view()
}
