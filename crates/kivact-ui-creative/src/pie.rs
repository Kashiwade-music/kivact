//! Role: Pie menu of the Creative kit with pointer and keyboard selection.
use kivact::state::IntoBinding;
use kivact::ui::button;
use kivact::view::HandlerResult;

use kivact::geometry::{Point, Rect};
use kivact::i18n::message;
use kivact::interaction::use_popup;
use kivact::state::IntoValue;
use kivact::style::{ColorToken, style};
use kivact::view::{IntoView, Key, Node, NodeBuilder, Role, column, node, text};

use kivact::state::binding;
use kivact::state::callback;
use kivact::style::{Icon, Position};
use kivact::view::Handled;
use kivact::view::{ButtonPress, MouseButton, Placement, PointerPhase, overlay_at};

/// A directional command, in clockwise degrees from the positive horizontal axis.
#[derive(Clone, Debug, PartialEq)]
pub struct PieEntry {
    pub label: String,
    pub angle: f64,
    /// The icon of the command: a name in the theme's icon set or an image.
    pub icon: Icon,
}
impl PieEntry {
    pub fn new(label: impl Into<String>, angle: f64, icon: impl Into<Icon>) -> Self {
        Self {
            label: label.into(),
            angle,
            icon: icon.into(),
        }
    }
}
/// Public state permits a surrounding application to open the same menu from a hovered editor.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PieState {
    #[default]
    Closed,
    Open {
        mode: PieMode,
        center: Point,
        hot: Option<usize>,
    },
}
/// How the directional menu was opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PieMode {
    Click,
    Held,
}
impl PieState {
    pub fn open_at(center: Point, held: bool) -> Self {
        Self::Open {
            mode: if held { PieMode::Held } else { PieMode::Click },
            center,
            hot: None,
        }
    }
    fn hot(self) -> Option<usize> {
        match self {
            Self::Closed => None,
            Self::Open { hot, .. } => hot,
        }
    }
    fn center(self) -> Point {
        match self {
            Self::Closed => Point::default(),
            Self::Open { center, .. } => center,
        }
    }
    /// Keep the center dead zone and select the closest offered direction.
    pub fn pointer(&mut self, pointer: Point, entries: &[PieEntry]) {
        let Self::Open { center, hot, .. } = self else {
            return;
        };
        let delta = Point::new(pointer.x - center.x, pointer.y - center.y);
        *hot = if delta.x.hypot(delta.y) < 22.0 {
            None
        } else {
            let angle = delta.y.atan2(delta.x).to_degrees();
            entries
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let distance = |entry: &PieEntry| {
                        ((angle - entry.angle + 180.0).rem_euclid(360.0) - 180.0).abs()
                    };
                    distance(a).total_cmp(&distance(b))
                })
                .map(|(index, _)| index)
        };
    }
}

/// A directional popup with pointer selection and keyboard numbers/arrows/Enter/Escape.
/// To implement a held shortcut, open `PieState::open_at(point, true)` on key-down and
/// commit its `hot` choice on key-up using the same application command callback.
#[track_caller]
pub fn pie_menu(
    entries: impl IntoValue<Vec<PieEntry>>,
    state: impl IntoBinding<PieState>,
) -> PieMenu {
    PieMenu {
        root: column(()).style(style().height(0).gap(0)),
        entries: entries.into_value(),
        state: state.into_binding(),
        choose: None,
    }
}

/// A directional popup with an application activation handler.
pub struct PieMenu {
    root: Node,
    entries: kivact::state::Value<Vec<PieEntry>>,
    state: kivact::state::Binding<PieState>,
    choose: Option<kivact::state::Callback<usize>>,
}
impl PieMenu {
    /// Run the selected directional command.
    pub fn on_activate<R: HandlerResult>(mut self, choose: impl Fn(usize) -> R + 'static) -> Self {
        self.choose = Some(callback(choose));
        self
    }
}
impl NodeBuilder for PieMenu {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for PieMenu {
    fn into_view(self) -> kivact::view::View {
        let choose = self.choose;
        pie_menu_view(self.root, self.entries, self.state, move |index| {
            if let Some(choose) = choose {
                choose.call(index);
            }
        })
        .into_view()
    }
}

fn pie_menu_view<R: HandlerResult>(
    root: Node,
    entries: impl IntoValue<Vec<PieEntry>>,
    state: impl IntoBinding<PieState>,
    choose: impl Fn(usize) -> R + 'static,
) -> Node {
    let choose = callback(choose);
    let entries = entries.into_value();
    let state = state.into_binding();
    let external = state.into_value();
    let controller = kivact::state::machine(
        external.get_untracked(),
        move || machine::Machine {
            entries: entries.get_untracked(),
        },
        move |_, effect| match effect {
            machine::Effect::Publish(next) => state.set(next),
            machine::Effect::Choose(index) => choose.call(index),
        },
    );
    controller.feed(external, machine::Event::External);
    let current = controller.state();
    let visible = binding(
        (move || matches!(current.get(), PieState::Open { .. })).into_value(),
        move |open: bool| {
            if !open {
                controller.send(machine::Event::Close);
            }
        },
    );
    let popup = use_popup(visible);
    let choose = move |index| {
        controller.send(machine::Event::Choose(index));
    };
    let layer = kivact::view::show(popup.is_open(), move || {
        let key_entries = entries;
        let buttons = kivact::view::for_index(entries, move |index, entry| {
            let angle = move || entry.with(|entry| entry.angle.to_radians());
            let button = button(move || entry.get().label)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .icon(kivact::view::icon(move || entry.get().icon))
                .with_node(move |node| node.selected(move || current.get().hot() == Some(index)));
            button
                .with_node(move |node| {
                    node.style(move || {
                        style()
                            .position(Position::Absolute)
                            .left(160.0 + angle().cos() * 104.0 - 48.0)
                            .top(160.0 + angle().sin() * 104.0 - 18.0)
                            .size(96, 36)
                    })
                })
                .with_node(move |node| {
                    node.on_pointer_default(move |event| {
                        if matches!(event.phase(), PointerPhase::Move | PointerPhase::Enter) {
                            controller.send(machine::Event::Pointer(event.window_pos()));
                        }
                        if matches!(event.phase(), PointerPhase::Down(_)) {
                            Handled::Yes
                        } else {
                            Handled::No
                        }
                    })
                })
                .on_press(move || choose(index))
                .into_view()
        });
        overlay_at(
            move || current.get().center(),
            Placement::Center,
            node()
                .child(buttons)
                .child(
                    text(message("creative.pie.label", "Shading")).style(
                        style()
                            .position(Position::Absolute)
                            .left(130)
                            .top(152)
                            .width(60)
                            .text_align(kivact::style::TextAlign::Center),
                    ),
                )
                .style(style().size(320, 320).position(Position::Relative))
                .draw(|draw, _| {
                    draw.rounded_rect(
                        Rect::new(92.0, 92.0, 136.0, 136.0),
                        68.0,
                        ColorToken::Raised,
                    )
                })
                .on_pointer_default(move |event| {
                    if matches!(event.phase(), PointerPhase::Move | PointerPhase::Enter) {
                        controller.send(machine::Event::Pointer(event.window_pos()));
                    } else if matches!(
                        event.phase(),
                        PointerPhase::Down(ButtonPress {
                            button: MouseButton::Primary,
                            ..
                        })
                    ) {
                        if let Some(index) = current.get().hot() {
                            choose(index);
                        }
                        return Handled::Yes;
                    }
                    Handled::No
                }),
        )
        .with_node(move |node| node.part(popup.content()))
        .with_node(move |node| node.role(Role::Menu))
        .with_node(move |node| node.name(message("creative.pie.name", "Shading pie")))
        .with_node(move |node| node.focusable(true))
        .with_node(move |node| {
            node.on_key_default(move |event| {
                let next = current.get();
                match event.key() {
                    Key::Char(key) if ('1'..='9').contains(&key) => {
                        let index = (key as u8 - b'1') as usize;
                        if index < key_entries.with(Vec::len) {
                            choose(index);
                        }
                    }
                    Key::ArrowRight | Key::ArrowDown | Key::ArrowLeft | Key::ArrowUp => {
                        if key_entries.with(Vec::is_empty) {
                            return Handled::No;
                        }
                        controller.send(machine::Event::Navigate(matches!(
                            event.key(),
                            Key::ArrowRight | Key::ArrowDown
                        )));
                    }
                    Key::Enter => {
                        if let Some(index) = next
                            .hot()
                            .filter(|index| *index < key_entries.with(Vec::len))
                        {
                            choose(index);
                        }
                    }
                    _ => return Handled::No,
                }
                Handled::Yes
            })
        })
    });
    root.child(layer)
}

mod machine {
    //! Role: Transition the public pie state and command activation atomically.
    use super::{PieEntry, PieState};
    use kivact::geometry::Point;
    use kivact::state::Step;
    pub(super) struct Machine {
        pub(super) entries: Vec<PieEntry>,
    }
    pub(super) enum Event {
        External(PieState),
        Close,
        Choose(usize),
        Pointer(Point),
        Navigate(bool),
    }
    pub(super) enum Effect {
        Publish(PieState),
        Choose(usize),
    }
    impl kivact::state::Machine for Machine {
        type State = PieState;
        type Event = Event;
        type Effect = Effect;
        fn transition(&self, state: &PieState, event: Event) -> Step<PieState, Event, Effect> {
            let next = match event {
                Event::External(next) => return Step::to(next),
                Event::Close => PieState::Closed,
                Event::Choose(index) => {
                    return Step::to(PieState::Closed)
                        .with(Effect::Choose(index))
                        .with(Effect::Publish(PieState::Closed));
                }
                Event::Pointer(point) => {
                    let mut next = *state;
                    next.pointer(point, &self.entries);
                    next
                }
                Event::Navigate(forward) => {
                    let mut next = *state;
                    if let PieState::Open { hot, .. } = &mut next {
                        let len = self.entries.len();
                        if len != 0 {
                            let index = hot.unwrap_or(0).min(len - 1);
                            *hot = Some(if forward {
                                (index + 1) % len
                            } else {
                                (index + len - 1) % len
                            });
                        }
                    }
                    next
                }
            };
            Step::to(next).with(Effect::Publish(next))
        }
    }
}
