//! Role: Render the responsive Neutral navigation drawer and its toggle.
use crate::color as c;
use crate::layout::LayoutInset;
use crate::metrics as m;
use crate::motion::{EASE_STANDARD, MEDIUM};
use crate::space as s;
use crate::stack::{ContainerInset, rtl};
use crate::type_scale as t;

use kivact::interaction::use_dialog;
use kivact::motion::Transition;
use kivact::state::Signal;
use kivact::view::Element;
use kivact::view::Node;
use kivact::view::ViewFn;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::scroll;
use kivact::view::text;

use kivact::Prop;
use kivact::component;
use kivact::geometry::Edge;
use kivact::interaction::DialogMode;
use kivact::state::Machine;
use kivact::state::MachineHandle;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::machine;
use kivact::state::provide;
use kivact::state::signal;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::FlexDirection;
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
// xtask-allow(state): independent: open is the drawer's open binding, written by the toggle, the drawer's close and the backdrop; auto_toggle is fixed when the context is made
#[derive(Clone, Copy)]
struct MobileContext {
    layout: MachineHandle<LayoutMachine>,
    open: Signal<bool>,
    toggle: NodeRef,
    drawer: NodeRef,
    auto_toggle: bool,
}
impl MobileContext {
    fn is_mobile(self) -> bool {
        self.layout.get() == Layout::Mobile
    }
    fn is_open(self) -> bool {
        self.is_mobile() && self.open.get()
    }
    /// Closing always applies; opening only in the mobile layout.
    fn set_open(self, open: bool) {
        if !open || self.is_mobile() {
            self.open.set(open);
        }
    }
}
/// Whether the shell uses its mobile layout.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Layout {
    Desktop,
    Mobile,
}
/// The layout follows the viewport; leaving the mobile layout closes the drawer.
struct LayoutMachine;

impl Machine for LayoutMachine {
    type State = Layout;
    /// Whether the viewport is below the mobile breakpoint.
    type Event = bool;
    /// Close the drawer.
    type Effect = ();
    fn transition(&self, state: &Layout, mobile: bool) -> Step<Layout, bool, ()> {
        match (state, mobile) {
            (Layout::Desktop, true) => Step::to(Layout::Mobile),
            (Layout::Mobile, false) => Step::to(Layout::Desktop).with(()),
            _ => Step::stay(),
        }
    }
}
/// Read mobile navigation from the surrounding app shell, or an inert default outside one.
pub fn use_app_shell_mobile() -> AppShellMobile {
    try_use_context::<AppShellMobile>().unwrap_or_default()
}
impl AppShellMobile {
    pub(crate) fn configured(
        mobile: Option<Value<bool>>,
        open: Option<Signal<bool>>,
        auto_toggle: bool,
    ) -> Self {
        let Some(open) = open else {
            return Self::default();
        };
        let initial = if mobile.is_some_and(|mobile| mobile.get_untracked()) {
            Layout::Mobile
        } else {
            Layout::Desktop
        };
        let layout = machine(
            initial,
            || LayoutMachine,
            move |_, ()| {
                if open.get_untracked() {
                    open.set(false);
                }
            },
        );
        if let Some(mobile) = mobile {
            layout.feed(mobile, |mobile| mobile);
        }
        Self {
            context: Some(MobileContext {
                layout,
                open,
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
    /// Hide the configured drawer; do nothing outside a shell.
    pub fn hide(self) {
        if let Some(context) = self.context {
            context.set_open(false);
        }
    }
    /// Invert the configured drawer's open state; do nothing outside a shell or in the desktop layout.
    pub fn toggle(self) {
        if let Some(context) = self.context
            && context.is_mobile()
        {
            context.set_open(!context.open.get_untracked());
        }
    }
    /// The drawer's open state as a binding written back to the shell machine.
    pub(crate) fn open_binding(self) -> Option<Signal<bool>> {
        self.context.map(|context| {
            kivact::state::binding(
                move || context.is_open(),
                move |open| context.set_open(open),
            )
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
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
///
/// `open` binds the drawer to caller state; omitted, it uses the surrounding app shell's
/// binding. `name` is the accessible name of the drawer's overlay panel (the header text is
/// used when it is omitted), and `header` is the plain-text heading inside the panel.
#[component]
pub fn mobile_nav(
    content: ViewFn,
    open: Option<Signal<bool>>,
    #[prop(default = Length::Px(320.0))] width: Value<Length>,
    #[prop(default = DrawerSide::Auto)] side: Value<DrawerSide>,
    name: Option<Value<String>>,
    header: Option<Value<String>>,
    header_view: Option<ViewFn>,
) -> impl Element {
    let mobile = use_app_shell_mobile();
    let open = open
        .or_else(|| mobile.open_binding())
        .unwrap_or_else(|| signal(false));
    let anchor = mobile
        .context
        .map(|context| context.toggle)
        .unwrap_or_else(node_ref);
    let drawer = mobile.drawer_ref().unwrap_or_else(node_ref);
    let fallback = kivact::i18n::message("soft.mobile_nav.navigation", "Navigation");
    let label = name.or(header).unwrap_or(fallback);
    let behavior = use_dialog(open, DialogMode::Drawer(Edge::Right));
    node().child(show(behavior.is_open(), move || {
        let right = match side.get() {
            DrawerSide::Start => rtl(),
            DrawerSide::End => !rtl(),
            DrawerSide::Auto => anchor.rect().get().map_or(!rtl(), |rect| {
                rect.x + rect.width / 2.0 >= kivact::platform::window_size().get().width / 2.0
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
}

/// An app-shell drawer toggle with an optional custom icon.
///
/// It renders inside an app shell with mobile navigation and is hidden on the desktop layout.
/// `label` is the accessible label of the toggle's button, and `icon` replaces the menu glyph.
#[component]
pub fn mobile_nav_toggle(label: Option<Value<String>>, icon: Option<View>) -> impl Element {
    let mobile = use_app_shell_mobile();
    let trigger = mobile.is_enabled().then(|| {
        let label = label
            .unwrap_or_else(|| kivact::i18n::message("soft.mobile_nav.open", "Open navigation"));
        let mut trigger = button(label)
            .variant(ButtonVariant::Ghost)
            .size(ControlSize::Medium)
            .icon(icon.unwrap_or_else(|| kivact::view::icon("menu").into_view()))
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
        trigger
    });
    node().hidden(move || !mobile.is_mobile()).child(trigger)
}

fn drawer_scrim() -> Node {
    node().style(|| {
        let hidden = style().opacity(0.0);
        style()
            .position(Position::Fixed)
            .top(0)
            .bottom(0)
            .left(0)
            .right(0)
            .background(c::OVERLAY_SCRIM)
            .backdrop_blur(2)
            .enter(hidden.clone())
            .exit(hidden)
            .transition(
                TransitionProp::All,
                Transition::tween(MEDIUM as f64, EASE_STANDARD),
            )
    })
}
fn drawer_header(
    header_view: Option<&ViewFn>,
    header: Option<Value<String>>,
    behavior: kivact::interaction::DialogApi,
    open: Signal<bool>,
) -> impl IntoView {
    let title = if let Some(header_view) = header_view {
        Some(header_view.call(()))
    } else {
        header.map(|label| {
            text(label)
                .with_node(move |node| node.part(behavior.title()))
                .style(|| {
                    style()
                        .font_size(t::HEADING2.size_px)
                        .line_height(t::HEADING2.line_height_px / t::HEADING2.size_px)
                        .font_weight(t::HEADING2.weight)
                        .color(c::TEXT_PRIMARY)
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
        style()
            .flex_direction(if rtl() {
                FlexDirection::RowReverse
            } else {
                FlexDirection::Row
            })
            .min_height(s::SPACE_S12)
            .padding_x(s::SPACE_S2)
            .align_items(Align::Center)
            .flex_shrink(0)
            .justify_content(if has_title {
                Justify::SpaceBetween
            } else {
                Justify::End
            })
            .border_bottom(m::BORDER_WIDTH, c::BORDER)
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
                    .flex_direction(FlexDirection::Column)
                    .padding(s::SPACE_S2)
            })
            .child(contents),
    )
    .style(style().flex_grow(1).min_height(0))
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
            let offset = if right { pct(100) } else { pct(-100) };
            let hidden = style().translate_x(offset);
            let base = style()
                .position(Position::Fixed)
                .top(0)
                .bottom(0)
                .width(width.get())
                .max_width(pct(85))
                .clip(true)
                .flex_direction(FlexDirection::Column)
                .background(c::BG_SURFACE)
                .enter(hidden.clone())
                .exit(hidden)
                .transition(
                    TransitionProp::All,
                    Transition::tween(MEDIUM as f64, EASE_STANDARD),
                );
            if right {
                base.right(0).border_left(m::BORDER_WIDTH, c::BORDER)
            } else {
                base.left(0).border_right(m::BORDER_WIDTH, c::BORDER)
            }
        })
        .child((header, body))
}
