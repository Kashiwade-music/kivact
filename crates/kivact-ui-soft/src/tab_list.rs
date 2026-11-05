//! Role: Render Neutral tabs, manual keyboard selection and overflow menus.
use crate::scrollable_area::use_scroll_overflow;
use crate::side_navigation::items::logical_row;
use crate::stack::{ContainerInset, rtl};

use kivact::draw::linear_gradient;
use kivact::geometry::Point;
use kivact::interaction::ListActivation;
use kivact::interaction::ListItem;
use kivact::interaction::use_menu;
use kivact::interaction::use_tabs;
use kivact::motion::Transition;
use kivact::state::IntoBinding;
use kivact::state::ItemKey;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ScrollAxes;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::scroll;
use kivact::view::stack;
use kivact::view::text;

use kivact::interaction::MenuEntry;
use kivact::state::Binding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::Value;
use kivact::state::effect_on;
use kivact::state::memo;
use kivact::state::provide;
use kivact::state::signal;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::Position;
use kivact::style::ScrollbarWidth;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::ButtonVariant;
use kivact::ui::ControlSize;
use kivact::ui::button;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::NodeRef;
use kivact::view::Placement;
use kivact::view::Role;
use kivact::view::ScrollBehavior;
use kivact::view::View;
use kivact::view::dynamic;
use kivact::view::for_each;
use kivact::view::node_ref;
use kivact::view::overlay;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabLayout {
    #[default]
    Hug,
    Fill,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabPattern {
    #[default]
    Navigation,
    Tabs,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabOverflow {
    #[default]
    Scroll,
    Visible,
}

pub struct Tab<K> {
    root: Node,
    key: K,
    label: Value<String>,
    icon: Option<View>,
    selected_icon: Option<View>,
    label_hidden: Value<bool>,
    end: Option<View>,
    disabled: Value<bool>,
    panel: Option<NodeRef>,
}

pub fn tab<K>(key: K, label: impl IntoText) -> Tab<K> {
    Tab {
        root: node(),
        key,
        label: label.into_text_value(),
        icon: None,
        selected_icon: None,
        label_hidden: (false).into_value(),
        end: None,
        disabled: false.into_value(),
        panel: None,
    }
}

impl<K> Tab<K> {
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }
    pub fn selected_icon(mut self, icon: impl IntoView) -> Self {
        self.selected_icon = Some(icon.into_view());
        self
    }
    pub fn label_hidden(mut self, hidden: impl IntoValue<bool>) -> Self {
        self.label_hidden = hidden.into_value();
        self
    }
    pub fn end_content(mut self, end: impl IntoView) -> Self {
        self.end = Some(provide((), move || end.into_view()));
        self
    }
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn panel(mut self, panel: NodeRef) -> Self {
        self.panel = Some(panel);
        self
    }
}

impl<K> NodeBuilder for Tab<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone)]
pub struct TabMenuOption<K> {
    key: K,
    label: Value<String>,
    icon: Option<ViewFn>,
    disabled: bool,
}

impl<K> TabMenuOption<K> {
    pub fn new(key: K, label: impl IntoText) -> Self {
        Self {
            key,
            label: label.into_text_value(),
            icon: None,
            disabled: false,
        }
    }
    pub fn icon<V: IntoView>(mut self, icon: impl IntoViewFn<V>) -> Self {
        self.icon = Some(icon.into_view_fn());
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

pub struct TabMenu<K> {
    root: Node,
    label: Value<String>,
    options: Vec<TabMenuOption<K>>,
}

pub fn tab_menu<K>(label: impl IntoText, options: Vec<TabMenuOption<K>>) -> TabMenu<K> {
    TabMenu {
        root: node(),
        label: label.into_text_value(),
        options,
    }
}

impl<K> NodeBuilder for TabMenu<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

pub enum TabEntry<K> {
    Tab(Tab<K>),
    Menu(TabMenu<K>),
}

impl<K> From<Tab<K>> for TabEntry<K> {
    fn from(tab: Tab<K>) -> Self {
        Self::Tab(tab)
    }
}

impl<K> From<TabMenu<K>> for TabEntry<K> {
    fn from(menu: TabMenu<K>) -> Self {
        Self::Menu(menu)
    }
}

#[derive(Clone, Copy)]
struct Look {
    size: ControlSize,
    layout: TabLayout,
    divider: bool,
}

pub struct TabList<K: ItemKey> {
    root: Node,
    label: Value<String>,
    binding: Binding<K>,
    entries: Vec<TabEntry<K>>,
    look: Look,
    pattern: Value<TabPattern>,
    overflow: Value<TabOverflow>,
    full_bleed: Value<bool>,
    content: Option<kivact::view::ViewFn<K>>,
}

pub fn tab_list<K: ItemKey>(name: impl IntoText, binding: impl IntoBinding<K>) -> TabList<K> {
    TabList {
        root: node(),
        label: name.into_text_value(),
        binding: binding.into_binding(),
        entries: Vec::new(),
        look: Look {
            size: ControlSize::Medium,
            layout: TabLayout::Hug,
            divider: false,
        },
        pattern: (TabPattern::Navigation).into_value(),
        overflow: (TabOverflow::Scroll).into_value(),
        full_bleed: (false).into_value(),
        content: None,
    }
}

impl<K: ItemKey> TabList<K> {
    pub fn tabs(mut self, tabs: impl IntoIterator<Item = impl Into<TabEntry<K>>>) -> Self {
        self.entries.extend(tabs.into_iter().map(Into::into));
        self
    }
    pub fn tab(mut self, tab: Tab<K>) -> Self {
        self.entries.push(tab.into());
        self
    }
    pub fn menu(mut self, menu: TabMenu<K>) -> Self {
        self.entries.push(menu.into());
        self
    }
    pub fn size(mut self, size: ControlSize) -> Self {
        self.look.size = size;
        self
    }
    pub fn layout(mut self, layout: TabLayout) -> Self {
        self.look.layout = layout;
        self
    }
    pub fn divider(mut self, divider: bool) -> Self {
        self.look.divider = divider;
        self
    }
    pub fn pattern(mut self, pattern: impl IntoValue<TabPattern>) -> Self {
        self.pattern = pattern.into_value();
        self
    }
    pub fn overflow(mut self, overflow: impl IntoValue<TabOverflow>) -> Self {
        self.overflow = overflow.into_value();
        self
    }
    pub fn full_bleed(mut self, full_bleed: impl IntoValue<bool>) -> Self {
        self.full_bleed = full_bleed.into_value();
        self
    }
    pub fn content<V: IntoView>(mut self, panel: impl Fn(K) -> V + 'static) -> Self {
        self.content = Some(panel.into());
        self
    }
}

impl<K: ItemKey> NodeBuilder for TabList<K> {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

struct FocusMember {
    reference: NodeRef,
    selected: Value<bool>,
    disabled: Value<bool>,
}

#[derive(Clone)]
struct FocusTrack {
    members: Rc<[FocusMember]>,
    focused_within: Value<bool>,
    roving: Signal<Option<usize>>,
}

impl FocusTrack {
    fn focusable(&self, index: usize) -> bool {
        if self.members[index].disabled.get() {
            return false;
        }
        let preferred = self
            .roving
            .get()
            .filter(|index| self.focused_within.get() && !self.members[*index].disabled.get());
        preferred
            .or_else(|| {
                self.members
                    .iter()
                    .position(|member| member.selected.get() && !member.disabled.get())
            })
            .or_else(|| {
                self.members
                    .iter()
                    .position(|member| !member.disabled.get())
            })
            == Some(index)
    }

    fn key(&self, index: usize, event: KeyEvent) -> Handled {
        if event.modifiers().alt || event.modifiers().ctrl || event.modifiers().meta {
            return Handled::No;
        }
        let len = self.members.len();
        let direction = match event.key() {
            Key::ArrowRight => {
                if rtl() {
                    len - 1
                } else {
                    1
                }
            }
            Key::ArrowLeft => {
                if rtl() {
                    1
                } else {
                    len - 1
                }
            }
            Key::ArrowDown => 1,
            Key::ArrowUp => len - 1,
            Key::Home => 0,
            Key::End => len - 1,
            _ => return Handled::No,
        };
        let next = match event.key() {
            Key::Home => (0..len).find(|i| !self.members[*i].disabled.get()),
            Key::End => (0..len).rev().find(|i| !self.members[*i].disabled.get()),
            _ => (1..=len)
                .map(|step| (index + step * direction) % len)
                .find(|i| !self.members[*i].disabled.get()),
        };
        if let Some(next) = next {
            self.roving.set(Some(next));
            self.members[next].reference.focus();
        }
        Handled::Yes
    }
}

impl<K: ItemKey> IntoView for TabList<K> {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            binding,
            mut entries,
            look,
            pattern,
            overflow,
            full_bleed,
            content,
        } = self;
        provide((), move || {
            if pattern.get() == TabPattern::Tabs {
                entries.retain(|entry| matches!(entry, TabEntry::Tab(_)));
            }
            let strip_ref = node_ref();
            let viewport_ref = node_ref();
            let inset = if full_bleed.get() {
                try_use_context::<ContainerInset>()
                    .map(|value| value.get())
                    .unwrap_or_default()
            } else {
                ContainerInset::default()
            };
            let options = entries
                .iter()
                .filter_map(|entry| match entry {
                    TabEntry::Tab(tab) => Some((tab.key.clone(), tab.label, tab.disabled)),
                    TabEntry::Menu(_) => None,
                })
                .collect::<Vec<_>>();
            let behavior = (pattern.get() == TabPattern::Tabs).then(|| {
                use_tabs(
                    move || {
                        options
                            .iter()
                            .map(|(key, label, disabled)| {
                                ListItem::new(key.clone(), *label).disabled(disabled.get())
                            })
                            .collect::<Vec<_>>()
                    },
                    binding,
                )
                .activation(ListActivation::Manual)
            });
            let members = entries
                .iter()
                .map(|entry| {
                    let (selected, disabled) = match entry {
                        TabEntry::Tab(tab) => {
                            let key = tab.key.clone();
                            (memo(move || binding.get() == key), tab.disabled)
                        }
                        TabEntry::Menu(menu) => {
                            let keys = menu
                                .options
                                .iter()
                                .map(|option| option.key.clone())
                                .collect::<Vec<_>>();
                            (
                                memo(move || keys.contains(&binding.get())),
                                false.into_value(),
                            )
                        }
                    };
                    FocusMember {
                        reference: node_ref(),
                        selected,
                        disabled,
                    }
                })
                .collect::<Vec<_>>();
            let focus = FocusTrack {
                members: members.into(),
                focused_within: strip_ref.focus_within(),
                roving: signal(None),
            };
            let scrollable = overflow.get() == TabOverflow::Scroll;
            let children = entries
                .into_iter()
                .enumerate()
                .map(|(index, entry)| {
                    let member = &focus.members[index];
                    let reference = member.reference;
                    let selected = member.selected;
                    let active_ref = reference.focused();
                    if scrollable {
                        let geometry = reference.rect();
                        effect_on(
                            move || {
                                geometry.get().is_some() && (active_ref.get() || selected.get())
                            },
                            move |active| {
                                if *active {
                                    reference.scroll_into_view();
                                }
                            },
                        );
                    }
                    match entry {
                        TabEntry::Tab(tab) => {
                            let Tab {
                                root,
                                key,
                                label,
                                icon,
                                selected_icon,
                                label_hidden,
                                end,
                                disabled,
                                panel,
                            } = tab;
                            let has_selected_icon = selected_icon.is_some();
                            let standard_icon = icon.map(|icon| {
                                tab_glyph(icon, look.size, move || {
                                    selected.get() && has_selected_icon
                                })
                            });
                            let selected_icon = selected_icon
                                .map(|icon| tab_glyph(icon, look.size, move || !selected.get()));
                            let title =
                                (!label_hidden.get()).then(|| stable_label(label, selected));
                            let mut root = root
                                .node_ref(reference)
                                .name(label)
                                .disabled(disabled)
                                .with_node(move |node| {
                                    node.base_style(move || {
                                        tab_style(look, selected.get(), disabled.get())
                                    })
                                })
                                .child((
                                    standard_icon,
                                    selected_icon,
                                    title,
                                    end,
                                    indicator(look, selected, false),
                                ));
                            if let Some(behavior) = behavior {
                                root = root.with_node(move |node| node.part(behavior.item(key)));
                            } else {
                                let state = focus.clone();
                                let keys = focus.clone();
                                root = root
                                    .with_node(move |node| node.role(Role::Button))
                                    .with_node(move |node| node.selected(selected))
                                    .focusable(move || state.focusable(index))
                                    .on_key(move |event| keys.key(index, event))
                                    .on_press(move || {
                                        if !disabled.get() {
                                            binding.set(key.clone());
                                        }
                                    });
                            }
                            if let Some(panel) = panel {
                                root = root.with_node(move |node| node.controls(panel));
                            }
                            root.into_view()
                        }
                        TabEntry::Menu(menu) => {
                            tab_menu_view(menu, binding, look, focus.clone(), index)
                        }
                    }
                })
                .collect::<Vec<_>>();
            let strip = row(children).node_ref(strip_ref).style(move || {
                let t = kivact::style::use_tokens::<crate::Tokens>();
                let (left, right) = inset.sides();
                let mut base = style()
                    .direction(logical_row())
                    .gap(t.spacing.s0_5)
                    .min_width(0)
                    .padding_left((left - t.spacing.s3).max(0.0))
                    .padding_right((right - t.spacing.s3).max(0.0));
                if look.layout == TabLayout::Fill {
                    base = base.width(pct(100));
                } else if scrollable {
                    base = base.width(Length::MaxContent).min_width(pct(100));
                }
                if look.divider {
                    base = base.padding_bottom(t.spacing.s1);
                }
                base
            });
            let strip = if let Some(behavior) = behavior {
                strip
                    .with_node(move |node| node.part(behavior.list()))
                    .name(label)
            } else {
                strip
                    .with_node(move |node| node.role(Role::Navigation))
                    .name(label)
            };
            let strip = if scrollable {
                let state = use_scroll_overflow(viewport_ref, strip_ref);
                let viewport = scroll(strip)
                    .axes(ScrollAxes::Horizontal)
                    .node_ref(viewport_ref)
                    .style(move || {
                        let base = style()
                            .scrollbar_width(ScrollbarWidth::None)
                            .min_width(0)
                            .max_width(pct(100));
                        let start = state.start.get();
                        let end = state.end.get();
                        if !start && !end {
                            return base;
                        }
                        let mut mask = linear_gradient(if rtl() { 270 } else { 90 });
                        if start {
                            mask = mask.stop(Color::TRANSPARENT, 0).stop(Color::BLACK, 32);
                        } else {
                            mask = mask.stop(Color::BLACK, 0);
                        }
                        if end {
                            mask = mask
                                .stop(Color::BLACK, pct(100) - 32)
                                .stop(Color::TRANSPARENT, pct(100));
                        } else {
                            mask = mask.stop(Color::BLACK, pct(100));
                        }
                        base.mask(mask)
                    });
                let arrows = if kivact::platform::capabilities().get().touch {
                    None
                } else {
                    Some((
                        tab_scroll_arrow(look.size, viewport_ref, state.start, false),
                        tab_scroll_arrow(look.size, viewport_ref, state.end, true),
                    ))
                };
                node()
                    .style(style().position(Position::Relative))
                    .child((viewport, arrows))
                    .into_view()
            } else {
                strip.into_view()
            };
            let strip = node()
                .style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let (left, right) = inset.sides();
                    let base = style()
                        .direction(Direction::Column)
                        .margin_left(-left)
                        .margin_right(-right)
                        .min_width(0);
                    if look.divider {
                        base.border_bottom(t.sizes.border_width, t.colors.border)
                    } else {
                        base
                    }
                })
                .child(strip);
            let panel = content.map(|content| {
                dynamic(binding, move |key| {
                    let panel = node().child(content.call(key.clone()));
                    if let Some(behavior) = behavior {
                        panel.with_node(move |node| node.part(behavior.panel(key)))
                    } else {
                        panel
                    }
                })
            });
            root.with_node(move |node| {
                node.base_style(style().direction(Direction::Column).min_width(0))
            })
            .child((strip, panel))
        })
    }
}

fn tab_style(look: Look, selected: bool, disabled: bool) -> kivact::style::Style {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let role = t.typography.label;
    let base = style()
        .position(Position::Relative)
        .direction(logical_row())
        .height(tab_height(look.size))
        .padding_x(t.spacing.s3)
        .gap(t.spacing.s1)
        .align_items(Align::Center)
        .justify_content(Justify::Center)
        .radius(t.radius.element)
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .color(if selected {
            t.colors.text_primary
        } else {
            t.colors.text_secondary
        })
        .text_wrap(false)
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
        .transition(
            TransitionProp::Paint,
            Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
        );
    let base = if look.layout == TabLayout::Fill {
        base.grow(1).basis(0).min_width(0)
    } else {
        base.shrink(0)
    };
    if disabled {
        base
    } else {
        base.hover(style().background(t.colors.overlay_hover))
            .pressed(style().background(t.colors.overlay_pressed))
    }
}

fn tab_height(size: ControlSize) -> f64 {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    match size {
        ControlSize::Small => t.sizes.control_sm,
        ControlSize::Large => t.sizes.control_lg,
        _ => t.sizes.control_md,
    }
}

fn stable_label(label: Value<String>, selected: Value<bool>) -> Node {
    stack((
        text(label)
            .with_node(move |node| node.role(Role::Generic))
            .style(|| {
                style()
                    .opacity(0.0)
                    .weight(
                        kivact::style::use_tokens::<crate::Tokens>()
                            .typography
                            .weight_semibold,
                    )
                    .text_wrap(false)
            }),
        text(label).style(move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            style()
                .weight(if selected.get() {
                    t.typography.weight_semibold
                } else {
                    t.typography.weight_normal
                })
                .text_wrap(false)
        }),
    ))
}

fn tab_glyph(glyph: View, size: ControlSize, hidden: impl Fn() -> bool + 'static) -> Node {
    node()
        .hidden(hidden)
        .style(move || {
            let px = match size {
                ControlSize::Small => 14,
                ControlSize::Large => 18,
                _ => 16,
            };
            style()
                .size(px, px)
                .shrink(0)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
        })
        .child(glyph)
}

fn indicator(look: Look, selected: Value<bool>, menu: bool) -> Node {
    node().hit_testable(false).style(move || {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        let gap = if look.divider {
            t.spacing.s1 + t.sizes.border_width
        } else {
            t.sizes.border_width
        };
        style()
            .position(Position::Absolute)
            .bottom(-gap)
            .left(t.spacing.s3)
            .right(t.spacing.s3)
            .height(t.spacing.s0_5)
            .radius(t.radius.full)
            .opacity(if selected.get() { 1.0 } else { 0.0 })
            .background(if menu {
                t.colors.icon_primary
            } else {
                t.colors.accent
            })
            .transition(
                TransitionProp::Paint,
                Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
            )
    })
}

fn tab_scroll_arrow(
    size: ControlSize,
    viewport: NodeRef,
    shown: Value<bool>,
    forward: bool,
) -> impl IntoView {
    let physical_right = forward != rtl();
    let (name, glyph) = if physical_right {
        ("Scroll right", "chevron-right")
    } else {
        ("Scroll left", "chevron-left")
    };
    button(name)
        .icon(icon(glyph).style(|| style().size(16, 16)))
        .variant(ButtonVariant::Ghost)
        .size(size)
        .hide_label(true)
        .focusable(false)
        .hidden(move || !shown.get())
        .on_press(move || {
            if let Some(rect) = viewport.rect().get() {
                let delta = rect.width * if physical_right { 0.8 } else { -0.8 };
                viewport.scroll_by(
                    Point { x: delta, y: 0.0 },
                    if kivact::platform::is_motion_reduced().get() {
                        ScrollBehavior::Instant
                    } else {
                        ScrollBehavior::Smooth(kivact::motion::Easing::EaseOut)
                    },
                );
            }
        })
        .style(move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            let base = style()
                .position(Position::Absolute)
                .top(0)
                .z_index(1)
                .size(tab_height(size), tab_height(size))
                .radius(t.radius.full)
                .background(t.colors.bg_popover)
                .color(t.colors.text_secondary);
            let mut base = if physical_right {
                base.right(0)
            } else {
                base.left(0)
            };
            for shadow in &t.shadows.low {
                base = base.shadow(*shadow);
            }
            base
        })
}

fn tab_menu_view<K: ItemKey>(
    menu: TabMenu<K>,
    binding: Binding<K>,
    look: Look,
    focus: FocusTrack,
    index: usize,
) -> View {
    let TabMenu {
        root,
        label,
        options,
    } = menu;
    let options: Rc<[TabMenuOption<K>]> = options.into();
    let entries = options.clone();
    let options_for_label = options.clone();
    let shown_label = memo(move || {
        let selected = binding.get();
        options_for_label
            .iter()
            .find(|option| option.key == selected)
            .map_or_else(|| label.get(), |option| option.label.get())
    });
    let behavior = use_menu(move || {
        let selected = binding.get();
        entries
            .iter()
            .enumerate()
            .map(|(index, option)| {
                MenuEntry::radio(index, option.label, option.key == selected)
                    .disabled(option.disabled)
            })
            .collect::<Vec<_>>()
    })
    .on_activate({
        let options = options.clone();
        move |index: usize| binding.set(options[index].key.clone())
    });
    let part = behavior.trigger();
    let anchor = node_ref();
    let selected = focus.members[index].selected;
    let open = behavior.is_open();
    let state = focus.clone();
    let trigger = root
        .with_node(move |node| node.part(part))
        .node_ref(anchor)
        .node_ref(focus.members[index].reference)
        .name(shown_label)
        .focusable(move || state.focusable(index))
        .on_key(move |event| {
            if event.key() == Key::ArrowDown || event.key() == Key::ArrowUp {
                Handled::No
            } else {
                focus.key(index, event)
            }
        })
        .with_node(move |node| node.base_style(move || tab_style(look, selected.get(), false)))
        .child((
            stable_label(shown_label, selected),
            icon("chevron-down").style(move || {
                style()
                    .size(16, 16)
                    .rotate(if open.get() { 180 } else { 0 })
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(
                            kivact::style::use_tokens::<crate::Tokens>().motion.fast as f64,
                            kivact::style::use_tokens::<crate::Tokens>()
                                .motion
                                .ease_standard,
                        ),
                    )
            }),
            indicator(look, selected, true),
        ));
    let menu = behavior.show(move || {
        let options = options.clone();
        let rows = for_each(
            behavior.entries(),
            |(id, _)| id.clone(),
            move |entry| {
                let index = *entry.get().1.key().expect("radio item");
                let part = behavior.item(index);
                let state = part.state();
                let option = &options[index];
                let selected_key = option.key.clone();
                let selected = memo(move || binding.get() == selected_key);
                let icon = option
                    .icon
                    .as_ref()
                    .map(|icon| tab_glyph(icon.call(()), ControlSize::Medium, || false));
                let check = icon_check().hidden(move || !selected.get());
                row((
                    icon,
                    text(option.label).style(style().grow(1).single_line()),
                    check,
                ))
                .with_node(move |node| node.part(part))
                .style(move || {
                    let t = kivact::style::use_tokens::<crate::Tokens>();
                    let role = t.typography.body;
                    let input = state.get().input;
                    let base = style()
                        .direction(logical_row())
                        .padding_x(t.spacing.s3)
                        .padding_y(t.spacing.s2)
                        .align_items(Align::Center)
                        .gap(t.spacing.s2)
                        .radius(t.radius.element)
                        .font_size(role.size_px)
                        .line_height(role.line_height_px / role.size_px)
                        .color(t.colors.text_primary)
                        .weight(if selected.get() {
                            t.typography.weight_medium
                        } else {
                            role.weight
                        })
                        .disabled(style().opacity(t.sizes.disabled_opacity));
                    if input.hovered || input.focused {
                        base.background(t.colors.overlay_hover)
                    } else {
                        base
                    }
                })
            },
        );
        let header = text(label).style(|| {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            crate::nav_heading::supporting()
                .padding_x(t.spacing.s3)
                .padding_y(t.spacing.s1)
                .weight(t.typography.weight_semibold)
        });
        let panel = column((header, rows)).style(|| {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            let mut base = style()
                .padding(t.spacing.s1)
                .gap(t.spacing.s0_5)
                .min_width(160)
                .background(t.colors.bg_popover)
                .radius(t.radius.container);
            for shadow in &t.shadows.low {
                base = base.shadow(*shadow);
            }
            base
        });
        overlay(anchor, Placement::Below, panel)
            .offset(4)
            .with_node(move |node| node.part(behavior.content()))
            .name(label)
    });
    (trigger, menu).into_view()
}

fn icon_check() -> Node {
    icon("check").style(|| {
        let t = kivact::style::use_tokens::<crate::Tokens>();
        style()
            .size(t.sizes.icon_sm, t.sizes.icon_sm)
            .color(t.colors.accent)
    })
}

kivact::state::constant_value!(TabOverflow, TabPattern);
