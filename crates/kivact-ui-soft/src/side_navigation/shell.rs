//! Role: Render the sidebar shell and collapse control.

use super::context::{NavSurface, SideNavCx, nav_cx};
use super::items::logical_row;

use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::color as c;
use crate::mobile_nav::use_app_shell_mobile;
use crate::navigation_session::with_navigation_group;
use crate::radius::RADIUS_FULL;
use crate::space::{self as s, SPACE_S12_VAR, Space};
use crate::stack::rtl;

use kivact::component;
use kivact::i18n::message;
use kivact::interaction::{ScalarRange, use_splitter};
use kivact::motion::Spring;
use kivact::motion::Transition;
use kivact::motion::animated;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::provide;
use kivact::state::signal;
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::Justify;
use kivact::style::Position;
use kivact::style::auto;
use kivact::style::pct;
use kivact::style::style;
use kivact::style::use_theme;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::ui::tooltip;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::Node;
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
    Controlled(Signal<bool>),
}

/// Initial width, width bounds and collapse ownership for a resizable sidebar.
/// `Default` starts expanded at 260 px, bounded to 180–480 px.
pub struct SideNavResize {
    default_width: f64,
    min: f64,
    max: f64,
    collapse: SideNavCollapse,
}

impl Default for SideNavResize {
    fn default() -> Self {
        Self {
            default_width: 260.0,
            min: 180.0,
            max: 480.0,
            collapse: SideNavCollapse::Uncontrolled { default: false },
        }
    }
}

impl SideNavResize {
    /// Set the initial expanded width in pixels.
    pub fn width(mut self, width: impl Into<f64>) -> Self {
        self.default_width = width.into();
        self
    }
    /// Set the allowed expanded width range in pixels. Bounds that are not
    /// finite, a negative minimum, a zero maximum or a maximum below the
    /// minimum keep the default 180–480 px.
    pub fn range(mut self, min: impl Into<f64>, max: impl Into<f64>) -> Self {
        let (min, max) = (min.into(), max.into());
        if min.is_finite() && max.is_finite() && min >= 0.0 && max > 0.0 && max >= min {
            self.min = min;
            self.max = max;
        } else {
            let fallback = Self::default();
            self.min = fallback.min;
            self.max = fallback.max;
        }
        self
    }
    /// Start an uncontrolled sidebar collapsed.
    pub fn initially_collapsed(mut self) -> Self {
        if let SideNavCollapse::Uncontrolled { default } = &mut self.collapse {
            *default = true;
        }
        self
    }
    /// Use the caller's single collapse binding.
    pub fn collapsed(mut self, collapsed: Signal<bool>) -> Self {
        self.collapse = SideNavCollapse::Controlled(collapsed);
        self
    }
}

/// A sidebar with a scrollable entry region and retained header and footer slots.
///
/// `collapsible` and `resizable` apply on the desktop shell only; `collapse_button` shows the
/// built-in toggle when the sidebar can collapse.
#[component]
pub fn side_nav(
    children: View,
    header: Option<View>,
    top_content: Option<View>,
    footer: Option<View>,
    footer_icons: Option<View>,
    collapsible: Option<SideNavCollapse>,
    #[prop(default = true)] collapse_button: Value<bool>,
    resizable: Option<SideNavResize>,
) -> impl Element {
    let root = node()
        .with_node(move |node| node.role(Role::Navigation))
        .name(message("soft.side_nav.label", "Edge navigation"));
    let mode = shell_nav_mode();
    let root = root.with_node(move |node| {
        node.base_style(move || {
            style()
                .flex_direction(if mode == ShellNavMode::MobileBar {
                    logical_row()
                } else {
                    FlexDirection::Column
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
            .child((header, footer_icons));
    }
    let nav_ref = node_ref();
    let collapse = if mode == ShellNavMode::Desktop {
        collapsible
    } else {
        None
    };
    let resize = if mode == ShellNavMode::Desktop {
        resizable
    } else {
        None
    };
    let collapsed = sidebar_collapse(collapse, resize.as_ref());
    let expanded_width = signal(resize.as_ref().map_or(260.0, |resize| resize.default_width));
    let cx = SideNavCx {
        surface: collapsed.map_or(NavSurface::Fixed, NavSurface::Collapsible),
        ..SideNavCx::default()
    };
    let width = sidebar_width(cx, expanded_width, resize.is_none() && collapsed.is_some());
    let slots = SideNavSlots {
        children,
        header,
        top: top_content,
        footer,
        icons: footer_icons,
    };
    let root = root.node_ref(nav_ref).style(move || {
        if mode == ShellNavMode::Desktop {
            style()
                .position(Position::Relative)
                .width(width())
                .height(pct(100))
                .clip(true)
        } else {
            style().width(pct(100)).flex_grow(1)
        }
    });
    root.child(with_navigation_group(move || {
        let controls = slots.controls(cx, mode, collapse_button, collapsed);
        let grip = resize.map(|resize| resize_grip(resize, collapsed, expanded_width, nav_ref));
        (controls, grip)
    }))
}

/// A control that toggles the sidebar's collapsed rail. It is hidden when there is no collapse
/// binding, and on mobile. `binding` overrides the surrounding sidebar's binding, and `label`
/// is the visible text of the toggle (the icon alone when it is not given).
#[component]
pub fn side_nav_collapse_button(
    binding: Option<Signal<bool>>,
    #[prop(default = ControlSize::Medium)] size: Value<ControlSize>,
    label: Option<Value<String>>,
) -> impl Element {
    let root = node();
    let Some(collapsed) = binding.or(nav_cx().collapsed()) else {
        return root.hidden(true);
    };
    if use_app_shell_mobile().is_mobile() {
        return root.hidden(true);
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
    root.child(tooltip(title, trigger).placement(Placement::End))
}

fn sidebar_collapse(
    collapse: Option<SideNavCollapse>,
    resize: Option<&SideNavResize>,
) -> Option<Signal<bool>> {
    resize
        .map(|resize| match &resize.collapse {
            SideNavCollapse::Uncontrolled { default } => signal(*default),
            SideNavCollapse::Controlled(collapsed) => *collapsed,
        })
        .or_else(|| {
            collapse.map(|collapse| match collapse {
                SideNavCollapse::Uncontrolled { default } => signal(default),
                SideNavCollapse::Controlled(collapsed) => collapsed,
            })
        })
}

struct SideNavSlots {
    children: View,
    header: Option<View>,
    top: Option<View>,
    footer: Option<View>,
    icons: Option<View>,
}
impl SideNavSlots {
    fn controls(
        self,
        cx: SideNavCx,
        mode: ShellNavMode,
        collapse_button: Value<bool>,
        collapsed: Option<Signal<bool>>,
    ) -> View {
        let Self {
            children,
            header,
            top,
            footer,
            icons,
        } = self;
        let has_top = header.is_some() || top.is_some();
        let has_bottom =
            footer.is_some() || icons.is_some() || collapse_button.get() && collapsed.is_some();
        provide(cx, move || {
            let top = column((header, top)).style(move || {
                let mut base = style().gap(s::SPACE_S2).flex_shrink(0);
                if has_top && mode == ShellNavMode::Desktop {
                    base = base.padding(s::SPACE_S2);
                }
                if cx.rail() {
                    base = base.align_items(Align::Center);
                }
                base
            });
            let items =
                column(children).style(move || sidebar_items_style(cx, mode, has_top, has_bottom));
            let items = if mode == ShellNavMode::Desktop {
                scroll(items)
                    .style(style().flex_grow(1).min_height(0))
                    .into_view()
            } else {
                items.into_view()
            };
            let bottom = has_bottom.then(|| {
                let controls = row((
                    (collapse_button.get() && collapsed.is_some())
                        .then(|| side_nav_collapse_button().size(ControlSize::Small)),
                    icons,
                ))
                .style(move || {
                    style()
                        .flex_direction(if cx.rail() {
                            FlexDirection::ColumnReverse
                        } else {
                            logical_row()
                        })
                        .align_items(Align::Center)
                        .gap(s::SPACE_S1)
                });
                column((
                    footer.map(|slot| node().hidden(move || cx.rail()).child(slot)),
                    controls,
                ))
                .style(move || sidebar_bottom_style(cx, mode))
            });
            (top, items, bottom)
        })
    }
}
fn sidebar_items_style(
    cx: SideNavCx,
    mode: ShellNavMode,
    has_top: bool,
    has_bottom: bool,
) -> kivact::style::Style {
    let mut base = style().gap(s::SPACE_S0_5);
    if mode == ShellNavMode::Desktop {
        base = base
            .padding_x(s::SPACE_S2)
            .padding_top(if has_top { s::SPACE_S1 } else { s::SPACE_S2 })
            .padding_bottom(if has_bottom { s::SPACE_S1 } else { s::SPACE_S2 });
    }
    if cx.rail() {
        base = base.align_items(Align::Center);
    }
    base
}
fn sidebar_bottom_style(cx: SideNavCx, mode: ShellNavMode) -> kivact::style::Style {
    let mut base = style().margin_top(auto()).gap(s::SPACE_S2).flex_shrink(0);
    if mode == ShellNavMode::Desktop {
        let top = if cx.rail() { 0.0 } else { Space::S1.px() };
        base = base
            .padding_x(s::SPACE_S2)
            .padding_bottom(s::SPACE_S2)
            .padding_top(top);
    }
    if cx.rail() {
        base = base.align_items(Align::Center);
    }
    base
}

fn resize_grip(
    resize: SideNavResize,
    collapsed: Option<Signal<bool>>,
    expanded_width: kivact::state::Signal<f64>,
    nav_ref: kivact::view::NodeRef,
) -> Node {
    let size = binding(
        move || {
            if collapsed.is_some_and(|collapsed| collapsed.get()) {
                0.0
            } else {
                expanded_width.get()
            }
        },
        move |size| {
            if size < 160.0 {
                if let Some(collapsed) = collapsed {
                    collapsed.set(true);
                }
            } else {
                if let Some(collapsed) = collapsed {
                    collapsed.set(false);
                }
                expanded_width.set(size.clamp(resize.min, resize.max));
            }
        },
    );
    let behavior = use_splitter(
        size,
        ScalarRange::new(0.0, resize.max).expect("invariant: sidebar maximum is positive"),
        Orientation::Horizontal,
    )
    .collapsible((160.0 / resize.max).min(1.0))
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
            let state = visual.get();
            style()
                .size(3, s::SPACE_S8)
                .border_radius(RADIUS_FULL)
                .translate_x(if rtl() { -7.0 } else { 7.0 })
                .opacity(
                    if state.dragging || state.input.is_hovered() || state.input.is_focused() {
                        1.0
                    } else {
                        0.0
                    },
                )
                .background(if state.dragging {
                    c::BORDER_EMPHASIZED
                } else {
                    c::BORDER
                })
        }))
}

fn sidebar_width(
    cx: SideNavCx,
    expanded_width: kivact::state::Signal<f64>,
    animate: bool,
) -> impl Fn() -> f64 {
    let target = move || {
        if cx.rail() {
            use_theme().get().get(&SPACE_S12_VAR)
        } else {
            expanded_width.get()
        }
    };
    let animated_width = if animate {
        Some(animated(
            target,
            Transition::spring(Spring {
                stiffness: 157.91,
                damping: 25.13,
                mass: 1.0,
                rest: 1e-3,
            }),
        ))
    } else {
        None
    };
    move || {
        animated_width
            .as_ref()
            .map_or_else(&target, |width| width.get())
    }
}
