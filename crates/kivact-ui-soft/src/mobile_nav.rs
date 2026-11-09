//! Role: Render the responsive Neutral navigation drawer and its toggle.
use crate::layout::LayoutInset;
use crate::stack::{ContainerInset, rtl};

use kivact::interaction::use_dialog;
use kivact::motion::Transition;
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ViewFn;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::scroll;
use kivact::view::text;

use kivact::geometry::Edge;
use kivact::interaction::DialogMode;
use kivact::state::Binding;
use kivact::state::IntoText;
use kivact::state::Machine;
use kivact::state::MachineHandle;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::machine;
use kivact::state::provide;
use kivact::state::signal;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::view::IntoView;
use kivact::view::NodeRef;
use kivact::view::Placement;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::overlay;
use kivact::view::show;

#[derive(Clone, Copy, Default)]
/// Copy handle for the surrounding app shell's mobile drawer state.
pub struct AppShellMobile {
    context: Option<MobileContext>,
}
#[derive(Clone, Copy)]
struct MobileContext {
    state: MachineHandle<MobileMachine>,
    open: Binding<bool>,
    toggle: NodeRef,
    drawer: NodeRef,
    // xtask-allow(state-fields): passive record - auto toggle
    auto_toggle: bool,
}
#[derive(Clone, Copy, PartialEq)]
enum AppShellMobileState {
    Desktop,
    Mobile,
}
/// Readonly snapshot read for each viewport-width event.
struct MobileMachine {
    /// Whether the drawer binding is open when the width event arrives.
    open: bool,
}
impl Machine for MobileMachine {
    type State = AppShellMobileState;
    /// Whether the viewport is below the mobile breakpoint.
    type Event = bool;
    /// Close the drawer binding.
    type Effect = ();
    fn transition(
        &self,
        state: &AppShellMobileState,
        below: bool,
    ) -> Step<AppShellMobileState, bool, ()> {
        match (state, below) {
            (_, true) => Step::to(AppShellMobileState::Mobile),
            (AppShellMobileState::Mobile, false) if self.open => {
                Step::to(AppShellMobileState::Desktop).with(())
            }
            (_, false) => Step::to(AppShellMobileState::Desktop),
        }
    }
}
/// Read mobile navigation from the surrounding app shell, or an inert default outside one.
pub fn use_app_shell_mobile() -> AppShellMobile {
    try_use_context::<AppShellMobile>()
        .map(|value| value.get())
        .unwrap_or_default()
}
impl AppShellMobile {
    pub(crate) fn configured(
        mobile: Option<Value<bool>>,
        open: Option<Binding<bool>>,
        auto_toggle: bool,
    ) -> Self {
        let Some(open) = open else {
            return Self::default();
        };
        let initial = if mobile.is_some_and(|mobile| mobile.get_untracked()) {
            AppShellMobileState::Mobile
        } else {
            AppShellMobileState::Desktop
        };
        let state = machine(
            initial,
            move || MobileMachine { open: open.get() },
            move |_, ()| open.set(false),
        );
        if let Some(mobile) = mobile {
            state.feed(mobile, |below| below);
        }
        Self {
            context: Some(MobileContext {
                state,
                open,
                toggle: node_ref(),
                drawer: node_ref(),
                auto_toggle,
            }),
        }
    }
    /// Return whether the configured shell currently uses its mobile layout.
    pub fn is_mobile(self) -> bool {
        self.context
            .is_some_and(|context| context.state.get() == AppShellMobileState::Mobile)
    }
    /// Return whether the mobile-layout drawer is currently open.
    pub fn is_open(self) -> bool {
        self.context.is_some_and(|context| {
            context.state.get() == AppShellMobileState::Mobile && context.open.get()
        })
    }
    /// Return whether an app shell provided mobile navigation controls.
    pub fn is_enabled(self) -> bool {
        self.context.is_some()
    }
    /// Return whether the shell uses an automatic drawer toggle; true outside a configured shell.
    pub fn has_auto_toggle(self) -> bool {
        self.context.is_none_or(|context| context.auto_toggle)
    }
    /// Return the configured drawer's node reference, or `None` outside a shell.
    pub fn drawer_ref(self) -> Option<NodeRef> {
        self.context.map(|context| context.drawer)
    }
    /// Request that the configured drawer open; do nothing outside a shell or in the desktop layout.
    pub fn open(self) {
        if let Some(context) = self.context
            && self.is_mobile()
        {
            context.open.set(true);
        }
    }
    /// Request that the configured drawer close; do nothing outside a shell.
    pub fn close(self) {
        if let Some(context) = self.context {
            context.open.set(false);
        }
    }
    /// Invert the configured drawer's open binding; do nothing outside a shell or in the desktop layout.
    pub fn toggle(self) {
        if let Some(context) = self.context
            && self.is_mobile()
        {
            context.open.set(!context.open.get());
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Logical edge on which a mobile navigation drawer opens.
pub enum DrawerSide {
    /// Open at the reading direction's start edge.
    Start,
    /// Open at the reading direction's end edge.
    End,
    #[default]
    /// Choose the edge nearest the drawer toggle, falling back to the logical end.
    Auto,
}

/// A modal navigation drawer whose content is rebuilt on each opening.
pub struct MobileNav {
    root: Node,
    open: Option<Binding<bool>>,
    width: Value<f64>,
    side: Value<DrawerSide>,
    label: Option<Value<String>>,
    header: Option<Value<String>>,
    header_view: Option<ViewFn>,
    content: ViewFn,
}

/// Create a drawer whose content factory runs whenever the drawer opens.
pub fn mobile_nav<V: IntoView>(content: impl IntoViewFn<V>) -> MobileNav {
    MobileNav {
        root: node(),
        open: None,
        width: (320.0).into_value(),
        side: (DrawerSide::Auto).into_value(),
        label: None,
        header: None,
        header_view: None,
        content: content.into_view_fn(),
    }
}

impl MobileNav {
    /// Use an application-owned open binding instead of the surrounding shell binding.
    pub fn open(mut self, open: impl IntoBinding<bool>) -> Self {
        self.open = Some(open.into_binding());
        self
    }
    /// Set drawer width in pixels; negative widths are clamped to zero.
    pub fn width(mut self, width: impl IntoValue<f64>) -> Self {
        let width = width.into_value();
        self.width = (move || width.get().max(0.0)).into_value();
        self
    }
    /// Choose the logical edge or automatic placement of the drawer.
    pub fn side(mut self, side: impl IntoValue<DrawerSide>) -> Self {
        self.side = side.into_value();
        self
    }
    /// Set the drawer's accessible navigation label.
    pub fn name(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }
    /// Set a plain-text drawer heading.
    pub fn header(mut self, header: impl IntoText) -> Self {
        self.header = Some(header.into_text_value());
        self
    }
    /// Build custom heading content each time the drawer opens.
    pub fn header_view<V: IntoView>(mut self, header: impl IntoViewFn<V>) -> Self {
        self.header_view = Some(header.into_view_fn());
        self
    }
}

impl NodeBuilder for MobileNav {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for MobileNav {
    fn into_view(self) -> View {
        let Self {
            root,
            open,
            width,
            side,
            label,
            header,
            header_view,
            content,
        } = self;
        kivact::view::component(move || {
            let mobile = use_app_shell_mobile();
            let open = open
                .or(mobile.context.map(|context| context.open))
                .unwrap_or_else(|| signal(false).into_binding());
            let anchor = mobile
                .context
                .map(|context| context.toggle)
                .unwrap_or_else(node_ref);
            let drawer = mobile.drawer_ref().unwrap_or_else(node_ref);
            let fallback = kivact::i18n::message("soft.mobile_nav.navigation", "Navigation");
            let label = label.or(header).unwrap_or(fallback);
            let behavior = use_dialog(open, DialogMode::Drawer(Edge::Right));
            root.child(show(behavior.is_open(), move || {
                let right = match side.get() {
                    DrawerSide::Start => rtl(),
                    DrawerSide::End => !rtl(),
                    DrawerSide::Auto => anchor.rect().get().map_or(!rtl(), |rect| {
                        rect.x + rect.width / 2.0
                            >= kivact::platform::window_size().get().width / 2.0
                    }),
                };
                let scrim = drawer_scrim();
                let header = drawer_header(header_view.as_ref(), header, behavior, open);
                let body = drawer_body(content.clone());
                let panel = drawer_panel(drawer, width, right, header, body);
                (
                    scrim,
                    overlay(anchor, Placement::Center, panel)
                        .with_node(move |node| node.part(behavior.content()))
                        .name(label),
                )
            }))
        })
    }
}

/// An app-shell drawer toggle with an optional custom icon.
pub struct MobileNavToggle {
    root: Node,
    label: Option<Value<String>>,
    icon: Option<View>,
}

/// Create a toggle for the surrounding app shell's mobile navigation.
pub fn mobile_nav_toggle() -> MobileNavToggle {
    MobileNavToggle {
        root: node(),
        label: None,
        icon: None,
    }
}

impl MobileNavToggle {
    /// Override the drawer toggle's accessible label.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }
    /// Mount a replacement toggle icon once.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
}

impl NodeBuilder for MobileNavToggle {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for MobileNavToggle {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            icon: glyph,
        } = self;
        kivact::view::component(move || {
            let mobile = use_app_shell_mobile();
            if !mobile.is_enabled() {
                return View::empty();
            }
            let label = label.unwrap_or_else(|| {
                kivact::i18n::message("soft.mobile_nav.open", "Open navigation")
            });
            let mut trigger = button(label)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Medium)
                .icon(glyph.unwrap_or_else(|| icon("menu").into_view()))
                .hide_label(true)
                .on_press(move || mobile.toggle())
                .with_node(move |node| {
                    node.with_node(move |node| node.expanded(move || mobile.is_open()))
                });
            if let Some(toggle) = mobile.context.map(|context| context.toggle) {
                trigger = trigger.node_ref(toggle);
            }
            if let Some(drawer) = mobile.drawer_ref() {
                trigger = trigger.with_node(move |node| node.controls(drawer));
            }
            root.hidden(move || !mobile.is_mobile())
                .child(trigger)
                .into_view()
        })
    }
}

kivact::state::constant_value!(DrawerSide);

fn drawer_scrim() -> Node {
    node().style(|| {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        let hidden = style().opacity(0.0);
        style()
            .position(Position::Fixed)
            .top(0)
            .bottom(0)
            .left(0)
            .right(0)
            .background(t.colors.overlay_scrim)
            .backdrop_blur(2)
            .enter(hidden.clone())
            .exit(hidden)
            .transition(
                TransitionProp::All,
                Transition::tween(t.motion.medium as f64, t.motion.ease_standard.clone()),
            )
    })
}
fn drawer_header(
    header_view: Option<&ViewFn>,
    header: Option<Value<String>>,
    behavior: kivact::interaction::DialogApi,
    open: Binding<bool>,
) -> impl IntoView {
    let title = if let Some(header_view) = header_view {
        Some(header_view.call(()))
    } else {
        header.map(|label| {
            text(label)
                .with_node(move |node| node.part(behavior.title()))
                .style(|| {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let text = t.typography.heading2;
                    style()
                        .font_size(text.size_px)
                        .line_height(text.line_height_px / text.size_px)
                        .weight(text.weight)
                        .color(t.colors.text_primary)
                })
                .into_view()
        })
    };
    let has_title = title.is_some();
    let close = button(kivact::i18n::message(
        "soft.mobile_nav.close",
        "Close navigation",
    ))
    .variant(ButtonVariant::Ghost)
    .size(ControlSize::Medium)
    .icon(icon("x"))
    .hide_label(true)
    .on_press(move || open.set(false));
    row((title, close)).style(move || {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        style()
            .direction(if rtl() {
                Direction::RowReverse
            } else {
                Direction::Row
            })
            .min_height(t.spacing.s12)
            .padding_x(t.spacing.s2)
            .align_items(Align::Center)
            .shrink(0)
            .justify_content(if has_title {
                Justify::SpaceBetween
            } else {
                Justify::End
            })
            .border_bottom(t.sizes.border_width, t.colors.border)
    })
}
fn drawer_body(content: ViewFn) -> impl IntoView {
    let contents = provide(ContainerInset::default(), move || {
        provide(LayoutInset::default(), move || content.call(()))
    });
    scroll(
        node()
            .style(|| {
                style()
                    .direction(Direction::Column)
                    .padding(kivact::style::use_tokens::<crate::Tokens>().spacing.s2)
            })
            .child(contents),
    )
    .style(style().grow(1).min_height(0))
}
fn drawer_panel(
    drawer: NodeRef,
    width: Value<f64>,
    right: bool,
    header: impl IntoView,
    body: impl IntoView,
) -> Node {
    node()
        .node_ref(drawer)
        .style(move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            let offset = if right { pct(100) } else { pct(-100) };
            let hidden = style().translate_x(offset);
            let base = style()
                .position(Position::Fixed)
                .top(0)
                .bottom(0)
                .width(width.get())
                .max_width(pct(85))
                .clip(true)
                .direction(Direction::Column)
                .background(t.colors.bg_surface)
                .enter(hidden.clone())
                .exit(hidden)
                .transition(
                    TransitionProp::All,
                    Transition::tween(t.motion.medium as f64, t.motion.ease_standard.clone()),
                );
            if right {
                base.right(0)
                    .border_left(t.sizes.border_width, t.colors.border)
            } else {
                base.left(0)
                    .border_right(t.sizes.border_width, t.colors.border)
            }
        })
        .child((header, body))
}
