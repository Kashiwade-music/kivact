//! Role: The navigation menu builder, its panel and trigger rendering.

use crate::navigation_menu::look::NavigationMenuTheme;
use kivact_core::state::{IntoText, IntoValue, Signal, Value, signal};
use kivact_core::view::{
    Element, Handled, IntoView, Key, Node, OverlayAlign, Placement, PointerPhase, Role, View, node,
    node_ref, overlay, show,
};
use kivact_interaction::{NavOpen, use_navigation_menu};
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
    open: Option<Signal<bool>>,
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
        open: None,
    }
}
impl NavigationMenu {
    /// Use this signal as the menu's open state: true while any panel is open.
    pub fn open(mut self, open: Signal<bool>) -> Self {
        self.open = Some(open);
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
            open,
        } = self;
        let variants = root.authored().variants();
        let api = use_navigation_menu(open.unwrap_or_else(|| signal(false)));
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
            let panel = api.panel();
            let anchor = node_ref();
            let presentation = PanelPresentation {
                shown: panel.is_open(),
                state,
                variants: variants.clone(),
                placement,
                align,
                open_delay,
                close_delay,
            };
            let trigger = navigation_trigger(panel, label, anchor, disabled, &presentation);
            children.push(trigger.into_view());
            children.push(navigation_panel(
                panel,
                Link { label, content },
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
    state: Value<NavOpen>,
    variants: kivact_core::view::Variants,
    placement: Placement,
    align: OverlayAlign,
    open_delay: Duration,
    close_delay: Duration,
}
/// Render one exclusive navigation panel with its existing popup part.
fn navigation_panel(
    panel: kivact_interaction::NavigationPanelApi,
    link: Link,
    anchor: kivact_core::view::NodeRef,
    presentation: PanelPresentation,
) -> View {
    let Link { label, content } = link;
    let PanelPresentation {
        shown,
        state,
        variants,
        placement,
        align,
        close_delay,
        ..
    } = presentation;
    let surface = node_ref();
    show(shown, move || {
        let content = content.clone();
        let panel_view = crate::layer::with_recipe_style(
            node().role(Role::Group).name(label),
            |r: &NavigationMenuTheme| r.panel_recipe(),
            &variants,
            move || state.get(),
        )
        .child(crate::navigation_menu::context::provide_panel(
            kivact_core::state::callback(move |()| panel.hide()),
            variants.clone(),
            move || content(),
        ));
        overlay(anchor, placement, panel_view)
            .align(align)
            .offset(0)
            .node_ref(surface)
            .with_node(|n| n.part(panel.content()))
            .on_pointer(move |event| {
                match event.phase() {
                    PointerPhase::Enter => panel.hover_after(Duration::ZERO),
                    PointerPhase::Leave => panel.leave_after(close_delay),
                    _ => (),
                }
                Handled::No
            })
            .on_key(move |event| {
                if event.key() == Key::Escape {
                    panel.hide();
                    anchor.focus();
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
    })
    .into_view()
}

/// Attach navigation trigger input to the menu's one exclusive panel authority.
fn navigation_trigger(
    panel: kivact_interaction::NavigationPanelApi,
    label: Value<String>,
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
            .with_node(|n| n.part(panel.trigger()))
            .node_ref(anchor)
            .on_pointer(move |event| {
                match event.phase() {
                    PointerPhase::Enter if !disabled.get_untracked() => {
                        panel.hover_after(open_delay)
                    }
                    PointerPhase::Leave => panel.leave_after(close_delay),
                    _ => (),
                }
                Handled::No
            })
            .on_key(move |event| match event.key() {
                Key::Escape if shown.get_untracked() => {
                    panel.hide();
                    Handled::Yes
                }
                Key::ArrowDown if !disabled.get_untracked() => {
                    panel.show();
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
