//! Role: Arrange responsive Neutral navigation around application content.
use crate::layout::LayoutInset;
use crate::media_query::{Breakpoint, viewport_below};
use crate::mobile_nav::{AppShellMobile, mobile_nav, mobile_nav_toggle};
use crate::scrollable_area::scrollable_area;
use crate::stack::{ContainerInset, Space, rtl};
use crate::tokens;
use kivact::state::IntoBinding;
use kivact::state::Value;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::node;
use kivact::view::row;
use kivact::view::scroll;
use kivact::view::text;

use kivact::state::Binding;
use kivact::state::Callback;
use kivact::state::IntoValue;
use kivact::state::Machine;
use kivact::state::Step;
use kivact::state::callback;
use kivact::state::machine;
use kivact::state::provide;
use kivact::state::signal;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::auto;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::show;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShellVariant {
    Wash,
    Surface,
    Section,
    #[default]
    Elevated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShellHeight {
    #[default]
    Fill,
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
    try_use_context::<ShellNavMode>()
        .map(|value| value.get())
        .unwrap_or_default()
}

pub struct MobileNavConfig {
    has_toggle: bool,
    open: Option<Binding<bool>>,
    on_change: Option<Callback<bool>>,
    content: Option<ViewFn>,
    breakpoint: Option<Breakpoint>,
}

impl Default for MobileNavConfig {
    fn default() -> Self {
        Self {
            has_toggle: true,
            open: None,
            on_change: None,
            content: None,
            breakpoint: Some(Breakpoint::Md),
        }
    }
}

impl MobileNavConfig {
    pub fn has_toggle(mut self, has_toggle: bool) -> Self {
        self.has_toggle = has_toggle;
        self
    }
    pub fn open(mut self, open: impl IntoBinding<bool>) -> Self {
        self.open = Some(open.into_binding());
        self
    }
    pub fn on_open_change<R: HandlerResult>(
        mut self,
        change: impl Fn(bool) -> R + 'static,
    ) -> Self {
        self.on_change = Some(callback(change));
        self
    }
    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }
    pub fn breakpoint(mut self, breakpoint: impl Into<Option<Breakpoint>>) -> Self {
        self.breakpoint = breakpoint.into();
        self
    }
}

pub enum MobileNavMode {
    Disabled,
    Auto(MobileNavConfig),
    Custom(ViewFn),
}

impl Default for MobileNavMode {
    fn default() -> Self {
        Self::Auto(MobileNavConfig::default())
    }
}

impl MobileNavMode {
    pub fn custom<V: IntoView>(content: impl IntoViewFn<V>) -> Self {
        Self::Custom(content.into_view_fn())
    }
}

pub struct AppShell {
    root: Node,
    content: View,
    top: Option<ViewFn>,
    side: Option<ViewFn>,
    banner: Option<ViewFn>,
    padding: Value<Space>,
    height: Value<ShellHeight>,
    variant: Value<ShellVariant>,
    mobile: MobileNavMode,
}

pub fn app_shell(content: impl IntoView) -> AppShell {
    AppShell {
        root: node(),
        content: content.into_view(),
        top: None,
        side: None,
        banner: None,
        padding: (Space::S0).into_value(),
        height: (ShellHeight::Fill).into_value(),
        variant: (ShellVariant::Elevated).into_value(),
        mobile: MobileNavMode::default(),
    }
}

impl AppShell {
    pub fn top_nav<V: IntoView>(mut self, nav: impl IntoViewFn<V>) -> Self {
        self.top = Some(nav.into_view_fn());
        self
    }
    pub fn side_nav<V: IntoView>(mut self, nav: impl IntoViewFn<V>) -> Self {
        self.side = Some(nav.into_view_fn());
        self
    }
    pub fn banner<V: IntoView>(mut self, banner: impl IntoViewFn<V>) -> Self {
        self.banner = Some(banner.into_view_fn());
        self
    }
    pub fn content_padding(mut self, padding: impl IntoValue<Space>) -> Self {
        self.padding = padding.into_value();
        self
    }
    pub fn height(mut self, height: impl IntoValue<ShellHeight>) -> Self {
        self.height = height.into_value();
        self
    }
    pub fn variant(mut self, variant: impl IntoValue<ShellVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }
    pub fn mobile_nav(mut self, mobile: MobileNavMode) -> Self {
        self.mobile = mobile;
        self
    }
}

impl NodeBuilder for AppShell {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for AppShell {
    fn into_view(self) -> View {
        let Self {
            root,
            content,
            top,
            side,
            banner,
            padding,
            height,
            variant,
            mobile,
        } = self;
        provide((), move || {
            let configured = !matches!(mobile, MobileNavMode::Disabled);
            let automatic = matches!(mobile, MobileNavMode::Auto(_));
            let has_top = top.is_some();
            let has_side = side.is_some();
            let has_banner = banner.is_some();
            let has_nav = has_top || has_side;
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
                let value = config.open.unwrap_or_else(|| signal(false).into_binding());
                Some(kivact::state::binding(
                    move || value.get(),
                    move |open| {
                        if open != value.get() {
                            value.set(open);
                            if let Some(change) = config.on_change {
                                change.call(open);
                            }
                        }
                    },
                ))
            } else {
                None
            };
            let auto_toggle = automatic && config.has_toggle;
            let mobile = AppShellMobile::configured(mobile_width, open, auto_toggle);
            let shell = provide(mobile, move || {
                let main_ref = node_ref();
                let main_focused = main_ref.focused();
                let skip = machine(
                    SkipFocus::Idle,
                    || SkipMachine,
                    move |_, ()| main_ref.focus(),
                );
                skip.feed(main_focused, SkipEvent::Focus);
                let skip_requested =
                    kivact::state::memo(move || skip.get() == SkipFocus::Requested);
                let skip_ref = node_ref();
                let skip_focused = skip_ref.focused();
                let label =
                    kivact::i18n::message("soft.app_shell.skip_to_content", "Skip to content");
                let skip = node()
                    .with_node(move |node| node.role(Role::Link))
                    .name(label)
                    .focusable(true)
                    .node_ref(skip_ref)
                    .on_press(move || skip.send(SkipEvent::Activate))
                    .style(move || {
                        let t = tokens();
                        if skip_focused.get() {
                            let base = style()
                                .position(Position::Fixed)
                                .top(t.spacing.s2)
                                .z_index(9999)
                                .width(auto())
                                .height(auto())
                                .padding_y(t.spacing.s2)
                                .padding_x(t.spacing.s4)
                                .background(t.colors.bg_surface)
                                .color(t.colors.text_accent)
                                .font_size(t.typography.body.size_px)
                                .weight(t.typography.weight_semibold)
                                .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
                            if rtl() {
                                base.right(t.spacing.s2)
                            } else {
                                base.left(t.spacing.s2)
                            }
                        } else {
                            style()
                                .position(Position::Absolute)
                                .size(1, 1)
                                .opacity(0.0)
                                .clip(true)
                        }
                    })
                    .child(text(label));
                let header_ref = node_ref();
                let header_height = header_ref.rect();
                let desktop_top = top.clone();
                let mobile_top = top.clone();
                let bar_side = side.clone();
                let header = column((
                    banner.map(|banner| banner.call(())),
                    show(
                        move || !mobile.is_mobile(),
                        move || desktop_top.as_ref().map(|slot| slot.call(())),
                    ),
                    show(
                        move || mobile.is_mobile() && (has_top || auto_toggle && enabled),
                        move || {
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
                                title.map(|title| node().style(style().grow(1)).child(title)),
                                auto_toggle.then(mobile_nav_toggle),
                            ))
                            .style(|| {
                                let t = tokens();
                                style()
                                    .direction(if rtl() {
                                        Direction::RowReverse
                                    } else {
                                        Direction::Row
                                    })
                                    .align_items(Align::Center)
                                    .min_height(t.spacing.s12)
                                    .padding_x(t.spacing.s2)
                                    .justify_content(Justify::End)
                            })
                        },
                    ),
                ))
                .node_ref(header_ref)
                .style(move || {
                    let t = tokens();
                    let visible =
                        has_banner || has_top || (mobile.is_mobile() && auto_toggle && enabled);
                    let mut base = style()
                        .shrink(0)
                        .background(shell_background(variant.get()));
                    if visible && kivact::platform::capabilities().get().safe_area {
                        let safe = kivact::platform::safe_area().get();
                        base = base
                            .padding_top(safe.top)
                            .padding_left(safe.left)
                            .padding_right(safe.right);
                    }
                    if variant.get() == ShellVariant::Section && has_top {
                        base = base.border_bottom(t.sizes.border_width, t.colors.border);
                    }
                    if height.get() == ShellHeight::Auto {
                        base = base.position(Position::Sticky).top(0).z_index(1);
                    }
                    base
                });
                let desktop_side = side.clone();
                let sidebar = show(
                    move || has_side && !mobile.is_mobile(),
                    move || {
                        let nav = desktop_side.as_ref().map(|slot| slot.call(()));
                        node()
                            .style(move || {
                                let t = tokens();
                                let mut base =
                                    style().direction(Direction::Column).shrink(0).min_height(0);
                                if variant.get() == ShellVariant::Section {
                                    base = if rtl() {
                                        base.border_left(t.sizes.border_width, t.colors.border)
                                    } else {
                                        base.border_right(t.sizes.border_width, t.colors.border)
                                    };
                                }
                                if height.get() == ShellHeight::Auto {
                                    let top = header_height.get().map_or(0.0, |rect| rect.height);
                                    base = base
                                        .position(Position::Sticky)
                                        .top(top)
                                        .height(
                                            (kivact::platform::window_size().get().height - top)
                                                .max(0.0),
                                        )
                                        .clip(true);
                                }
                                base
                            })
                            .child(nav)
                    },
                );
                let main_style = move || {
                    let t = tokens();
                    let mut base = style()
                        .direction(Direction::Column)
                        .grow(1)
                        .min_height(0)
                        .min_width(0);
                    if variant.get() == ShellVariant::Elevated {
                        base = base.background(t.colors.bg_surface);
                        if has_top && has_side && !mobile.is_mobile() {
                            base = if rtl() {
                                base.radius_top_right(t.radius.page)
                            } else {
                                base.radius_top_left(t.radius.page)
                            };
                        }
                    }
                    base.focus_visible(style().ring_with_offset(
                        t.sizes.focus_ring_width,
                        -t.sizes.focus_ring_width,
                        t.colors.accent,
                    ))
                };
                let main_name = kivact::i18n::message("soft.app_shell.main", "Main content");
                let padding_px = padding.get().px();
                let inset = ContainerInset::uniform(padding_px);
                let content = provide(inset, move || {
                    provide(LayoutInset::from_padding(inset), move || content)
                });
                let contents = column(content).style(move || {
                    let mut base = style().padding(padding_px);
                    if kivact::platform::capabilities().get().safe_area {
                        base = base.padding_bottom(
                            padding_px + kivact::platform::safe_area().get().bottom,
                        );
                    }
                    base
                });
                let main = if height.get() == ShellHeight::Fill {
                    scrollable_area(main_name, contents)
                        .with_node(move |node| node.role(Role::Main))
                        .node_ref(main_ref)
                        .force_focusable(skip_requested.into_value())
                        .style(main_style)
                        .into_view()
                } else {
                    node()
                        .with_node(move |node| node.role(Role::Main))
                        .name(main_name)
                        .node_ref(main_ref)
                        .focusable(skip_requested)
                        .style(main_style)
                        .child(contents)
                        .into_view()
                };
                let middle = row((sidebar, main)).style(|| {
                    style()
                        .direction(if rtl() {
                            Direction::RowReverse
                        } else {
                            Direction::Row
                        })
                        .grow(1)
                        .min_width(0)
                        .min_height(0)
                });
                let drawer = show(
                    move || enabled && mobile.is_mobile(),
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
                );
                let body = column((header, middle)).style(style().min_height(pct(100)).grow(1));
                let body = if height.get() == ShellHeight::Auto {
                    scroll(body)
                        .style(style().grow(1).min_height(0))
                        .into_view()
                } else {
                    body.into_view()
                };
                (skip, body, drawer)
            });
            root.with_node(move |node| {
                node.base_style(move || {
                    let viewport = kivact::platform::window_size().get().height;
                    let base = style()
                        .position(Position::Relative)
                        .direction(Direction::Column)
                        .clip(true)
                        .background(shell_background(variant.get()));
                    if viewport > 0.0 {
                        base.height(viewport)
                    } else {
                        base.height(pct(100))
                    }
                })
            })
            .child(shell)
        })
    }
}

fn shell_background(variant: ShellVariant) -> kivact::style::Color {
    let colors = tokens().colors;
    match variant {
        ShellVariant::Wash | ShellVariant::Elevated => colors.bg_body,
        ShellVariant::Surface | ShellVariant::Section => colors.bg_surface,
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

kivact::state::constant_value!(ShellHeight, ShellVariant);
