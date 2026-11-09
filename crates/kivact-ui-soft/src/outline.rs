//! Role: Track visible document headings and navigate their outline.
use crate::side_navigation::items::logical_row;
use crate::stack::rtl;

use kivact::geometry::Point;
use kivact::motion::Transition;
use kivact::motion::animated;
use kivact::state::IntoBinding;
use kivact::view::Element;
use kivact::view::Node;
use kivact::view::column;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::state::Binding;
use kivact::state::Callback;
use kivact::state::IntoValue;
use kivact::state::Machine;
use kivact::state::MachineHandle;
use kivact::state::Step;
use kivact::state::Value;
use kivact::state::callback;
use kivact::state::effect_on;
use kivact::state::machine;
use kivact::state::memo;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::Position;
use kivact::style::TransitionProp;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::Handled;
use kivact::view::HandlerResult;
use kivact::view::IntoView;
use kivact::view::Key;
use kivact::view::KeyEvent;
use kivact::view::NodeRef;
use kivact::view::Role;
use kivact::view::ScrollBehavior;
use kivact::view::View;
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OutlineDensity {
    /// Standard outline spacing, the default.
    #[default]
    Default,
    /// Tighter outline spacing.
    Compact,
}

/// Heading navigation with optional scroll tracking and controlled active state.
pub struct Outline {
    root: Node,
    items: Value<Vec<OutlineItem>>,
    active: Option<Binding<Option<String>>>,
    density: Value<OutlineDensity>,
    offset: Value<f64>,
    viewport: Option<NodeRef>,
    content: Option<NodeRef>,
    scroll_on_press: Value<bool>,
    change: Option<Callback<String>>,
    start: Option<Callback<String>>,
    end: Option<Callback<String>>,
}

/// Create a table of contents with standard spacing, zero offset and scrolling enabled.
pub fn outline(items: impl IntoValue<Vec<OutlineItem>>) -> Outline {
    Outline {
        root: node()
            .with_node(move |node| node.role(Role::Navigation))
            .name(kivact::i18n::message(
                "soft.outline.label",
                "Table of contents",
            )),
        items: items.into_value(),
        active: None,
        density: (OutlineDensity::Default).into_value(),
        offset: (0.0).into_value(),
        viewport: None,
        content: None,
        scroll_on_press: (true).into_value(),
        change: None,
        start: None,
        end: None,
    }
}

impl Outline {
    /// Connect the caller's optional active heading identifier.
    pub fn active(mut self, active: impl IntoBinding<Option<String>>) -> Self {
        self.active = Some(active.into_binding());
        self
    }
    /// Choose the link spacing; standard by default.
    pub fn density(mut self, density: impl IntoValue<OutlineDensity>) -> Self {
        self.density = density.into_value();
        self
    }
    /// Set the heading activation and scroll inset in logical pixels; defaults to zero.
    pub fn offset(mut self, offset: impl IntoValue<f64>) -> Self {
        self.offset = offset.into_value();
        self
    }
    /// Observe and scroll within this viewport instead of the window.
    pub fn scroll_container(mut self, viewport: NodeRef) -> Self {
        self.viewport = Some(viewport);
        self
    }
    /// Observe this content node so layout changes update heading tracking.
    pub fn scroll_content(mut self, content: NodeRef) -> Self {
        self.content = Some(content);
        self
    }
    /// Enable scrolling on link activation when mounted; true by default.
    pub fn scroll_on_press(mut self, scroll: impl IntoValue<bool>) -> Self {
        self.scroll_on_press = scroll.into_value();
        self
    }
    /// Report the heading identifier when scroll tracking changes the active heading.
    pub fn on_active_change<R: HandlerResult>(
        mut self,
        change: impl Fn(String) -> R + 'static,
    ) -> Self {
        self.change = Some(callback(change));
        self
    }
    /// Report the destination identifier before navigation begins.
    pub fn on_navigate_start<R: HandlerResult>(
        mut self,
        start: impl Fn(String) -> R + 'static,
    ) -> Self {
        self.start = Some(callback(start));
        self
    }
    /// Report the destination identifier when scrolling finishes, or immediately without animated scrolling.
    pub fn on_navigate_end<R: HandlerResult>(
        mut self,
        end: impl Fn(String) -> R + 'static,
    ) -> Self {
        self.end = Some(callback(end));
        self
    }
}

impl Element for Outline {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
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
    // xtask-allow(state-fields): option: mounted scroll-viewport reference, not scroll phase.
    viewport: Option<NodeRef>,
    offset: f64,
    // xtask-allow(state-fields): independent: immutable setup snapshot controlling whether navigation requests scrolling.
    scroll_on_press: bool,
    // xtask-allow(state-fields): option: caller-owned active-heading binding; pending lifecycle belongs to ScrollMachine.
    controlled: Option<Binding<Option<String>>>,
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

impl IntoView for Outline {
    fn into_view(self) -> View {
        let Self {
            root,
            items,
            active,
            density,
            offset,
            viewport,
            content,
            scroll_on_press,
            change,
            start,
            end,
        } = self;
        kivact::view::scope(move || {
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
            if let Some(change) = change {
                effect_on(active, move |id| {
                    if let Some(id) = id {
                        change.call(id.clone());
                    }
                });
            }
            root.with_node(move |node| {
                node.base_style(style().direction(Direction::Column).width(pct(100)))
            })
            .child(dynamic(items, move |items| {
                outline_list(items, active, density.get(), navigation)
            }))
        })
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
            .style(|| {
                style()
                    .gap(kivact::style::use_tokens::<crate::Tokens>().spacing.s0_5)
                    .grow(1)
                    .min_width(0)
            }),
    ))
    .style(|| {
        style()
            .direction(logical_row())
            .gap(kivact::style::use_tokens::<crate::Tokens>().spacing.s0_5)
            .align_items(Align::Stretch)
    })
}

fn outline_row(level: u8, density: OutlineDensity, active: bool) -> kivact::style::Style {
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let role = t.typography.body;
    let indent = match level {
        3 => t.spacing.s7,
        4 => t.spacing.s11,
        5.. => t.spacing.s12,
        _ => t.spacing.s3,
    };
    let (left, right) = if rtl() {
        (t.spacing.s2, indent)
    } else {
        (indent, t.spacing.s2)
    };
    style()
        .direction(logical_row())
        .align_items(Align::Center)
        .width(pct(100))
        .radius(t.radius.element)
        .padding_left(left)
        .padding_right(right)
        .padding_y(if density == OutlineDensity::Compact {
            t.spacing.s1
        } else {
            t.spacing.s2
        })
        .font_size(role.size_px)
        .line_height(role.line_height_px / role.size_px)
        .weight(if active {
            t.typography.weight_semibold
        } else {
            role.weight
        })
        .color(if active {
            t.colors.text_primary
        } else {
            t.colors.text_secondary
        })
        .hover(
            style()
                .background(t.colors.overlay_hover)
                .color(t.colors.text_primary),
        )
        .pressed(style().background(t.colors.overlay_pressed))
        .focus_visible(crate::recipe_style::ring(&t, t.colors.accent))
        .transition(
            TransitionProp::All,
            Transition::tween(t.motion.fast as f64, t.motion.ease_standard.clone()),
        )
}

kivact::state::constant_value!(OutlineDensity);

fn active_heading(
    active: Option<Binding<Option<String>>>,
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
    let t = kivact::style::use_tokens::<crate::Tokens>();
    let transition = Transition::tween(t.motion.fast_min as f64, t.motion.ease_standard.clone());
    let height = animated(
        move || indicator_rect.get().map_or(0.0, |(_, height)| height),
        transition.clone(),
    );
    node()
        .style(|| {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            style()
                .position(Position::Relative)
                .width(t.spacing.s0_5)
                .shrink(0)
                .align_self(Align::Stretch)
                .background(t.colors.border)
                .radius(t.radius.full)
        })
        .child(node().style(move || {
            let t = kivact::style::use_tokens::<crate::Tokens>();
            let offset = indicator_rect.get().map_or(0.0, |(y, _)| y);
            style()
                .position(Position::Absolute)
                .top(0)
                .left(0)
                .width(pct(100))
                .height(height.get())
                .translate_y(offset)
                .radius(t.radius.full)
                .background(t.colors.icon_primary)
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
