//! Role: Neutral toolbar layout, shared roving focus and keyboard hints.
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::IntoChildren;

use std::time::Duration;

use crate::{SectionVariant, kbd, tokens};
use kivact::motion::Easing;
use kivact::motion::Transition;
use kivact::state::IntoText;
use kivact::state::Signal;
use kivact::state::effect;
use kivact::state::on_cleanup;
use kivact::state::provide;
use kivact::state::signal;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::Color;
use kivact::style::Direction;
use kivact::style::Justify;
use kivact::style::Length;
use kivact::style::Track;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::task::sleep;
use kivact::task::spawn;
use kivact::ui::ControlSize;
use kivact::view::Handled;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::NodeRef;
use kivact::view::Orientation;
use kivact::view::OverlayAlign;
use kivact::view::Placement;
use kivact::view::ReadingDirection;
use kivact::view::Role;
use kivact::view::View;
use kivact::view::node;
use kivact::view::node_ref;
use kivact::view::overlay;
use kivact::view::row;
use kivact::view::show_some;
use kivact::view::text;
use kivact::view::use_direction;

/// Neutral toolbar surface treatment.
/// Physical block and logical inline edges of the toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Top,
    Bottom,
    Start,
    End,
}

/// Toolbar chrome with start, center, and end slots.
pub struct Toolbar {
    root: Node,
    start: Option<Vec<View>>,
    center: Option<Vec<View>>,
    end: Option<Vec<View>>,
    size: Value<ControlSize>,
    gap: Value<f64>,
    orientation: Value<Orientation>,
    variant: Value<SectionVariant>,
    dividers: Vec<Edge>,
}

#[track_caller]
pub fn toolbar(label: impl IntoText) -> Toolbar {
    Toolbar {
        root: node()
            .with_node(move |node| node.role(Role::Toolbar))
            .name(label),
        start: None,
        center: None,
        end: None,
        size: (ControlSize::Medium).into_value(),
        gap: (4.0).into_value(),
        orientation: (Orientation::Horizontal).into_value(),
        variant: (SectionVariant::Transparent).into_value(),
        dividers: Vec::new(),
    }
}

impl Toolbar {
    pub fn start(mut self, start: impl IntoChildren) -> Self {
        self.start = Some(start.into_views());
        self
    }

    pub fn center(mut self, center: impl IntoChildren) -> Self {
        self.center = Some(center.into_views());
        self
    }

    pub fn end(mut self, end: impl IntoChildren) -> Self {
        self.end = Some(end.into_views());
        self
    }

    pub fn size(mut self, size: impl IntoValue<ControlSize>) -> Self {
        self.size = size.into_value();
        self
    }

    pub fn gap(mut self, gap: impl IntoValue<f64>) -> Self {
        self.gap = gap.into_value();
        self
    }

    pub fn orientation(mut self, orientation: impl IntoValue<Orientation>) -> Self {
        self.orientation = orientation.into_value();
        self
    }

    pub fn variant(mut self, variant: impl IntoValue<SectionVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }

    pub fn dividers(mut self, edges: &[Edge]) -> Self {
        self.dividers = edges.to_vec();
        self
    }
}

impl NodeBuilder for Toolbar {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Registration {
    slot: usize,
    reference: NodeRef,
    focus_visible: kivact::state::Value<bool>,
}

#[derive(Clone, Copy)]
struct Registrations(Signal<Vec<Registration>>, Signal<usize>);

/// Register a control for the toolbar's keyboard hint. All focusable controls
/// in a slot participate in the toolbar's arrow navigation.
#[track_caller]
pub fn toolbar_item<T: NodeBuilder>(item: T) -> T {
    let reference = node_ref();
    let focus_visible = reference.focus_visible();
    item.with_node(move |node| {
        node.node_ref(reference).on_attach(move |_| {
            let Some(Registrations(items, next)) =
                try_use_context::<Registrations>().map(|value| value.get())
            else {
                return;
            };
            let slot = next.get_untracked();
            next.set(slot + 1);
            items.update(|items| {
                items.push(Registration {
                    slot,
                    reference,
                    focus_visible,
                })
            });
            on_cleanup(move || items.update(|items| items.retain(|item| item.slot != slot)));
        })
    })
}

impl IntoView for Toolbar {
    fn into_view(self) -> View {
        let registry = Registrations(signal(Vec::new()), signal(0));
        let vertical = self.orientation.get() == Orientation::Vertical;
        let rtl = use_direction().get() == ReadingDirection::Rtl;
        let variant = self.variant.get();
        let dividers = self.dividers;
        let orientation = self.orientation.get();
        let gap = self.gap.get();
        let has_center = self.center.is_some();
        let size = self.size.get();
        let height = match size {
            ControlSize::ExtraSmall | ControlSize::Small => 28,
            ControlSize::Medium => 32,
            ControlSize::Large => 36,
        };
        provide(registry, move || {
            let direction = match (vertical, rtl) {
                (true, _) => Direction::Column,
                (false, true) => Direction::RowReverse,
                _ => Direction::Row,
            };
            let reverse_grid = has_center && rtl && !vertical;
            let make = |slot: Option<Vec<View>>, alignment, grid_column: i32| {
                kivact::ui::provide_control_size(size, move || {
                    let base = style()
                        .direction(direction)
                        .gap(gap)
                        .min_width(0)
                        .align_items(Align::Center)
                        .justify_content(alignment);
                    let base = if reverse_grid {
                        base.grid_column(grid_column)
                    } else {
                        base
                    };
                    row(slot).with_node(move |node| node.base_style(base))
                })
            };
            let start = make(
                self.start,
                if rtl && !vertical {
                    Justify::End
                } else {
                    Justify::Start
                },
                3,
            );
            let center = has_center.then(|| make(self.center, Justify::Center, 2));
            let end = make(
                self.end,
                if rtl && !vertical {
                    Justify::Start
                } else {
                    Justify::End
                },
                1,
            );
            let Registrations(items, _) = registry;
            let focused = move || {
                items.with(|items| {
                    items.iter().find_map(|registered| {
                        registered
                            .focus_visible
                            .get()
                            .then_some(registered.reference)
                    })
                })
            };
            let (hint_anchor, hint) = keyboard_hint(vertical, focused);
            let root = self
                .root
                .orientation(orientation)
                .roving_focus(orientation)
                .on_key(move |event| {
                    if matches!(
                        event.key(),
                        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight
                    ) {
                        hint_anchor.set(None);
                    }
                    Handled::No
                })
                .with_node(move |node| {
                    node.base_style(move || {
                        let t = tokens();
                        let c = &t.colors;
                        let fill = match variant {
                            SectionVariant::Section => c.bg_surface,
                            SectionVariant::Transparent => Color::TRANSPARENT,
                            SectionVariant::Muted => c.bg_muted,
                        };
                        let mut base = style()
                            .background(fill)
                            .padding(12)
                            .width(pct(100))
                            .min_height(height + 24)
                            .direction(direction)
                            .align_items(if vertical {
                                Align::Stretch
                            } else {
                                Align::Center
                            });
                        if has_center && !vertical {
                            base = base
                                .display(kivact::style::Display::Grid)
                                .grid_template_columns((
                                    Track::minmax(0, Track::fr(1)),
                                    Length::Auto,
                                    Track::minmax(0, Track::fr(1)),
                                ));
                        } else {
                            base = base.justify_content(Justify::SpaceBetween);
                        }
                        for edge in &dividers {
                            base = match edge {
                                Edge::Top => base.border_top(1, c.border),
                                Edge::Bottom => base.border_bottom(1, c.border),
                                Edge::Start if !rtl => base.border_left(1, c.border),
                                Edge::End if rtl => base.border_left(1, c.border),
                                _ => base.border_right(1, c.border),
                            };
                        }
                        base
                    })
                });
            root.child((start, center, end, hint))
        })
    }
}

pub(crate) fn keyboard_hint(
    vertical: bool,
    focused: impl Fn() -> Option<NodeRef> + 'static,
) -> (Signal<Option<NodeRef>>, View) {
    let anchor = signal(None);
    let seen = signal(false);
    effect(move || {
        let focused = focused();
        if let Some(focused) = focused {
            if !seen.get_untracked() {
                seen.set(true);
                anchor.set(Some(focused));
                spawn(async move {
                    sleep(Duration::from_millis(3000)).await;
                    anchor.set(None);
                });
            }
        } else {
            anchor.set(None);
        }
    });
    let hint = show_some(anchor, move |anchor| {
        let keys = if vertical {
            ("up", "down")
        } else {
            ("left", "right")
        };
        let label = kivact::i18n::message("soft.keyboard_hint.to_navigate", "to navigate");
        let surface = row((
            kbd(keys.0),
            kbd(keys.1),
            text(label).style(style().margin_left(4)),
        ))
        .hit_testable(false)
        .style(|| {
            let t = tokens();
            let base = style()
                .gap(4)
                .padding_x(8)
                .padding_y(4)
                .align_items(Align::Center)
                .background(t.colors.bg_popover)
                .radius(t.radius.element)
                .color(t.colors.text_secondary)
                .font_size(12)
                .line_height(20.0 / 12.0)
                .text_wrap(false)
                .enter(style().opacity(0.0))
                .exit(style().opacity(0.0))
                .transition(
                    TransitionProp::Paint,
                    Transition::tween(150, Easing::CubicBezier(0.25, 0.1, 0.25, 1.0)),
                );
            t.shadows
                .low
                .iter()
                .cloned()
                .fold(base, |base, shadow| base.shadow(shadow))
        });
        overlay(anchor.get(), Placement::Below, surface)
            .offset(8.0)
            .align(OverlayAlign::Start)
    })
    .into_view();
    (anchor, hint)
}
