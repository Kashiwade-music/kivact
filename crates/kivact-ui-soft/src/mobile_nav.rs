//! Role: Render the responsive Neutral navigation drawer and its toggle.
use crate::layout::LayoutInset;
use crate::stack::{ContainerInset, rtl};

use kivact::interaction::use_dialog;
use kivact::motion::Transition;
use kivact::state::IntoBinding;
use kivact::state::IntoValue;
use kivact::view::Element;
use kivact::view::IntoViewFn;
use kivact::view::Node;
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
use kivact::style::Length;
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
    toggle: NodeRef,
    drawer: NodeRef,
    // xtask-allow(state-fields): option: fixed at AppShell construction as MobileNavMode::Auto with MobileNavConfig::has_toggle; false means the app supplies its own drawer control.
    auto_toggle: bool,
}
impl MobileContext {
    fn is_mobile(self) -> bool {
        matches!(self.state.get(), AppShellMobileState::Mobile { .. })
    }
    fn is_open(self) -> bool {
        matches!(self.state.get(), AppShellMobileState::Mobile { open: true })
    }
    fn set_open(self, open: bool) {
        self.state.send(AppShellMobileEvent::SetOpen(open));
    }
}
#[derive(Clone, Copy, PartialEq)]
enum AppShellMobileState {
    Desktop,
    Mobile {
        /// Whether the drawer is open.
        open: bool,
    },
}
#[derive(Clone, Copy, PartialEq)]
enum AppShellMobileEvent {
    /// Whether the viewport is below the mobile breakpoint.
    Resize(bool),
    /// The caller's open binding changed.
    Sync(bool),
    /// Open or close the drawer.
    SetOpen(bool),
    /// Invert the drawer's open state.
    Toggle,
}
/// Readonly snapshot read for each viewport-width event.
struct MobileMachine {
    /// Whether the drawer binding is open when the width event arrives.
    open: bool,
}
impl Machine for MobileMachine {
    type State = AppShellMobileState;
    type Event = AppShellMobileEvent;
    /// The value to write to the caller's open binding.
    type Effect = bool;
    fn transition(
        &self,
        state: &AppShellMobileState,
        event: AppShellMobileEvent,
    ) -> Step<AppShellMobileState, AppShellMobileEvent, bool> {
        match (state, event) {
            (AppShellMobileState::Mobile { .. }, AppShellMobileEvent::Resize(true)) => Step::stay(),
            (_, AppShellMobileEvent::Resize(true)) => {
                Step::to(AppShellMobileState::Mobile { open: self.open })
            }
            (AppShellMobileState::Mobile { open }, AppShellMobileEvent::Resize(false)) => {
                let step = Step::to(AppShellMobileState::Desktop);
                if *open { step.with(false) } else { step }
            }
            (AppShellMobileState::Desktop, AppShellMobileEvent::Resize(false)) => Step::stay(),
            (AppShellMobileState::Mobile { open }, AppShellMobileEvent::Sync(next)) => {
                if *open == next {
                    Step::stay()
                } else {
                    Step::to(AppShellMobileState::Mobile { open: next })
                }
            }
            (AppShellMobileState::Desktop, AppShellMobileEvent::Sync(_)) => Step::stay(),
            (AppShellMobileState::Mobile { open }, AppShellMobileEvent::SetOpen(next)) => {
                if *open == next {
                    Step::stay()
                } else {
                    Step::to(AppShellMobileState::Mobile { open: next }).with(next)
                }
            }
            (AppShellMobileState::Desktop, AppShellMobileEvent::SetOpen(false)) => {
                Step::stay().with(false)
            }
            (AppShellMobileState::Desktop, AppShellMobileEvent::SetOpen(true)) => Step::stay(),
            (AppShellMobileState::Mobile { open }, AppShellMobileEvent::Toggle) => {
                let next = !*open;
                Step::to(AppShellMobileState::Mobile { open: next }).with(next)
            }
            (AppShellMobileState::Desktop, AppShellMobileEvent::Toggle) => Step::stay(),
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
            AppShellMobileState::Mobile {
                open: open.get_untracked(),
            }
        } else {
            AppShellMobileState::Desktop
        };
        let state = machine(
            initial,
            move || MobileMachine { open: open.get() },
            move |_, write: bool| {
                if open.get_untracked() != write {
                    open.set(write);
                }
            },
        );
        if let Some(mobile) = mobile {
            state.feed(mobile, AppShellMobileEvent::Resize);
        }
        state.feed(move || open.get(), AppShellMobileEvent::Sync);
        Self {
            context: Some(MobileContext {
                state,
                toggle: node_ref(),
                drawer: node_ref(),
                auto_toggle,
            }),
        }
    }
    /// Return whether the configured shell currently uses its mobile layout.
    pub fn is_mobile(self) -> bool {
        self.context.is_some_and(MobileContext::is_mobile)
    }
    /// Return whether the mobile-layout drawer is currently open.
    pub fn is_open(self) -> bool {
        self.context.is_some_and(MobileContext::is_open)
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
    /// Show the configured drawer; do nothing outside a shell or in the desktop layout.
    pub fn show(self) {
        if let Some(context) = self.context
            && context.is_mobile()
        {
            context.set_open(true);
        }
    }
    /// Request that the configured drawer close; do nothing outside a shell.
    pub fn close(self) {
        if let Some(context) = self.context {
            context.set_open(false);
        }
    }
    /// Invert the configured drawer's open state; do nothing outside a shell or in the desktop layout.
    pub fn toggle(self) {
        if let Some(context) = self.context
            && context.is_mobile()
        {
            context.state.send(AppShellMobileEvent::Toggle);
        }
    }
    /// The drawer's open state as a binding written back to the shell machine.
    pub(crate) fn open_binding(self) -> Option<Binding<bool>> {
        self.context.map(|context| {
            kivact::state::binding(
                move || context.is_open(),
                move |open| context.set_open(open),
            )
        })
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
    width: Value<Length>,
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
        width: (Length::Px(320.0)).into_value(),
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
    /// Set the drawer width as a layout length (`320.0` is 320 px).
    pub fn width(mut self, width: impl IntoValue<Length>) -> Self {
        self.width = width.into_value();
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

impl Element for MobileNav {
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
        kivact::view::scope(move || {
            let mobile = use_app_shell_mobile();
            let open = open
                .or_else(|| mobile.open_binding())
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

impl Element for MobileNavToggle {
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
        kivact::view::scope(move || {
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
    width: Value<Length>,
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
