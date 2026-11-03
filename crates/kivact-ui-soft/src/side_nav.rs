//! Role: Render Neutral sidebar navigation, nested items and its collapsed rail.
use crate::app_shell::{ShellNavMode, shell_nav_mode};
use crate::mobile_nav::use_app_shell_mobile;
use crate::resizable::{
    PillPlacement, ResizableConfig, ResizableState, ResizeSize, resize_handle, use_resizable,
};
use crate::stack::rtl;
use crate::tokens;
use kivact::control::{Node, NodeBuilder, column, icon, node, row, scroll, text};
use kivact::i18n::{message, message_with};
use kivact::interaction::{
    Delays, use_content_reveal, use_dialog, use_disclosure, use_hover_surface,
};
use kivact::motion::{Spring, Transition, animated};
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, Justify, Position, TransitionProp, auto, pct, style};
use kivact::ui::{ButtonVariant, ControlSize, DialogKind, button, tooltip};
use kivact::view::{
    HandlerResult, IntoText, IntoValue, IntoView, Placement, Role, View, callback, node_ref,
    overlay, provide, show, try_context,
};
use kivact::{Binding, Callback, Value, binding, signal};
use std::rc::Rc;
use std::time::Duration;

type Slot = Rc<dyn Fn() -> View>;

#[derive(Clone, Copy, Default)]
struct SideNavCx {
    collapsed: Option<Binding<bool>>,
    depth: usize,
    popup: Option<Binding<bool>>,
}

fn nav_cx() -> SideNavCx {
    try_context::<SideNavCx>().unwrap_or_default()
}

impl SideNavCx {
    fn rail(self) -> bool {
        self.collapsed.is_some_and(|collapsed| collapsed.get())
    }
}

pub enum SideNavCollapse {
    Uncontrolled { default: bool },
    Controlled(Binding<bool>),
}

pub struct SideNavResize {
    pub default_width: f32,
    pub min: f32,
    pub max: f32,
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
    header: Option<Slot>,
    top: Option<Slot>,
    footer: Option<Slot>,
    icons: Option<Slot>,
    collapse: Option<SideNavCollapse>,
    resize: Option<SideNavResize>,
    collapse_button: bool,
}

pub fn side_nav(children: impl IntoView) -> SideNav {
    SideNav {
        root: node()
            .role(Role::Navigation)
            .name(message("soft.side_nav.label", "Side navigation")),
        children: children.into_view(),
        header: None,
        top: None,
        footer: None,
        icons: None,
        collapse: None,
        resize: None,
        collapse_button: true,
    }
}

impl SideNav {
    pub fn header<V: IntoView>(mut self, header: impl Fn() -> V + 'static) -> Self {
        self.header = Some(Rc::new(move || header().into_view()));
        self
    }
    pub fn top_content<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.top = Some(Rc::new(move || content().into_view()));
        self
    }
    pub fn footer<V: IntoView>(mut self, footer: impl Fn() -> V + 'static) -> Self {
        self.footer = Some(Rc::new(move || footer().into_view()));
        self
    }
    pub fn footer_icons<V: IntoView>(mut self, icons: impl Fn() -> V + 'static) -> Self {
        self.icons = Some(Rc::new(move || icons().into_view()));
        self
    }
    pub fn collapsible(mut self, collapse: SideNavCollapse) -> Self {
        self.collapse = Some(collapse);
        self
    }
    pub fn collapse_button(mut self, show: bool) -> Self {
        self.collapse_button = show;
        self
    }
    pub fn resizable(mut self, resize: SideNavResize) -> Self {
        self.resize = Some(resize);
        self
    }
}

impl NodeBuilder for SideNav {
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
            let root = root.base_style(move || {
                style()
                    .direction(if mode == ShellNavMode::MobileBar {
                        logical_row()
                    } else {
                        Direction::Column
                    })
                    .min_width(0)
                    .min_height(0)
            });
            if mode == ShellNavMode::MobileBar {
                return root
                    .style(|| {
                        style()
                            .width(pct(100))
                            .align_items(Align::Center)
                            .justify_content(Justify::SpaceBetween)
                    })
                    .child((header.map(|slot| slot()), icons.map(|slot| slot())));
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
                    SideNavCollapse::Uncontrolled { default } => signal(default).into(),
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
                footer.is_some() || icons.is_some() || collapse_button && collapsed.is_some();
            let controls = provide(cx, move || {
                let top =
                    column((header.map(|slot| slot()), top.map(|slot| slot()))).style(move || {
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
                        (collapse_button && collapsed.is_some())
                            .then(|| side_nav_collapse_button().size(ControlSize::Small)),
                        icons.map(|slot| slot()),
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
                        footer.map(|slot| node().hidden(move || cx.rail()).child(slot())),
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
                    .controls(nav_ref)
            });
            root.child((controls, grip))
        })
    }
}

pub struct SideNavCollapseButton {
    root: Node,
    binding: Option<Binding<bool>>,
    size: ControlSize,
    label: Option<Value<String>>,
}

pub fn side_nav_collapse_button() -> SideNavCollapseButton {
    SideNavCollapseButton {
        root: node(),
        binding: None,
        size: ControlSize::Medium,
        label: None,
    }
}

impl SideNavCollapseButton {
    pub fn binding(mut self, collapsed: impl Into<Binding<bool>>) -> Self {
        self.binding = Some(collapsed.into());
        self
    }
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    pub fn label(mut self, label: impl IntoText) -> Self {
        self.label = Some(label.into_text_value());
        self
    }
}

impl NodeBuilder for SideNavCollapseButton {
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
                .size(size)
                .icon(glyph)
                .hide_label(label.is_none())
                .expanded(move || !collapsed.get())
                .on_press(move || collapsed.set(!collapsed.get()));
            root.child(tooltip(title).placement(Placement::End).attach(trigger))
                .into_view()
        })
    }
}

pub enum ItemCollapse {
    AlwaysOpen,
    Uncontrolled { default_collapsed: bool },
    Controlled(Binding<bool>),
}

pub struct SideNavItem {
    root: Node,
    label: Value<String>,
    icon: Option<View>,
    selected_icon: Option<View>,
    selected: Value<bool>,
    disabled: Value<bool>,
    end: Option<View>,
    actions: Option<View>,
    children: Option<Slot>,
    collapse: ItemCollapse,
    size: ControlSize,
    action: Option<Callback<()>>,
}

pub fn side_nav_item(label: impl IntoText) -> SideNavItem {
    let label = label.into_text_value();
    SideNavItem {
        root: node(),
        label,
        icon: None,
        selected_icon: None,
        selected: false.into_value(),
        disabled: false.into_value(),
        end: None,
        actions: None,
        children: None,
        collapse: ItemCollapse::Uncontrolled {
            default_collapsed: false,
        },
        size: ControlSize::Medium,
        action: None,
    }
}

impl SideNavItem {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn selected_icon(mut self, icon: impl IntoView) -> Self {
        self.selected_icon = Some(icon.into_view());
        self
    }
    pub fn selected(mut self, selected: impl IntoValue<bool>) -> Self {
        self.selected = selected.into_value();
        self
    }
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    pub fn end_content<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.end = Some(provide((), content));
        self
    }
    pub fn actions<V: IntoView>(mut self, actions: impl FnOnce() -> V + 'static) -> Self {
        self.actions = Some(provide((), actions));
        self
    }
    pub fn children<V: IntoView>(mut self, children: impl Fn() -> V + 'static) -> Self {
        self.children = Some(Rc::new(move || children().into_view()));
        self
    }
    pub fn collapsible(mut self, collapse: ItemCollapse) -> Self {
        self.collapse = collapse;
        self
    }
    pub fn on_press<R: HandlerResult>(mut self, action: impl Fn() -> R + 'static) -> Self {
        self.action = Some(callback(move |_| action()));
        self
    }
}

impl NodeBuilder for SideNavItem {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SideNavItem {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            icon: leading,
            selected_icon,
            selected,
            disabled,
            end,
            actions,
            children,
            collapse,
            size,
            action,
        } = self;
        provide((), move || {
            let cx = nav_cx();
            let mode = shell_nav_mode();
            let has_icon = leading.is_some() || selected_icon.is_some();
            let has_selected_icon = selected_icon.is_some();
            let child_slot = children.clone();
            let has_children = children.is_some();
            let mobile = use_app_shell_mobile();
            let collapse = match collapse {
                ItemCollapse::AlwaysOpen => None,
                ItemCollapse::Uncontrolled { default_collapsed } => {
                    Some(signal(default_collapsed).into())
                }
                ItemCollapse::Controlled(binding) => Some(binding),
            };
            let open = collapse.map(|collapse: Binding<bool>| {
                binding(move || !collapse.get(), move |open| collapse.set(!open))
            });
            let (disclosure, reveal) = if let Some(open) = open.filter(|_| has_children) {
                let disclosure = use_disclosure(open).disabled(disabled);
                let motion = tokens().motion;
                let reveal = use_content_reveal(
                    disclosure.shown(),
                    Transition::tween(motion.medium as f64, motion.ease_standard),
                );
                (
                    Some(disclosure.leaving(move || reveal.leaving())),
                    Some(reveal),
                )
            } else {
                (None, None)
            };
            let independent = disclosure.is_some() && action.is_some();
            let anchor = node_ref();
            let popup_ref = node_ref();
            let popup = if children.is_some() && cx.collapsed.is_some() {
                let delayed = use_hover_surface(
                    anchor,
                    popup_ref,
                    move || cx.rail() && !disabled.get(),
                    Delays {
                        open: Duration::from_millis(150),
                        close: Duration::from_millis(200),
                    },
                );
                let pinned = signal(false);
                Some(binding(
                    move || cx.rail() && (pinned.get() || delayed.shown().get()),
                    move |open| {
                        pinned.set(open);
                        if !open {
                            delayed.dismiss();
                        }
                    },
                ))
            } else {
                None
            };
            let leading = leading.map(|icon| {
                icon_box(
                    icon,
                    move || has_selected_icon && selected.get(),
                    selected,
                    disabled,
                )
            });
            let selected_glyph = selected_icon
                .map(|icon| icon_box(icon, move || !selected.get(), selected, disabled));
            let label_view = text(label)
                .hidden(move || cx.rail())
                .style(style().grow(1).min_width(0).single_line());
            let chevron = disclosure.map(|disclosure| {
                let glyph = icon("chevron-down").style(move || {
                    let t = tokens();
                    style()
                        .size(t.spacing.s6, t.spacing.s6)
                        .rotate(if disclosure.shown().get() { 180 } else { 0 })
                        .transition(
                            TransitionProp::Paint,
                            Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
                        )
                });
                if independent {
                    let collapse = message_with(
                        "soft.side_nav.item_collapse",
                        "Collapse {label}",
                        move || [("label", label.get())],
                    );
                    let expand =
                        message_with("soft.side_nav.item_expand", "Expand {label}", move || {
                            [("label", label.get())]
                        });
                    button(move || {
                        if disclosure.shown().get() {
                            collapse.get()
                        } else {
                            expand.get()
                        }
                    })
                    .variant(ButtonVariant::Ghost)
                    .size(ControlSize::Small)
                    .hide_label(true)
                    .icon(glyph)
                    .part(disclosure.trigger())
                    .hidden(move || cx.rail())
                    .into_view()
                } else {
                    glyph.hidden(move || cx.rail()).into_view()
                }
            });
            let (inline_chevron, separate_chevron) = if independent {
                (None, chevron)
            } else {
                (chevron, None)
            };
            let end = end.map(|end| node().hidden(move || cx.rail()).child(end));
            let primary = node()
                .node_ref(anchor)
                .name(label)
                .role(if action.is_some() {
                    Role::Link
                } else {
                    Role::Button
                })
                .focusable(move || {
                    !disabled.get() && (action.is_some() || disclosure.is_some() || popup.is_some())
                })
                .disabled(disabled)
                .selected(selected)
                .current_item(move || selected.get().then_some(kivact::view::CurrentItem::Page))
                .style(move || {
                    style()
                        .direction(logical_row())
                        .grow(1)
                        .min_width(0)
                        .align_self(Align::Stretch)
                        .align_items(Align::Center)
                        .gap(tokens().spacing.s2)
                        .focus_visible(crate::recipe_style::ring(&tokens(), tokens().colors.accent))
                        .justify_content(if cx.rail() {
                            Justify::Center
                        } else {
                            Justify::Start
                        })
                })
                .child((leading, selected_glyph, label_view, end, inline_chevron));
            let primary = if !independent {
                if let Some(disclosure) = disclosure.filter(|_| popup.is_none()) {
                    primary.part(disclosure.trigger())
                } else {
                    primary
                }
            } else {
                primary
            };
            let primary = primary.on_press(move || {
                if disabled.get() {
                    return;
                }
                if let Some(popup) = popup.filter(|_| cx.rail()) {
                    popup.set(!popup.get());
                } else if let Some(action) = action {
                    action.call(());
                    if let Some(popup) = cx.popup {
                        popup.set(false);
                    }
                    if mode == ShellNavMode::Drawer {
                        mobile.close();
                    }
                } else if popup.is_some()
                    && let Some(open) = open
                {
                    open.set(!open.get());
                }
            });
            let primary = if children.is_none() && cx.collapsed.is_some() {
                tooltip(label).placement(Placement::End).attach(primary)
            } else {
                primary.into_view()
            };
            let actions = actions.map(|actions| {
                row(actions)
                    .hidden(move || cx.rail())
                    .style(|| style().gap(tokens().spacing.s1))
            });
            let bar = row((primary, separate_chevron, actions)).style(move || {
                side_row_style(size, cx.depth, cx.rail(), selected.get(), disabled.get())
            });
            let nested = show(
                move || !cx.rail() && has_children && reveal.is_none_or(|reveal| reveal.present()),
                move || {
                    let child = child_slot.clone();
                    let cx = SideNavCx {
                        depth: cx.depth + 1,
                        ..cx
                    };
                    provide(cx, move || {
                        let group = column(child.map(|slot| slot()))
                            .style(|| style().gap(tokens().spacing.s0_5));
                        if let Some((disclosure, reveal)) = disclosure.zip(reveal) {
                            let measured = node_ref();
                            let height = reveal.container_style(measured);
                            column(group.node_ref(measured).style(style().shrink(0)))
                                .role(Role::Group)
                                .part(disclosure.content())
                                .style(move || height.get())
                        } else {
                            group.role(Role::Group)
                        }
                    })
                },
            );
            let flyout = popup.zip(children).map(|(open, children)| {
                let behavior = use_dialog(open, DialogKind::Popover).anchor(anchor);
                let title =
                    message_with("soft.side_nav.item_submenu", "{label} submenu", move || {
                        [("label", label.get())]
                    });
                show(behavior.shown(), move || {
                    let children = children.clone();
                    let cx = SideNavCx {
                        popup: Some(open),
                        ..SideNavCx::default()
                    };
                    let surface = column((
                        text(label).style(|| {
                            let t = tokens();
                            style()
                                .padding_y(t.spacing.s1)
                                .padding_x(t.spacing.s2)
                                .font_size(t.typography.supporting.size_px)
                                .weight(t.typography.weight_semibold)
                                .color(t.colors.text_secondary)
                        }),
                        provide(cx, move || children()),
                    ))
                    .node_ref(popup_ref)
                    .style(|| {
                        let t = tokens();
                        let mut base = style()
                            .background(t.colors.bg_popover)
                            .min_width(180)
                            .padding(t.spacing.s1)
                            .radius(t.radius.container)
                            .gap(t.spacing.s0_5);
                        for shadow in &t.shadows.low {
                            base = base.shadow(*shadow);
                        }
                        base
                    });
                    overlay(anchor, Placement::End, surface)
                        .offset(4)
                        .part(behavior.content())
                        .name(title)
                })
            });
            root.hidden(move || cx.rail() && !has_icon)
                .base_style(style().direction(Direction::Column))
                .child((bar, nested, flyout))
        })
    }
}

fn icon_box(
    glyph: View,
    hidden: impl Fn() -> bool + 'static,
    selected: Value<bool>,
    disabled: Value<bool>,
) -> Node {
    node()
        .hidden(hidden)
        .style(move || {
            let t = tokens();
            style()
                .size(t.sizes.icon_sm, t.sizes.icon_sm)
                .shrink(0)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .color(if disabled.get() {
                    t.colors.icon_disabled
                } else if selected.get() {
                    t.colors.text_primary
                } else {
                    t.colors.icon_secondary
                })
        })
        .child(glyph)
}

pub(crate) fn logical_row() -> Direction {
    if rtl() {
        Direction::RowReverse
    } else {
        Direction::Row
    }
}

pub(crate) fn side_row_style(
    size: ControlSize,
    depth: usize,
    rail: bool,
    selected: bool,
    disabled: bool,
) -> kivact::style::Style {
    let t = tokens();
    let height = match size {
        ControlSize::Small => t.sizes.control_sm,
        ControlSize::Large => t.sizes.control_lg,
        _ => t.sizes.control_md,
    };
    let pad = if size == ControlSize::Small {
        t.spacing.s1
    } else {
        t.spacing.s2
    };
    let base = style()
        .direction(logical_row())
        .align_items(Align::Center)
        .gap(t.spacing.s2)
        .height(height)
        .radius(t.radius.element)
        .font_size(t.typography.label.size_px)
        .line_height(t.typography.label.line_height_px / t.typography.label.size_px)
        .weight(if selected {
            t.typography.weight_medium
        } else {
            t.typography.weight_normal
        })
        .color(if disabled {
            t.colors.text_disabled
        } else {
            t.colors.text_primary
        })
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent));
    let base = if rail {
        base.width(height).justify_content(Justify::Center)
    } else {
        let start = pad + depth as f32 * t.spacing.s6;
        let (left, right) = if rtl() { (pad, start) } else { (start, pad) };
        base.width(pct(100)).padding_left(left).padding_right(right)
    };
    if selected {
        base.background(t.colors.neutral)
    } else if disabled {
        base
    } else {
        base.hover(style().background(t.colors.overlay_hover))
            .pressed(style().background(t.colors.overlay_pressed))
    }
}

pub struct SideNavSection {
    root: Node,
    title: Value<String>,
    subtitle: Option<Value<String>>,
    end: Option<View>,
    children: View,
    header_hidden: bool,
}

pub fn side_nav_section(title: impl IntoText, children: impl IntoView) -> SideNavSection {
    let title = title.into_text_value();
    SideNavSection {
        root: node().role(Role::Group).name(title),
        title,
        subtitle: None,
        end: None,
        children: children.into_view(),
        header_hidden: false,
    }
}

impl SideNavSection {
    pub fn subtitle(mut self, subtitle: impl IntoText) -> Self {
        self.subtitle = Some(subtitle.into_text_value());
        self
    }
    pub fn end_content<V: IntoView>(mut self, content: impl FnOnce() -> V + 'static) -> Self {
        self.end = Some(provide((), content));
        self
    }
    pub fn header_hidden(mut self, hidden: bool) -> Self {
        self.header_hidden = hidden;
        self
    }
}

impl NodeBuilder for SideNavSection {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for SideNavSection {
    fn into_view(self) -> View {
        let Self {
            root,
            title,
            subtitle,
            end,
            children,
            header_hidden,
        } = self;
        provide((), move || {
            let cx = nav_cx();
            let title = text(title).style(|| {
                style()
                    .weight(tokens().typography.weight_semibold)
                    .single_line()
            });
            let header = row((
                column((title, subtitle.map(text))).style(style().grow(1).min_width(0)),
                end,
            ))
            .hidden(move || header_hidden || cx.rail())
            .style(|| {
                let t = tokens();
                style()
                    .direction(logical_row())
                    .padding_y(t.spacing.s1)
                    .padding_x(t.spacing.s2)
                    .gap(t.spacing.s2)
                    .align_items(Align::Center)
                    .font_size(t.typography.supporting.size_px)
                    .line_height(
                        t.typography.supporting.line_height_px / t.typography.supporting.size_px,
                    )
                    .color(t.colors.text_secondary)
            });
            root.base_style(move || {
                let t = tokens();
                let base = style()
                    .direction(Direction::Column)
                    .padding_y(t.spacing.s1)
                    .gap(t.spacing.s0_5);
                if cx.rail() {
                    base.align_items(Align::Center)
                } else {
                    base
                }
            })
            .child((header, children))
        })
    }
}

pub(crate) fn nav_is_rail() -> Value<bool> {
    let cx = nav_cx();
    kivact::memo(move || cx.rail())
}
