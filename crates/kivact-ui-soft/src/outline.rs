//! Role: Track visible document headings and navigate their outline.
use crate::color as c;
use crate::motion::{EASE_STANDARD, FAST, FAST_MIN};
use crate::radius::RADIUS_ELEMENT;
use crate::radius::RADIUS_FULL;
use crate::recipe_parts::ring;
use crate::space::{SPACE_S0_5, SPACE_S1, SPACE_S2, SPACE_S3, SPACE_S7, SPACE_S11, SPACE_S12};
use crate::stack::rtl;
use crate::type_scale::{self as t, WEIGHT_SEMIBOLD};

use kivact::component;
use kivact::geometry::Point;
use kivact::motion::Transition;
use kivact::motion::animated;
use kivact::state::Signal;
use kivact::view::Element;
use kivact::view::Node;
use kivact::view::column;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::Prop;
use kivact::state::Callback;
use kivact::state::Machine;
use kivact::state::MachineHandle;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::effect_on;
use kivact::state::machine;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::Position;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::Handled;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::NodeRef;
use kivact::view::Role;
use kivact::view::ScrollBehavior;
use kivact::view::dynamic;
use kivact::view::node_ref;
use std::rc::Rc;
use std::time::Duration;

/// One heading destination in a table of contents.
#[derive(Clone, PartialEq)]
pub struct OutlineItem {
    /// Stable identifier used by the active binding and callbacks.
    pub id: String,
    /// Visible link label.
    pub label: String,
    /// Heading depth used for indentation.
    pub level: u8,
    /// Mounted heading to observe and scroll into view.
    pub target: NodeRef,
}

impl OutlineItem {
    /// Describe a heading destination without creating or owning its node.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        level: u8,
        target: NodeRef,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            level,
            target,
        }
    }
}

/// Vertical spacing of outline links.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum OutlineDensity {
    /// Standard outline spacing, the default.
    #[default]
    Default,
    /// Tighter outline spacing.
    Compact,
}

/// Heading navigation with optional scroll tracking and controlled active state.
///
/// `scroll_container` observes and scrolls within a viewport instead of the window;
/// `scroll_content` observes a content node so layout changes update heading tracking.
#[component]
pub fn outline(
    items: Value<Vec<OutlineItem>>,
    active: Option<Signal<Option<String>>>,
    #[prop(default = OutlineDensity::Default)] density: Value<OutlineDensity>,
    #[prop(default = 0.0)] offset: Value<f64>,
    scroll_container: Option<NodeRef>,
    scroll_content: Option<NodeRef>,
    #[prop(default = true)] scroll_on_press: Value<bool>,
    on_active_change: Option<Callback<String>>,
    on_navigate_start: Option<Callback<String>>,
    on_navigate_end: Option<Callback<String>>,
) -> impl Element {
    let root = node()
        .with_node(move |node| node.role(Role::Navigation))
        .name(kivact::i18n::message(
            "soft.outline.label",
            "Table of contents",
        ));
    let viewport = scroll_container;
    let content = scroll_content;
    let start = on_navigate_start;
    let end = on_navigate_end;
    let pending = machine(
        OutlineScroll::Idle,
        || ScrollMachine,
        move |_, id| {
            if let Some(end) = end {
                end.call(id);
            }
        },
    );
    let navigation = Navigation {
        viewport,
        offset: offset.get(),
        scroll_on_press: scroll_on_press.get(),
        controlled: active,
        pending,
        start,
        end,
    };
    kivact::state::on_cleanup(move || navigation.finish());
    let active = active_heading(active, pending, items, viewport, content, offset);
    if let Some(change) = on_active_change {
        effect_on(active, move |id| {
            if let Some(id) = id {
                change.call(id.clone());
            }
        });
    }
    root.with_node(move |node| {
        node.base_style(
            style()
                .flex_direction(FlexDirection::Column)
                .width(pct(100)),
        )
    })
    .child(dynamic(items, move |items| {
        outline_list(items, active, density.get(), navigation)
    }))
}

#[derive(Clone, PartialEq)]
enum OutlineScroll {
    Idle,
    Scrolling { target: String },
}
enum ScrollEvent {
    Begin(String),
    Finish,
}
struct ScrollMachine;
impl Machine for ScrollMachine {
    type State = OutlineScroll;
    type Event = ScrollEvent;
    type Effect = String;
    fn transition(
        &self,
        state: &OutlineScroll,
        event: ScrollEvent,
    ) -> Step<OutlineScroll, ScrollEvent, String> {
        match event {
            ScrollEvent::Begin(target) => Step::to(OutlineScroll::Scrolling { target })
                .after(Duration::from_millis(250), ScrollEvent::Finish),
            ScrollEvent::Finish => {
                let step = Step::to(OutlineScroll::Idle).cancel(ScrollEvent::Finish);
                match state {
                    OutlineScroll::Idle => step,
                    OutlineScroll::Scrolling { target } => step.with(target.clone()),
                }
            }
        }
    }
}
#[derive(Clone, PartialEq)]
enum OutlineFocus {
    Unset,
    At(String),
}
enum FocusEvent {
    /// A navigation key moved focus to this heading link.
    Move(String),
    /// Keyboard focus is now inside (`true`) or outside (`false`) the list.
    Within(bool),
}
struct FocusMachine;
impl Machine for FocusMachine {
    type State = OutlineFocus;
    type Event = FocusEvent;
    type Effect = ();
    fn transition(
        &self,
        _: &OutlineFocus,
        event: FocusEvent,
    ) -> Step<OutlineFocus, FocusEvent, ()> {
        match event {
            FocusEvent::Move(id) => Step::to(OutlineFocus::At(id)),
            FocusEvent::Within(true) => Step::stay(),
            FocusEvent::Within(false) => Step::to(OutlineFocus::Unset),
        }
    }
}

#[derive(Clone, Copy)]
struct Navigation {
    viewport: Option<NodeRef>,
    offset: f64,
    scroll_on_press: bool,
    controlled: Option<Signal<Option<String>>>,
    pending: MachineHandle<ScrollMachine>,
    start: Option<Callback<String>>,
    end: Option<Callback<String>>,
}

impl Navigation {
    fn finish(self) {
        self.pending.send(ScrollEvent::Finish);
    }

    fn navigate(self, item: &OutlineItem) {
        self.finish();
        if let Some(start) = self.start {
            start.call(item.id.clone());
        }
        if let Some(active) = self.controlled {
            active.set(Some(item.id.clone()));
        }
        let target = item.target;
        let is_motion_reduced = kivact::platform::is_motion_reduced().get();
        if self.scroll_on_press {
            if let Some(viewport) = self.viewport {
                if let (Some(viewport_rect), Some(target_rect)) =
                    (viewport.rect().get(), target.rect().get())
                {
                    let point = Point {
                        x: 0.0,
                        y: viewport.scroll_offset().get().y + target_rect.y
                            - viewport_rect.y
                            - self.offset,
                    };
                    viewport.scroll_to(
                        point,
                        if is_motion_reduced {
                            ScrollBehavior::Instant
                        } else {
                            ScrollBehavior::Smooth(kivact::motion::Easing::EaseOut)
                        },
                    );
                }
            } else {
                target.scroll_into_view();
            }
        }
        if self.scroll_on_press && self.viewport.is_some() && !is_motion_reduced {
            self.pending.send(ScrollEvent::Begin(item.id.clone()));
        } else if let Some(end) = self.end {
            end.call(item.id.clone());
        }
    }
}

fn outline_list(
    items: Vec<OutlineItem>,
    active: Value<Option<String>>,
    density: OutlineDensity,
    navigation: Navigation,
) -> Node {
    let list = node_ref();
    let focused = list.focus_within();
    let roving = machine(OutlineFocus::Unset, || FocusMachine, |_, ()| {});
    roving.feed(focused, FocusEvent::Within);
    let items = Rc::new(items);
    let refs = Rc::new(items.iter().map(|_| node_ref()).collect::<Vec<_>>());
    let active_items = items.clone();
    let active_refs = refs.clone();
    let indicator_rect = memo(move || {
        let active = active.get()?;
        let index = active_items.iter().position(|item| item.id == active)?;
        let item = active_refs[index].rect().get()?;
        let list = list.rect().get()?;
        Some((item.y - list.y, item.height))
    });
    let track = outline_track(indicator_rect);
    let rows = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let id = item.id.clone();
            let selected_id = id.clone();
            let roving_id = id.clone();
            let item = item.clone();
            let items = items.clone();
            let refs = refs.clone();
            let reference = refs[index];
            let level = item.level;
            let selected = memo(move || active.get().as_deref() == Some(selected_id.as_str()));
            let focusable = move || {
                if let OutlineFocus::At(roving) = roving.get() {
                    return roving == roving_id;
                }
                selected.get() || active.get().is_none() && index == 0
            };
            let label = item.label.clone();
            let link = node()
                .with_node(move |node| node.role(Role::Link))
                .name(label.clone())
                .node_ref(reference)
                .focusable(focusable)
                .with_node(move |node| node.selected(selected))
                .on_press(move || navigation.navigate(&item))
                .on_key(move |event| outline_key(event, index, &items, &refs, roving))
                .style(move || outline_row(level, density, selected.get()))
                .child(text(label).style(style().single_line()));
            node()
                .with_node(move |node| node.role(Role::ListItem))
                .child(link)
        })
        .collect::<Vec<_>>();
    row((
        track,
        column(rows)
            .node_ref(list)
            .with_node(move |node| node.role(Role::List))
            .style(|| style().gap(SPACE_S0_5).flex_grow(1).min_width(0)),
    ))
    .style(|| {
        style()
            .flex_direction(logical_row())
            .gap(SPACE_S0_5)
            .align_items(Align::Stretch)
    })
}

/// The row direction that follows the reading direction.
fn logical_row() -> FlexDirection {
    if rtl() {
        FlexDirection::RowReverse
    } else {
        FlexDirection::Row
    }
}

fn outline_row(level: u8, density: OutlineDensity, active: bool) -> kivact::style::Style {
    let role = t::BODY;
    let indent = match level {
        3 => SPACE_S7,
        4 => SPACE_S11,
        5.. => SPACE_S12,
        _ => SPACE_S3,
    };
    let (left, right) = if rtl() {
        (SPACE_S2, indent)
    } else {
        (indent, SPACE_S2)
    };
    style()
        .flex_direction(logical_row())
        .align_items(Align::Center)
        .width(pct(100))
        .border_radius(RADIUS_ELEMENT)
        .padding_left(left)
        .padding_right(right)
        .padding_y(if density == OutlineDensity::Compact {
            SPACE_S1
        } else {
            SPACE_S2
        })
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .font_weight(if active { WEIGHT_SEMIBOLD } else { role.weight })
        .color(if active {
            c::TEXT_PRIMARY
        } else {
            c::TEXT_SECONDARY
        })
        .hover(style().background(c::OVERLAY_HOVER).color(c::TEXT_PRIMARY))
        .active(style().background(c::OVERLAY_PRESSED))
        .focus_visible(ring(c::ACCENT))
        .transition(
            TransitionProp::All,
            Transition::tween(FAST as f64, EASE_STANDARD),
        )
}

fn active_heading(
    active: Option<Signal<Option<String>>>,
    pending: MachineHandle<ScrollMachine>,
    items: Value<Vec<OutlineItem>>,
    viewport: Option<NodeRef>,
    content: Option<NodeRef>,
    offset: Value<f64>,
) -> Value<Option<String>> {
    memo(move || {
        if let Some(active) = active {
            return active.get();
        }
        if let OutlineScroll::Scrolling { target } = pending.get() {
            return Some(target);
        }
        let items = items.get();
        let first = items.first()?;
        let Some(viewport) = viewport else {
            return Some(first.id.clone());
        };
        let viewport_rect = viewport.rect().get()?;
        let scroll = viewport.scroll_offset().get().y;
        if let Some(content) = content.and_then(|content| content.rect().get())
            && scroll > 0.0
            && content.height - viewport_rect.height - scroll <= 2.0
        {
            return items.last().map(|item| item.id.clone());
        }
        let line = viewport_rect.y + offset.get() + 1.0;
        let item = items
            .iter()
            .rev()
            .find(|item| item.target.rect().get().is_some_and(|rect| rect.y <= line));
        Some(item.unwrap_or(first).id.clone())
    })
}

fn outline_track(indicator_rect: Value<Option<(f64, f64)>>) -> Node {
    let transition = Transition::tween(FAST_MIN as f64, EASE_STANDARD);
    let height = animated(
        move || indicator_rect.get().map_or(0.0, |(_, height)| height),
        transition.clone(),
    );
    node()
        .style(|| {
            style()
                .position(Position::Relative)
                .width(SPACE_S0_5)
                .flex_shrink(0)
                .align_self(Align::Stretch)
                .background(c::BORDER)
                .border_radius(RADIUS_FULL)
        })
        .child(node().style(move || {
            let offset = indicator_rect.get().map_or(0.0, |(y, _)| y);
            style()
                .position(Position::Absolute)
                .top(0)
                .left(0)
                .width(pct(100))
                .height(height.get())
                .translate_y(offset)
                .border_radius(RADIUS_FULL)
                .background(c::ICON_PRIMARY)
                .transition(TransitionProp::All, transition.clone())
        }))
}

fn outline_key(
    event: KeyEvent,
    index: usize,
    items: &[OutlineItem],
    refs: &[NodeRef],
    roving: MachineHandle<FocusMachine>,
) -> Handled {
    let next = match event.key() {
        Key::ArrowUp => index.saturating_sub(1),
        Key::ArrowDown => (index + 1).min(items.len() - 1),
        Key::Home => 0,
        Key::End => items.len() - 1,
        _ => return Handled::No,
    };
    roving.send(FocusEvent::Move(items[next].id.clone()));
    refs[next].focus();
    Handled::Yes
}
