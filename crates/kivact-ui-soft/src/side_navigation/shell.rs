//! Role: Render the sidebar shell and collapse control.

use super::context::{SideNavCx, nav_cx};
use super::items::logical_row;

use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::mobile_nav::use_app_shell_mobile;
use crate::stack::rtl;

use kivact::i18n::message;
use kivact::interaction::{ScalarRange, use_splitter};
use kivact::motion::Spring;
use kivact::motion::Transition;
use kivact::motion::animated;
use kivact::state::Binding;
use kivact::state::IntoBinding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::provide;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::auto;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::tooltip;
use kivact::view::IntoView;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::Orientation;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::scroll;

/// Ownership and initial state of a sidebar's collapsed rail.
pub enum SideNavCollapse {
    /// Keep rail collapse state locally, starting from `default`.
    Uncontrolled {
        /// Whether the sidebar starts in its collapsed rail state.
        default: bool,
    },
    /// Read and write the application's rail collapse binding.
    Controlled(Binding<bool>),
}

// xtask-allow(state-fields): Construction metrics and optional caller collapse binding; the default is used only without that binding and is not a runtime lifecycle flag.
/// Initial width, width bounds and optional collapse binding for a resizable sidebar.
/// `Default` starts expanded at 260 px, bounded to 180–480 px.
pub struct SideNavResize {
    default_width: f64,
    min: f64,
    max: f64,
    default_collapsed: bool,
    collapsed: Option<Binding<bool>>,
}

impl Default for SideNavResize {
    fn default() -> Self {
        Self {
            default_width: 260.0,
            min: 180.0,
            max: 480.0,
            default_collapsed: false,
            collapsed: None,
        }
    }
}

impl SideNavResize {
    /// Set the initial expanded width in pixels.
    pub fn width(mut self, width: impl Into<f64>) -> Self {
        self.default_width = width.into();
        self
    }
    /// Set the allowed expanded width range in pixels.
    pub fn range(mut self, min: impl Into<f64>, max: impl Into<f64>) -> Self {
        self.min = min.into();
        self.max = max.into();
        self
    }
    /// Start an uncontrolled sidebar collapsed.
    pub fn initially_collapsed(mut self) -> Self {
        self.default_collapsed = true;
        self
    }
    /// Use the caller's single collapse binding.
    pub fn collapsed(mut self, collapsed: impl IntoBinding<bool>) -> Self {
        self.collapsed = Some(collapsed.into_binding());
        self
    }
}

/// A sidebar with a scrollable entry region and retained header and footer slots.
pub struct SideNav {
    root: Node,
    children: View,
    header: Option<View>,
    top: Option<View>,
    footer: Option<View>,
    icons: Option<View>,
    collapse: Option<SideNavCollapse>,
    resize: Option<SideNavResize>,
    collapse_button: Value<bool>,
}

/// Mount sidebar entries once within the shared sidebar context.
pub fn side_nav(children: impl IntoView) -> SideNav {
    SideNav {
        root: node()
            .with_node(move |node| node.role(Role::Navigation))
            .name(message("soft.side_nav.label", "Edge navigation")),
        children: children.into_view(),
        header: None,
        top: None,
        footer: None,
        icons: None,
        collapse: None,
        resize: None,
        collapse_button: (true).into_value(),
    }
}

impl SideNav {
    /// Mount header content once in the sidebar context.
    pub fn header(mut self, header: impl IntoView) -> Self {
        self.header = Some(header.into_view());
        self
    }
    /// Mount content above the scrollable navigation entries.
    pub fn top_content(mut self, content: impl IntoView) -> Self {
        self.top = Some(content.into_view());
        self
    }
    /// Mount footer content outside the scrollable entries.
    pub fn footer(mut self, footer: impl IntoView) -> Self {
        self.footer = Some(footer.into_view());
        self
    }
    /// Mount compact actions that remain available in the collapsed rail.
    pub fn footer_icons(mut self, icons: impl IntoView) -> Self {
        self.icons = Some(icons.into_view());
        self
    }
    /// Choose local or application-owned rail collapse state.
    pub fn collapsible(mut self, collapse: SideNavCollapse) -> Self {
        self.collapse = Some(collapse);
        self
    }
    /// Show or hide the sidebar's built-in collapse toggle.
    pub fn collapse_button(mut self, show: impl IntoValue<bool>) -> Self {
        self.collapse_button = show.into_value();
        self
    }
    /// Enable resizing with the supplied initial width, bounds and collapse ownership.
    pub fn resizable(mut self, resize: SideNavResize) -> Self {
        self.resize = Some(resize);
        self
    }
}

impl NodeBuilder for SideNav {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SideNav {
    fn into_view(self) -> View {
        let Self {
            root,
            children,
            header,
            top,
            footer,
            icons,
            collapse,
            resize,
            collapse_button,
        } = self;
        crate::navigation_session::with_navigation_group(move || {
            let mode = shell_nav_mode();
            let root = root.with_node(move |node| {
                node.base_style(move || {
                    style()
                        .direction(if mode == ShellNavMode::MobileBar {
                            logical_row()
                        } else {
                            Direction::Column
                        })
                        .min_width(0)
                        .min_height(0)
                })
            });
            if mode == ShellNavMode::MobileBar {
                return root
                    .style(|| {
                        style()
                            .width(pct(100))
                            .align_items(Align::Center)
                            .justify_content(Justify::SpaceBetween)
                    })
                    .child((header.map(|slot| slot), icons.map(|slot| slot)));
            }
            let nav_ref = node_ref();
            let collapse = if mode == ShellNavMode::Desktop {
                collapse
            } else {
                None
            };
            let resize = if mode == ShellNavMode::Desktop {
                resize
            } else {
                None
            };
            let collapsed = resize
                .as_ref()
                .map(|resize| {
                    resize
                        .collapsed
                        .unwrap_or_else(|| signal(resize.default_collapsed).into_binding())
                })
                .or_else(|| {
                    collapse.map(|collapse| match collapse {
                        SideNavCollapse::Uncontrolled { default } => signal(default).into_binding(),
                        SideNavCollapse::Controlled(collapsed) => collapsed,
                    })
                });
            let expanded_width =
                signal(resize.as_ref().map_or(260.0, |resize| resize.default_width));
            let cx = SideNavCx {
                collapsed,
                ..SideNavCx::default()
            };
            let target = move || {
                if cx.rail() {
                    kivact::style::use_tokens::<crate::Tokens>().spacing.s12
                } else {
                    expanded_width.get()
                }
            };
            let animated_width = if resize.is_none() && collapsed.is_some() {
                Some(animated(
                    target,
                    Transition::spring(Spring {
                        stiffness: 157.91,
                        damping: 25.13,
                        mass: 1.0,
                    }),
                ))
            } else {
                None
            };
            let has_top = header.is_some() || top.is_some();
            let has_bottom =
                footer.is_some() || icons.is_some() || collapse_button.get() && collapsed.is_some();
            let controls = provide(cx, move || {
                let top =
                    column((header.map(|slot| slot), top.map(|slot| slot))).style(move || {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
                        let mut base = style().gap(t.spacing.s2).shrink(0);
                        if has_top && mode == ShellNavMode::Desktop {
                            base = base.padding(t.spacing.s2);
                        }
                        if cx.rail() {
                            base = base.align_items(Align::Center);
                        }
                        base
                    });
                let items = column(children).style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let mut base = style().gap(t.spacing.s0_5);
                    if mode == ShellNavMode::Desktop {
                        base = base
                            .padding_x(t.spacing.s2)
                            .padding_top(if has_top { t.spacing.s1 } else { t.spacing.s2 })
                            .padding_bottom(if has_bottom {
                                t.spacing.s1
                            } else {
                                t.spacing.s2
                            });
                    }
                    if cx.rail() {
                        base = base.align_items(Align::Center);
                    }
                    base
                });
                let items = if mode == ShellNavMode::Desktop {
                    scroll(items)
                        .style(style().grow(1).min_height(0))
                        .into_view()
                } else {
                    items.into_view()
                };
                let bottom = has_bottom.then(|| {
                    let controls = row((
                        (collapse_button.get() && collapsed.is_some())
                            .then(|| side_nav_collapse_button().size(ControlSize::Small)),
                        icons.map(|slot| slot),
                    ))
                    .style(move || {
                        style()
                            .direction(if cx.rail() {
                                Direction::ColumnReverse
                            } else {
                                logical_row()
                            })
                            .align_items(Align::Center)
                            .gap(kivact::style::use_tokens::<crate::Tokens>().spacing.s1)
                    });
                    column((
                        footer.map(|slot| node().hidden(move || cx.rail()).child(slot)),
                        controls,
                    ))
                    .style(move || {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
                        let mut base = style().margin_top(auto()).gap(t.spacing.s2).shrink(0);
                        if mode == ShellNavMode::Desktop {
                            base = base
                                .padding_x(t.spacing.s2)
                                .padding_bottom(t.spacing.s2)
                                .padding_top(if cx.rail() { 0.0 } else { t.spacing.s1 });
                        }
                        if cx.rail() {
                            base = base.align_items(Align::Center);
                        }
                        base
                    })
                });
                (top, items, bottom)
            });
            let root = root.node_ref(nav_ref).style(move || {
                if mode == ShellNavMode::Desktop {
                    style()
                        .position(Position::Relative)
                        .width(
                            animated_width
                                .as_ref()
                                .map_or_else(&target, |width| width.get()),
                        )
                        .height(pct(100))
                        .clip(true)
                } else {
                    style().width(pct(100)).grow(1)
                }
            });
            let grip = resize.map(|resize| {
                let size = binding(
                    move || {
                        if collapsed.is_some_and(|collapsed| collapsed.get()) {
                            0.0
                        } else {
                            expanded_width.get()
                        }
                    },
                    move |size| {
                        if let Some(collapsed) = collapsed {
                            collapsed.set(size < 160.0);
                        }
                        if size >= 160.0 {
                            expanded_width.set(size.clamp(resize.min, resize.max));
                        }
                    },
                );
                let behavior = use_splitter(
                    size,
                    ScalarRange::new(0.0, resize.max)
                        .expect("invariant: sidebar maximum is positive"),
                    Orientation::Horizontal,
                )
                .collapsible(160.0 / resize.max)
                .key_step(10.0)
                .expect("invariant: sidebar keyboard step is positive");
                let part = behavior.handle();
                let visual = part.state();
                node()
                    .part(part)
                    .name(message("soft.side_nav.resize", "Resize sidebar"))
                    .controls(nav_ref)
                    .style(move || {
                        let base = style()
                            .position(Position::Absolute)
                            .top(0)
                            .bottom(0)
                            .width(16)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .cursor(kivact::style::Cursor::EwResize);
                        if rtl() { base.left(0) } else { base.right(0) }
                    })
                    .child(node().style(move || {
                        let t = kivact::style::use_tokens::<crate::Tokens>();
                        let state = visual.get();
                        style()
                            .size(3, t.spacing.s8)
                            .radius(t.radius.full)
                            .translate_x(if rtl() { -7.0 } else { 7.0 })
                            .opacity(
                                if state.dragging || state.input.hovered || state.input.focused {
                                    1.0
                                } else {
                                    0.0
                                },
                            )
                            .background(if state.dragging {
                                t.colors.border_emphasized
                            } else {
                                t.colors.border
                            })
                    }))
            });
            root.child((controls, grip))
        })
    }
}

/// A control that toggles the sidebar's collapsed rail.
pub struct SideNavCollapseButton {
    root: Node,
    binding: Option<Binding<bool>>,
    size: Value<ControlSize>,
    label: Option<Value<String>>,
}

/// Create a collapse toggle using the surrounding sidebar context.
pub fn side_nav_collapse_button() -> SideNavCollapseButton {
    SideNavCollapseButton {
        root: node(),
        binding: None,
        size: (ControlSize::Medium).into_value(),
        label: None,
    }
}

impl SideNavCollapseButton {
    /// Override the surrounding sidebar's collapse binding.
    pub fn binding(mut self, collapsed: impl IntoBinding<bool>) -> Self {
        self.binding = Some(collapsed.into_binding());
        self
    }
    /// Set the collapse toggle button size.
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
    /// Override the toggle's accessible label.
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }
}

impl NodeBuilder for SideNavCollapseButton {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SideNavCollapseButton {
    fn into_view(self) -> View {
        let Self {
            root,
            binding,
            size,
            label,
        } = self;
        kivact::view::component(move || {
            let Some(collapsed) = binding.or(nav_cx().collapsed) else {
                return View::empty();
            };
            if use_app_shell_mobile().is_mobile() {
                return View::empty();
            }
            let expand = message("soft.side_nav.expand", "Expand sidebar");
            let collapse = message("soft.side_nav.collapse", "Collapse sidebar");
            let title = move || {
                if collapsed.get() {
                    expand.get()
                } else {
                    collapse.get()
                }
            };
            let glyph = icon(if rtl() {
                "chevron-right"
            } else {
                "chevron-left"
            })
            .style(move || style().rotate(if collapsed.get() { 180 } else { 0 }));
            let trigger = button(move || label.map_or_else(title, |label| label.get()))
                .variant(ButtonVariant::Ghost)
                .size(size.get())
                .icon(glyph)
                .hide_label(label.is_none())
                .with_node(move |node| node.expanded(move || !collapsed.get()))
                .on_press(move || collapsed.set(!collapsed.get()));
            root.child(tooltip(title).placement(Placement::End).attach(trigger))
                .into_view()
        })
    }
}
