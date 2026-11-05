//! Role: Compose exclusive navigation panels around the canonical navigation machine.
use kivact_core::state::{IntoText, IntoValue, Value, binding, memo};
use kivact_core::view::{
    Handled, IntoView, Key, Node, NodeBuilder, OverlayAlign, Placement, PointerPhase, Role, View,
    node, node_ref, overlay, show,
};
use kivact_interaction::{NavOpen, use_navigation_menu, use_popup};
use std::rc::Rc;
use std::time::Duration;
struct Link {
    label: Value<String>,
    content: Rc<dyn Fn() -> View>,
}
/// A row of panel triggers with shared exclusive opening state.
pub struct NavigationMenu {
    root: Node,
    links: Vec<Link>,
    open_delay: Duration,
    close_delay: Duration,
    placement: Placement,
    align: OverlayAlign,
    variants: crate::variants::Variants,
    changed: Option<Rc<dyn Fn(bool)>>,
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
        variants: crate::variants::Variants::default(),
        changed: None,
    }
}
impl NavigationMenu {
    /// Report accepted opening and closing transitions.
    pub fn on_open_change(mut self, f: impl Fn(bool) + 'static) -> Self {
        self.changed = Some(Rc::new(f));
        self
    }
    /// Add a panel trigger. Keys belong to the application's navigation data.
    pub fn link<K: 'static, V: IntoView>(
        mut self,
        _key: K,
        label: impl IntoText,
        content: impl Fn() -> V + 'static,
    ) -> Self {
        self.links.push(Link {
            label: label.into_text_value(),
            content: Rc::new(move || content().into_view()),
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
    /// Add a kit-defined visual variant.
    pub fn variant(self, variant: impl crate::Variant) -> Self {
        self.variants.add(variant);
        self
    }
}
impl NodeBuilder for NavigationMenu {
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
            variants,
            changed,
        } = self;
        let api = use_navigation_menu();
        let api = if let Some(changed) = changed {
            api.on_open_change(move |open| changed(open))
        } else {
            api
        };
        let state = api.state();
        let disabled = root
            .accessible_disabled()
            .unwrap_or_else(|| false.into_value());
        let mut children = Vec::new();
        for (at, Link { label, content }) in links.into_iter().enumerate() {
            let shown = memo(
                move || matches!(state.get(), NavOpen::Hover(i) | NavOpen::Pinned(i) if i == at),
            );
            let pinned = binding(
                move || state.get() == NavOpen::Pinned(at),
                move |pin| {
                    if pin {
                        api.pin(at);
                    } else if state.get_untracked() == NavOpen::Pinned(at) {
                        api.close();
                    }
                },
            );
            let pop = use_popup(pinned);
            let anchor = node_ref();
            let surface = node_ref();
            let trigger = crate::layer::with_recipe_style(
                crate::button(label)
                    .disabled(disabled)
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
                            api.close();
                            Handled::Yes
                        }
                        Key::ArrowDown if !disabled.get_untracked() => {
                            api.pin(at);
                            Handled::Yes
                        }
                        _ => Handled::No,
                    }),
                |r| r.navigation_menu_recipes().trigger_recipe(),
                &variants,
                move || state.get(),
            );
            children.push(trigger.into_view());
            let panel_variants = variants.clone();
            children.push(
                show(shown, move || {
                    let content = content.clone();
                    let panel = crate::layer::with_recipe_style(
                        node().role(Role::Group).name(label),
                        |r| r.navigation_menu_recipes().panel_recipe(),
                        &panel_variants,
                        move || state.get(),
                    )
                    .child(content());
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
                                api.close();
                                anchor.focus();
                                Handled::Yes
                            } else {
                                Handled::No
                            }
                        })
                })
                .into_view(),
            );
        }
        crate::layer::with_base_recipe_style(
            root.roving_focus(kivact_core::view::Orientation::Horizontal),
            |r| r.navigation_menu_recipes().root_recipe(),
            &variants,
            move || state.get(),
        )
        .child(children)
        .into_view()
    }
}
