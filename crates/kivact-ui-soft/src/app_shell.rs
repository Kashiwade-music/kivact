//! Role: Arrange responsive Neutral navigation around application content.
use crate::color as c;
use crate::layout::LayoutInset;
use crate::media_query::{Breakpoint, viewport_below};
use crate::metrics as m;
use crate::mobile_nav::{AppShellMobile, mobile_nav, mobile_nav_toggle};
use crate::radius as r;
use crate::recipe_parts::ring;
use crate::scrollable_area::scrollable_area;
use crate::space as s;
use crate::space::Space;
use crate::stack::{ContainerInset, rtl};
use crate::type_scale as t;

use kivact::Prop;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::IntoValue;
use kivact::state::Machine;
use kivact::state::Signal;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::machine;
use kivact::state::memo;
use kivact::state::provide;
use kivact::state::signal;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::auto;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::sidebar_provider;
use kivact::ui::theme::Variant;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::scroll;
use kivact::view::show;
use kivact::view::text;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
/// Background and boundary treatment for the application shell.
pub enum ShellVariant {
    /// Use the body background around the shell.
    Wash,
    /// Use the surface background.
    Surface,
    /// Use the surface background with navigation boundary dividers.
    Section,
    #[default]
    /// Use a body background with a surface main panel; the default.
    Elevated,
}

impl Variant for ShellVariant {}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
/// Scrolling strategy inside the viewport-height application shell.
pub enum ShellHeight {
    #[default]
    /// Fill the parent and scroll the main content; the default.
    Fill,
    /// Scroll the whole shell body, keeping navigation sticky.
    Auto,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ShellNavMode {
    #[default]
    Desktop,
    MobileBar,
    Drawer,
}

pub(crate) fn shell_nav_mode() -> ShellNavMode {
    try_use_context::<ShellNavMode>().unwrap_or_default()
}

/// Automatic mobile navigation configuration; defaults to a toggle and the Md breakpoint.
pub struct MobileNavConfig {
    has_toggle: bool,
    open: Option<Signal<bool>>,
    content: Option<ViewFn>,
    breakpoint: Option<Breakpoint>,
}

impl Default for MobileNavConfig {
    fn default() -> Self {
        Self {
            has_toggle: true,
            open: None,
            content: None,
            breakpoint: Some(Breakpoint::Md),
        }
    }
}

impl MobileNavConfig {
    /// Include the automatic mobile navigation toggle; enabled by default.
    pub fn has_toggle(mut self, has_toggle: bool) -> Self {
        self.has_toggle = has_toggle;
        self
    }
    /// Bind drawer openness to caller state; omission creates initially closed local state.
    pub fn open(mut self, open: Signal<bool>) -> Self {
        self.open = Some(open);
        self
    }
    /// Supply a reusable mobile drawer content factory instead of the default navigation content.
    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }
    /// Choose the mobile width threshold; defaults to Md, while None disables width switching.
    pub fn breakpoint(mut self, breakpoint: impl Into<Option<Breakpoint>>) -> Self {
        self.breakpoint = breakpoint.into();
        self
    }
}

/// Mobile navigation policy; automatic navigation is the default.
pub enum MobileNavMode {
    /// Disable mobile navigation and viewport-driven switching.
    Disabled,
    /// Generate mobile navigation using the supplied configuration.
    Auto(MobileNavConfig),
    /// Render caller-supplied mobile navigation from a reusable factory.
    Custom(ViewFn),
}

impl Default for MobileNavMode {
    fn default() -> Self {
        Self::Auto(MobileNavConfig::default())
    }
}

impl MobileNavMode {
    /// Use a reusable custom mobile navigation factory, evaluated in the shell’s mobile context.
    pub fn custom<V: IntoView>(content: impl IntoViewFn<V>) -> Self {
        Self::Custom(content.into_view_fn())
    }
}

/// Responsive application frame with navigation, main content and a skip-to-content link.
///
/// The look is the element's `.variant(ShellVariant::..)`; elevated by default. The shell is
/// fill-height with zero content padding and automatic mobile navigation by default.
/// `content_padding` and `height` are read at view construction, `top_nav`, `sidebar` and
/// `banner` supply the navigation and banner factories, and `mobile_nav` chooses disabled,
/// automatic or custom mobile navigation. `sidebar` returns the sidebar's content (its menus and
/// groups). `sidebar_open` gives the sidebar's open state; without it the shell keeps its own,
/// open at first.
#[component]
pub fn app_shell(
    content: View,
    top_nav: Option<ViewFn>,
    sidebar: Option<ViewFn>,
    banner: Option<ViewFn>,
    sidebar_open: Option<Signal<bool>>,
    #[prop(default = Space::S0)] content_padding: Value<Space>,
    #[prop(default = ShellHeight::Fill)] height: Value<ShellHeight>,
    #[prop(default = MobileNavMode::default())] mobile_nav: MobileNavMode,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let variant = memo(move || variants.get::<ShellVariant>().unwrap_or_default()).into_value();
    let open = sidebar_open.unwrap_or_else(|| signal(true));
    let (mobile, config, custom) =
        configure_mobile(mobile_nav, top_nav.is_some() || sidebar.is_some());
    let presentation = ShellPresentation {
        mobile,
        height,
        variant,
        has_top: top_nav.is_some(),
        has_side: sidebar.is_some(),
        has_banner: banner.is_some(),
    };
    let shell = provide(mobile, move || {
        let (skip, main_ref, skip_requested) = skip_link();
        let header_ref = node_ref();
        let header = presentation.header(top_nav.clone(), sidebar.clone(), banner, header_ref);
        let side = presentation
            .sidebar(sidebar.clone(), header_ref)
            .into_view();
        let main = presentation.main(content, content_padding, main_ref, skip_requested);
        let middle = sidebar_provider(open, side, main).style(|| {
            style()
                .flex_direction(if rtl() {
                    FlexDirection::RowReverse
                } else {
                    FlexDirection::Row
                })
                .flex_grow(1)
                .min_width(0)
                .min_height(0)
        });
        let drawer = presentation.drawer(config, custom, top_nav, sidebar);
        let body = column((header, middle)).style(style().min_height(pct(100)).flex_grow(1));
        let body = if height.get() == ShellHeight::Auto {
            scroll(body)
                .style(style().flex_grow(1).min_height(0))
                .into_view()
        } else {
            body.into_view()
        };
        (skip, body, drawer)
    });
    node()
        .with_node(move |node| node.base_style(move || shell_style(variant.get())))
        .child(shell)
}

fn shell_background(variant: ShellVariant) -> kivact::style::Color {
    match variant {
        ShellVariant::Wash | ShellVariant::Elevated => c::BG_BODY,
        ShellVariant::Surface | ShellVariant::Section => c::BG_SURFACE,
    }
}

#[derive(Clone, PartialEq)]
enum SkipFocus {
    Idle,
    Requested,
}
enum SkipEvent {
    Activate,
    Focus(bool),
}
struct SkipMachine;
impl Machine for SkipMachine {
    type State = SkipFocus;
    type Event = SkipEvent;
    type Effect = ();
    fn transition(&self, _: &SkipFocus, event: SkipEvent) -> Step<SkipFocus, SkipEvent, ()> {
        match event {
            SkipEvent::Activate => Step::to(SkipFocus::Requested).with(()),
            SkipEvent::Focus(false) => Step::to(SkipFocus::Idle),
            SkipEvent::Focus(true) => Step::stay(),
        }
    }
}

#[derive(Clone, Copy)]
struct ShellPresentation {
    mobile: AppShellMobile,
    height: Value<ShellHeight>,
    variant: Value<ShellVariant>,
    has_top: bool,
    has_side: bool,
    has_banner: bool,
}
fn configure_mobile(
    mobile: MobileNavMode,
    has_nav: bool,
) -> (AppShellMobile, MobileNavConfig, Option<ViewFn>) {
    let configured = !matches!(mobile, MobileNavMode::Disabled);
    let automatic = matches!(mobile, MobileNavMode::Auto(_));
    let (config, custom) = match mobile {
        MobileNavMode::Auto(config) => (config, None),
        MobileNavMode::Custom(custom) => (MobileNavConfig::default(), Some(custom)),
        MobileNavMode::Disabled => (MobileNavConfig::default(), None),
    };
    let enabled = configured && (has_nav || config.content.is_some() || custom.is_some());
    let mobile_width = if configured {
        config.breakpoint.map(viewport_below)
    } else {
        None
    };
    let open = if enabled {
        let value = config.open.unwrap_or_else(|| signal(false));
        Some(kivact::state::binding(
            move || value.get(),
            move |open| {
                if open != value.get() {
                    value.set(open);
                }
            },
        ))
    } else {
        None
    };
    let auto_toggle = automatic && config.has_toggle;
    let mobile = AppShellMobile::configured(mobile_width, open, auto_toggle);
    (mobile, config, custom)
}
fn skip_link() -> (Node, kivact::view::NodeRef, Value<bool>) {
    let main_ref = node_ref();
    let main_focused = main_ref.focused();
    let skip = machine(
        SkipFocus::Idle,
        || SkipMachine,
        move |_, ()| main_ref.focus(),
    );
    skip.feed(main_focused, SkipEvent::Focus);
    let skip_requested = kivact::state::memo(move || skip.get() == SkipFocus::Requested);
    let skip_ref = node_ref();
    let skip_focused = skip_ref.focused();
    let label = kivact::i18n::message("soft.app_shell.skip_to_content", "Skip to content");
    let skip = node()
        .with_node(move |node| node.role(Role::Link))
        .name(label)
        .focusable(true)
        .node_ref(skip_ref)
        .on_press(move || skip.send(SkipEvent::Activate))
        .style(move || {
            if skip_focused.get() {
                let base = style()
                    .position(Position::Fixed)
                    .top(s::SPACE_S2)
                    .z_index(9999)
                    .width(auto())
                    .height(auto())
                    .padding_y(s::SPACE_S2)
                    .padding_x(s::SPACE_S4)
                    .background(c::BG_SURFACE)
                    .color(c::TEXT_ACCENT)
                    .font_size(t::BODY.size_px)
                    .font_weight(t::WEIGHT_SEMIBOLD)
                    .focus_visible(ring(c::ACCENT));
                if rtl() {
                    base.right(s::SPACE_S2)
                } else {
                    base.left(s::SPACE_S2)
                }
            } else {
                style()
                    .position(Position::Absolute)
                    .size(1, 1)
                    .opacity(0.0)
                    .clip(true)
            }
        })
        .child(text(label).test_id("kivact.sr-only"));
    (skip, main_ref, skip_requested.into_value())
}
fn mobile_bar(
    mobile_top: Option<ViewFn>,
    bar_side: Option<ViewFn>,
    auto_toggle: bool,
) -> impl IntoView {
    let title = if let Some(slot) = &mobile_top {
        let slot = slot.clone();
        Some(provide(ShellNavMode::MobileBar, move || slot.call(())))
    } else {
        bar_side.as_ref().map(|slot| {
            let slot = slot.clone();
            provide(ShellNavMode::MobileBar, move || slot.call(()))
        })
    };
    row((
        title.map(|title| node().style(style().flex_grow(1)).child(title)),
        auto_toggle.then(mobile_nav_toggle),
    ))
    .style(|| {
        style()
            .flex_direction(if rtl() {
                FlexDirection::RowReverse
            } else {
                FlexDirection::Row
            })
            .align_items(Align::Center)
            .min_height(s::SPACE_S12)
            .padding_x(s::SPACE_S2)
            .justify_content(Justify::End)
    })
}
impl ShellPresentation {
    fn header(
        self,
        top: Option<ViewFn>,
        side: Option<ViewFn>,
        banner: Option<ViewFn>,
        header_ref: kivact::view::NodeRef,
    ) -> impl IntoView {
        let Self {
            mobile,
            height,
            variant,
            has_top,
            has_banner,
            ..
        } = self;
        let desktop_top = top.clone();
        let mobile_top = top.clone();
        let bar_side = side.clone();
        column((
            banner.map(|banner| banner.call(())),
            show(
                move || !mobile.is_mobile(),
                move || desktop_top.as_ref().map(|slot| slot.call(())),
            ),
            show(
                move || {
                    mobile.is_mobile()
                        && (has_top || mobile.has_auto_toggle() && mobile.is_enabled())
                },
                move || {
                    mobile_bar(
                        mobile_top.clone(),
                        bar_side.clone(),
                        mobile.has_auto_toggle(),
                    )
                },
            ),
        ))
        .node_ref(header_ref)
        .style(move || {
            let visible = has_banner
                || has_top
                || (mobile.is_mobile() && mobile.has_auto_toggle() && mobile.is_enabled());
            let mut base = style()
                .flex_shrink(0)
                .background(shell_background(variant.get()));
            if visible && kivact::platform::capabilities().get().safe_area {
                let safe = kivact::platform::safe_area().get();
                base = base
                    .padding_top(safe.top)
                    .padding_left(safe.left)
                    .padding_right(safe.right);
            }
            if variant.get() == ShellVariant::Section && has_top {
                base = base.border_bottom(m::BORDER_WIDTH, c::BORDER);
            }
            if height.get() == ShellHeight::Auto {
                base = base.position(Position::Sticky).top(0).z_index(1);
            }
            base
        })
    }
    /// The desktop sidebar: the central sidebar's panel with the caller's content, sticky in
    /// the auto-height shell. Its edge is the panel's own border.
    fn sidebar(self, nav: Option<ViewFn>, header_ref: kivact::view::NodeRef) -> impl IntoView {
        let Self {
            mobile,
            height,
            has_side,
            ..
        } = self;
        let header_height = header_ref.rect();
        show(
            move || has_side && !mobile.is_mobile(),
            move || {
                let content = nav.as_ref().map(|slot| slot.call(()));
                kivact::ui::sidebar().content(content).style(move || {
                    if height.get() != ShellHeight::Auto {
                        return style();
                    }
                    let top = header_height.get().map_or(0.0, |rect| rect.height);
                    style()
                        .position(Position::Sticky)
                        .top(top)
                        .height((kivact::platform::window_size().get().height - top).max(0.0))
                        .clip(true)
                })
            },
        )
    }
    fn main_style(self) -> kivact::style::Style {
        let Self {
            mobile,
            variant,
            has_top,
            has_side,
            ..
        } = self;

        let mut base = style()
            .flex_direction(FlexDirection::Column)
            .flex_grow(1)
            .min_height(0)
            .min_width(0);
        if variant.get() == ShellVariant::Elevated {
            base = base.background(c::BG_SURFACE);
            if has_top && has_side && !mobile.is_mobile() {
                base = if rtl() {
                    base.border_top_right_radius(r::RADIUS_PAGE)
                } else {
                    base.border_top_left_radius(r::RADIUS_PAGE)
                };
            }
        }
        base.focus_visible(
            style()
                .outline(m::FOCUS_RING_WIDTH, c::ACCENT)
                .outline_offset(-m::FOCUS_RING_WIDTH),
        )
    }
    fn main(
        self,
        content: View,
        padding: Value<Space>,
        main_ref: kivact::view::NodeRef,
        skip_requested: Value<bool>,
    ) -> View {
        let presentation = self;
        let height = self.height;
        let main_name = kivact::i18n::message("soft.app_shell.main", "Main content");
        let padding_px = padding.get().px();
        let inset = ContainerInset::uniform(padding_px);
        let content = provide(inset, move || {
            provide(LayoutInset::from_padding(inset), move || content)
        });
        let contents = column(content).style(move || {
            let mut base = style().padding(padding_px);
            if kivact::platform::capabilities().get().safe_area {
                base = base.padding_bottom(padding_px + kivact::platform::safe_area().get().bottom);
            }
            base
        });
        if height.get() == ShellHeight::Fill {
            scrollable_area(main_name, contents)
                .with_node(move |node| node.role(Role::Main))
                .node_ref(main_ref)
                .force_focusable(skip_requested)
                .style(move || presentation.main_style())
                .into_view()
        } else {
            node()
                .with_node(move |node| node.role(Role::Main))
                .name(main_name)
                .node_ref(main_ref)
                .focusable(skip_requested)
                .style(move || presentation.main_style())
                .child(contents)
                .into_view()
        }
    }
    fn drawer(
        self,
        config: MobileNavConfig,
        custom: Option<ViewFn>,
        top: Option<ViewFn>,
        side: Option<ViewFn>,
    ) -> impl IntoView {
        let Self { mobile, .. } = self;
        show(
            move || mobile.is_enabled() && mobile.is_mobile(),
            move || {
                if let Some(custom) = &custom {
                    return custom.call(());
                }
                let content = config.content.clone();
                let top = top.clone();
                let side = side.clone();
                mobile_nav(move || {
                    let content = content.clone();
                    let top = top.clone();
                    let side = side.clone();
                    provide(ShellNavMode::Drawer, move || {
                        if let Some(content) = content {
                            content.call(())
                        } else {
                            column((
                                side.map(|slot| slot.call(())),
                                top.map(|slot| slot.call(())),
                            ))
                            .into_view()
                        }
                    })
                })
                .into_view()
            },
        )
    }
}
fn shell_style(variant: ShellVariant) -> kivact::style::Style {
    let viewport = kivact::platform::window_size().get().height;
    let base = style()
        .position(Position::Relative)
        .flex_direction(FlexDirection::Column)
        .clip(true)
        .background(shell_background(variant));
    if viewport > 0.0 {
        base.height(viewport)
    } else {
        base.height(pct(100))
    }
}
