//! Role: Render Neutral disclosure triggers and coordinate groups.
use crate::stack::rtl;
use crate::tokens;
use kivact::interaction::use_content_reveal;
use kivact::interaction::use_disclosure;
use kivact::motion::Spring;
use kivact::motion::Transition;
use kivact::state::IntoBinding;
use kivact::view::IntoViewFn;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::ViewFn;
use kivact::view::column;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::state::Binding;
use kivact::state::IntoText;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::state::binding;
use kivact::state::memo;
use kivact::state::provide;
use kivact::state::signal;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::View;
use kivact::view::node_ref;
use kivact::view::show;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChevronPosition {
    Start,
    #[default]
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisclosureDensity {
    Compact,
    #[default]
    Balanced,
    Spacious,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupMode {
    #[default]
    Single,
    Multiple,
}

pub struct CollapsibleConfig {
    pub enabled: bool,
    pub default_open: bool,
    pub open: Option<Binding<bool>>,
    pub value: Option<String>,
}

impl Default for CollapsibleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_open: true,
            open: None,
            value: None,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CollapsibleState {
    pub is_enabled: bool,
    pub is_open: Value<bool>,
    open: Binding<bool>,
}

impl CollapsibleState {
    pub fn toggle(self) {
        if self.is_enabled {
            self.open.set(!self.open.get());
        }
    }
}

pub fn use_collapsible(config: CollapsibleConfig) -> CollapsibleState {
    let group = try_use_context::<GroupCx>().map(|value| value.get());
    let open = match (group, config.value) {
        (Some(group), Some(key)) => group.open_binding(key),
        _ => config
            .open
            .unwrap_or_else(|| signal(config.default_open).into_binding()),
    };
    CollapsibleState {
        is_enabled: config.enabled,
        is_open: memo(move || !config.enabled || open.get()),
        open,
    }
}

pub struct Collapsible {
    root: Node,
    label: View,
    label_name: Value<String>,
    open: Binding<bool>,
    disabled: Value<bool>,
    content: Option<ViewFn>,
    chevron: Option<ChevronPosition>,
    key: Option<String>,
    keep_mounted: bool,
    animate_layout: Value<bool>,
}

pub fn collapsible(trigger: impl IntoText, open: impl IntoBinding<bool>) -> Collapsible {
    let label_name = trigger.into_text_value();
    Collapsible {
        root: node(),
        label: text(label_name).into_view(),
        label_name,
        open: open.into_binding(),
        disabled: false.into_value(),
        content: None,
        chevron: None,
        key: None,
        keep_mounted: false,
        animate_layout: (false).into_value(),
    }
}

pub fn collapsible_default(trigger: impl IntoText, open: bool) -> Collapsible {
    collapsible(trigger, signal(open))
}

impl Collapsible {
    pub fn disabled(mut self, disabled: impl IntoValue<bool>) -> Self {
        self.disabled = disabled.into_value();
        self
    }
    pub fn chevron(mut self, chevron: ChevronPosition) -> Self {
        self.chevron = Some(chevron);
        self
    }
    pub fn value(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
    pub fn trigger_view(mut self, trigger: impl IntoView) -> Self {
        self.label = provide((), move || trigger.into_view());
        self
    }
    pub fn keep_mounted(mut self) -> Self {
        self.keep_mounted = true;
        self
    }
    pub fn animate_layout(mut self, animate: impl IntoValue<bool>) -> Self {
        self.animate_layout = animate.into_value();
        self
    }
    pub fn content<V: IntoView>(mut self, content: impl IntoViewFn<V>) -> Self {
        self.content = Some(content.into_view_fn());
        self
    }
}

impl NodeBuilder for Collapsible {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Collapsible {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            label_name,
            open,
            disabled,
            content,
            chevron,
            key,
            keep_mounted,
            animate_layout,
        } = self;
        provide((), move || {
            let group = try_use_context::<GroupCx>().map(|value| value.get());
            let position = chevron
                .or_else(|| group.as_ref().map(|group| group.chevron))
                .unwrap_or_default();
            let density = group.as_ref().and_then(|group| group.density);
            let state = use_collapsible(CollapsibleConfig {
                open: Some(open),
                value: key,
                ..CollapsibleConfig::default()
            });
            let open = state.open;
            let mut disclosure = use_disclosure(open).disabled(disabled);
            let shown = disclosure.is_open();
            let reveal = animate_layout.get().then(|| {
                use_content_reveal(
                    shown,
                    Transition::spring(Spring {
                        stiffness: 157.91,
                        damping: 25.13,
                        mass: 1.0,
                    }),
                )
            });
            if let Some(reveal) = reveal {
                disclosure = disclosure.leaving(move || reveal.leaving());
            }
            let present =
                memo(move || reveal.map_or_else(|| shown.get(), |reveal| reveal.present()));
            let chevron = icon(match position {
                ChevronPosition::Start => "chevron-right",
                ChevronPosition::End => "chevron-down",
            })
            .style(move || {
                let t = tokens();
                let mirrored = position == ChevronPosition::Start && rtl();
                let degrees = match (position, open.get()) {
                    (ChevronPosition::Start, true) if mirrored => -90,
                    (ChevronPosition::Start, true) => 90,
                    (ChevronPosition::End, true) => 180,
                    (_, false) => 0,
                };
                style()
                    .size(t.typography.large.size_px, t.typography.large.size_px)
                    .shrink(0)
                    .color(t.colors.icon_secondary)
                    .scale_xy(if mirrored { -1.0 } else { 1.0 }, 1.0)
                    .rotate(degrees)
                    .transition(
                        TransitionProp::Paint,
                        Transition::tween(t.motion.fast as f64, t.motion.ease_standard),
                    )
            });
            let label = node().style(style().grow(1)).child(label);
            let trigger = row(match position {
                ChevronPosition::Start => (chevron.into_view(), label.into_view()),
                ChevronPosition::End => (label.into_view(), chevron.into_view()),
            })
            .name(label_name)
            .with_node(move |node| node.part(disclosure.trigger()))
            .disabled(disabled)
            .style(move || {
                let t = tokens();
                style()
                    .direction(if rtl() {
                        Direction::RowReverse
                    } else {
                        Direction::Row
                    })
                    .gap(t.spacing.s2)
                    .align_items(Align::Center)
                    .width(pct(100))
                    .font_size(t.typography.large.size_px)
                    .weight(t.typography.weight_semibold)
                    .color(t.colors.text_primary)
                    .padding_y(padding_for(density))
                    .pressed(style().background(t.colors.overlay_pressed))
                    .disabled(style().opacity(t.sizes.disabled_opacity))
                    .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
            });
            let body = move || {
                let region = node()
                    .with_node(move |node| node.part(disclosure.content()))
                    .style(move || {
                        let t = tokens();
                        style()
                            .direction(Direction::Column)
                            .font_size(t.typography.body.size_px)
                            .line_height(
                                t.typography.body.line_height_px / t.typography.body.size_px,
                            )
                            .color(t.colors.text_primary)
                    });
                let details = node()
                    .style(move || {
                        let t = tokens();
                        style()
                            .direction(Direction::Column)
                            .shrink(0)
                            .padding_top(t.spacing.s1)
                            .padding_bottom(padding_for(density))
                    })
                    .child(content.as_ref().map(|content| content.call(())));
                if let Some(reveal) = reveal {
                    let measured = node_ref();
                    let height = reveal.container_style(measured);
                    region
                        .style(style().clip(true))
                        .style(move || height.get())
                        .child(details.node_ref(measured))
                } else {
                    region.child(details)
                }
            };
            let details = if keep_mounted {
                body().into_view()
            } else {
                show(present, body).into_view()
            };
            root.with_node(move |node| node.base_style(style().direction(Direction::Column)))
                .child((trigger, details))
        })
    }
}

fn padding_for(density: Option<DisclosureDensity>) -> f64 {
    let s = tokens().spacing;
    match density {
        Some(DisclosureDensity::Compact) => s.s1,
        Some(DisclosureDensity::Balanced) => s.s2,
        Some(DisclosureDensity::Spacious) => s.s3,
        None => 0.0,
    }
}

#[derive(Clone)]
struct GroupCx {
    open: Binding<Vec<String>>,
    mode: GroupMode,
    density: Option<DisclosureDensity>,
    chevron: ChevronPosition,
}

impl GroupCx {
    fn open_binding(self, key: String) -> Binding<bool> {
        let read_key = key.clone();
        let open = self.open;
        binding(
            move || open.get().contains(&read_key),
            move |expanded| {
                let mut next = open.get();
                if expanded {
                    if self.mode == GroupMode::Single {
                        next = vec![key.clone()];
                    } else if !next.contains(&key) {
                        next.push(key.clone());
                    }
                } else {
                    next.retain(|current| current != &key);
                }
                open.set(next);
            },
        )
    }
}

pub struct CollapsibleGroup {
    root: Node,
    items: Vec<Collapsible>,
    cx: GroupCx,
    dividers: Value<bool>,
}

pub fn collapsible_group(
    open: impl IntoBinding<Vec<String>>,
    items: Vec<Collapsible>,
) -> CollapsibleGroup {
    CollapsibleGroup {
        root: column(()),
        items,
        cx: GroupCx {
            open: open.into_binding(),
            mode: GroupMode::Single,
            density: None,
            chevron: ChevronPosition::End,
        },
        dividers: (false).into_value(),
    }
}

impl CollapsibleGroup {
    pub fn mode(mut self, mode: GroupMode) -> Self {
        self.cx.mode = mode;
        self
    }
    pub fn density(mut self, density: DisclosureDensity) -> Self {
        self.cx.density = Some(density);
        self
    }
    pub fn chevron(mut self, chevron: ChevronPosition) -> Self {
        self.cx.chevron = chevron;
        self
    }
    pub fn dividers(mut self, dividers: impl IntoValue<bool>) -> Self {
        self.dividers = dividers.into_value();
        self
    }
}

impl NodeBuilder for CollapsibleGroup {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for CollapsibleGroup {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            mut cx,
            dividers,
        } = self;
        if dividers.get() {
            cx.density.get_or_insert_default();
        }
        root.child(provide(cx, move || {
            items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    if dividers.get() && index > 0 {
                        item.style(move || {
                            let t = tokens();
                            style().border_top(t.sizes.border_width, t.colors.border)
                        })
                    } else {
                        item
                    }
                })
                .collect::<Vec<_>>()
        }))
        .into_view()
    }
}
