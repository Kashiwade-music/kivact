//! Role: Render the sidebar shell and collapse control.

use super::context::{SideNavCx, nav_cx};
use super::items::logical_row;

use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::mobile_nav::use_app_shell_mobile;
use crate::resizable::{
    PillPlacement, ResizableConfig, ResizableState, ResizeSize, resize_handle, use_resizable,
};
use crate::stack::rtl;
use crate::tokens;
use kivact::i18n::message;
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
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::row;
use kivact::view::scroll;

pub enum SideNavCollapse {
    Uncontrolled { default: bool },
    Controlled(Binding<bool>),
}

pub struct SideNavResize {
    pub default_width: f64,
    pub min: f64,
    pub max: f64,
    pub default_collapsed: bool,
    pub collapsed: Option<Binding<bool>>,
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

pub struct SideNav {
    root: Node,
    children: View,
    header: Option<ViewFn>,
    top: Option<ViewFn>,
    footer: Option<ViewFn>,
    icons: Option<ViewFn>,
    collapse: Option<SideNavCollapse>,
    resize: Option<SideNavResize>,
    collapse_button: Value<bool>,
}

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
    pub fn header<V: IntoView>(mut self, header: impl IntoViewFn<V>) -> Self {
        self.header = Some(header.into_view_fn());
        self
    }
    pub fn top_content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.top = Some(content.into_view_fn());
        self
    }
    pub fn footer<V: IntoView>(mut self, footer: impl IntoViewFn<V>) -> Self {
        self.footer = Some(footer.into_view_fn());
        self
    }
    pub fn footer_icons<V: IntoView>(mut self, icons: impl IntoViewFn<V>) -> Self {
        self.icons = Some(icons.into_view_fn());
        self
    }
    pub fn collapsible(mut self, collapse: SideNavCollapse) -> Self {
        self.collapse = Some(collapse);
        self
    }
    pub fn collapse_button(mut self, show: impl IntoValue<bool>) -> Self {
        self.collapse_button = show.into_value();
        self
    }
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
        provide((), move || {
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
                    .child((
                        header.map(|slot| slot.call(())),
                        icons.map(|slot| slot.call(())),
                    ));
            }
            let nav_ref = node_ref();
            let collapse = if mode == ShellNavMode::Desktop {
                collapse
            } else {
                None
            };
            let resize = if mode == ShellNavMode::Desktop {
                resize.map(|resize| {
                    use_resizable(ResizableConfig {
                        default_size: ResizeSize::Px(resize.default_width),
                        min_size: ResizeSize::Px(resize.min),
                        max_size: ResizeSize::Px(resize.max),
                        collapsible: true,
                        collapse_threshold: 160.0,
                        default_collapsed: resize.default_collapsed,
                        collapsed: resize.collapsed,
                        ..ResizableConfig::default()
                    })
                })
            } else {
                None
            };
            let collapsed = if let Some(state) = resize.clone() {
                let writer = state.clone();
                Some(binding(
                    move || state.is_collapsed(),
                    move |collapsed| {
                        if collapsed {
                            writer.collapse();
                        } else {
                            writer.expand();
                        }
                    },
                ))
            } else {
                collapse.map(|collapse| match collapse {
                    SideNavCollapse::Uncontrolled { default } => signal(default).into_binding(),
                    SideNavCollapse::Controlled(collapsed) => collapsed,
                })
            };
            let cx = SideNavCx {
                collapsed,
                ..SideNavCx::default()
            };
            let resize_style = resize.clone();
            let target = move || {
                if cx.rail() {
                    tokens().spacing.s12
                } else {
                    resize_style.as_ref().map_or(260.0, ResizableState::size)
                }
            };
            let animated_width = if resize.is_none() && collapsed.is_some() {
                Some(animated(
                    target.clone(),
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
                let top = column((
                    header.map(|slot| slot.call(())),
                    top.map(|slot| slot.call(())),
                ))
                .style(move || {
                    let t = tokens();
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
                    let t = tokens();
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
                        icons.map(|slot| slot.call(())),
                    ))
                    .style(move || {
                        style()
                            .direction(if cx.rail() {
                                Direction::ColumnReverse
                            } else {
                                logical_row()
                            })
                            .align_items(Align::Center)
                            .gap(tokens().spacing.s1)
                    });
                    column((
                        footer.map(|slot| node().hidden(move || cx.rail()).child(slot.call(()))),
                        controls,
                    ))
                    .style(move || {
                        let t = tokens();
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
            let grip = resize.map(|state| {
                // The sidebar's rail remains 48 px while its content is collapsed.
                resize_handle(state)
                    .overlay(true)
                    .pill_placement(PillPlacement::End)
                    .always_visible(false)
                    .name(message("soft.side_nav.resize", "Resize sidebar"))
                    .with_node(move |node| node.controls(nav_ref))
            });
            root.child((controls, grip))
        })
    }
}

pub struct SideNavCollapseButton {
    root: Node,
    binding: Option<Binding<bool>>,
    size: Value<ControlSize>,
    label: Option<Value<String>>,
}

pub fn side_nav_collapse_button() -> SideNavCollapseButton {
    SideNavCollapseButton {
        root: node(),
        binding: None,
        size: (ControlSize::Medium).into_value(),
        label: None,
    }
}

impl SideNavCollapseButton {
    pub fn binding(mut self, collapsed: impl IntoBinding<bool>) -> Self {
        self.binding = Some(collapsed.into_binding());
        self
    }
    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }
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
        provide((), move || {
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
