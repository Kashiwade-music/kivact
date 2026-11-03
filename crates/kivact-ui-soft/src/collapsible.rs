//! Role: Render Neutral disclosure triggers and coordinate groups.
use crate::stack::rtl;
use crate::tokens;
use kivact::control::{Node, NodeBuilder, column, icon, node, row, text};
use kivact::interaction::use_disclosure;
use kivact::motion::Transition;
use kivact::primitive::NodeExt;
use kivact::style::{Align, Direction, TransitionProp, pct, style};
use kivact::view::{IntoText, IntoValue, IntoView, View, provide, show, try_context};
use kivact::{Binding, Value, binding, signal};
use std::rc::Rc;

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

pub struct Collapsible {
    root: Node,
    label: View,
    open: Binding<bool>,
    disabled: Value<bool>,
    content: Option<Rc<dyn Fn() -> View>>,
    chevron: Option<ChevronPosition>,
    key: Option<String>,
}

pub fn collapsible(trigger: impl IntoText, open: impl Into<Binding<bool>>) -> Collapsible {
    Collapsible {
        root: node(),
        label: text(trigger).into_view(),
        open: open.into(),
        disabled: false.into_value(),
        content: None,
        chevron: None,
        key: None,
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
    pub fn trigger_view<V: IntoView>(mut self, trigger: impl FnOnce() -> V + 'static) -> Self {
        self.label = provide((), trigger);
        self
    }
    pub fn content<V: IntoView>(mut self, content: impl Fn() -> V + 'static) -> Self {
        self.content = Some(Rc::new(move || content().into_view()));
        self
    }
}

impl NodeBuilder for Collapsible {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Collapsible {
    fn into_view(self) -> View {
        let Self {
            root,
            label,
            open,
            disabled,
            content,
            chevron,
            key,
        } = self;
        provide((), move || {
            let group = try_context::<GroupCx>();
            let position = chevron
                .or_else(|| group.as_ref().map(|group| group.chevron))
                .unwrap_or_default();
            let density = group.as_ref().and_then(|group| group.density);
            let open = match (group, key) {
                (Some(group), Some(key)) => group.open_binding(key),
                _ => open,
            };
            let disclosure = use_disclosure(open);
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
            .part(disclosure.trigger())
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
            let details = show(disclosure.shown(), move || {
                node()
                    .part(disclosure.content())
                    .style(move || {
                        let t = tokens();
                        style()
                            .direction(Direction::Column)
                            .padding_top(t.spacing.s1)
                            .padding_bottom(padding_for(density))
                            .font_size(t.typography.body.size_px)
                            .line_height(
                                t.typography.body.line_height_px / t.typography.body.size_px,
                            )
                            .color(t.colors.text_primary)
                    })
                    .child(content.as_ref().map(|content| content()))
            });
            root.base_style(style().direction(Direction::Column))
                .child((trigger, details))
        })
    }
}

fn padding_for(density: Option<DisclosureDensity>) -> f32 {
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
    dividers: bool,
}

pub fn collapsible_group(
    open: impl Into<Binding<Vec<String>>>,
    items: Vec<Collapsible>,
) -> CollapsibleGroup {
    CollapsibleGroup {
        root: column(()),
        items,
        cx: GroupCx {
            open: open.into(),
            mode: GroupMode::Single,
            density: None,
            chevron: ChevronPosition::End,
        },
        dividers: false,
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
    pub fn dividers(mut self, dividers: bool) -> Self {
        self.dividers = dividers;
        self
    }
}

impl NodeBuilder for CollapsibleGroup {
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
        if dividers {
            cx.density.get_or_insert_default();
        }
        root.child(provide(cx, move || {
            items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    if dividers && index > 0 {
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
