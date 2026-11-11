//! Role: Attach velocity-aware dismissal to a drawer's dedicated drag handle.
use kivact_core::geometry::{Edge, Point, Rect};
use kivact_core::state::{Binding, Machine as MachineTrait, Step, Value, machine, memo};
use kivact_core::style::{style, use_theme};
use kivact_core::view::{Node, NodeRef, node};
use kivact_interaction::{DragCoordinates, DragPhase, DragUpdate, InteractionPolicy, use_drag};

/// How far a drawer's handle has pulled the panel away from its edge.
#[derive(Clone, Copy, PartialEq)]
enum Swipe {
    /// The handle is not pulling the panel.
    Resting,
    /// The handle pulls the panel by this many logical pixels.
    Pulling(f64),
}

/// The four drag events and the release's decision inputs.
enum Event {
    Start,
    Update(f64),
    End { distance: f64, velocity: Point },
    Cancel,
}

enum Effect {
    Close,
}

/// The measured panel and the theme's dismissal policy for one handle.
struct Machine {
    edge: Edge,
    rect: Option<Rect>,
    close_fraction: f64,
    flick_distance: f64,
    flick_speed: f64,
}

impl MachineTrait for Machine {
    type State = Swipe;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, _: &Swipe, event: Event) -> Step<Swipe, Event, Effect> {
        match event {
            Event::Start => Step::to(Swipe::Pulling(0.0)),
            Event::Update(distance) => Step::to(Swipe::Pulling(distance.max(0.0))),
            Event::End { distance, velocity } => {
                let distance = distance.max(0.0);
                let far = self.rect.is_some_and(|rect| {
                    let size = if matches!(self.edge, Edge::Left | Edge::Right) {
                        rect.width
                    } else {
                        rect.height
                    };
                    size > 0.0 && distance > size * self.close_fraction
                });
                let flick = distance > self.flick_distance
                    && toward(self.edge, velocity) > self.flick_speed;
                let step = Step::to(Swipe::Resting);
                if far || flick {
                    step.with(Effect::Close)
                } else {
                    step
                }
            }
            Event::Cancel => Step::to(Swipe::Resting),
        }
    }
}

pub(crate) fn handle(
    edge: Edge,
    panel: NodeRef,
    open: Binding<bool>,
    variants: &crate::variants::Variants,
) -> (Node, Value<f64>) {
    let policy = use_theme().get().ext::<InteractionPolicy>().drawer.swipe;
    let rect = panel.rect();
    let swipe = machine(
        Swipe::Resting,
        move || Machine {
            edge,
            rect: rect.get(),
            close_fraction: policy.close_fraction,
            flick_distance: policy.flick_distance,
            flick_speed: policy.flick_speed,
        },
        move |_, effect| match effect {
            Effect::Close => open.set(false),
        },
    );
    let drag = use_drag::<()>(DragCoordinates::Window)
        .track_velocity()
        .on_start(move |_| swipe.send(Event::Start))
        .on_update(move |update: DragUpdate<()>| {
            swipe.send(Event::Update(toward(edge, update.delta)))
        })
        .on_end(move |update: DragUpdate<()>| {
            swipe.send(if update.phase == DragPhase::Cancel {
                Event::Cancel
            } else {
                Event::End {
                    distance: toward(edge, update.delta),
                    velocity: update.velocity,
                }
            })
        });
    let part = drag.source(());
    let state = part.state();
    let pulled = memo(move || match swipe.get() {
        Swipe::Resting => 0.0,
        Swipe::Pulling(distance) => distance,
    });
    let handle =
        crate::layer::with_recipe_style(
            node().part(part).role(kivact_core::view::Role::Group).name(
                kivact_core::i18n::message("drawer.drag_handle", "Drag to close"),
            ),
            |recipes| recipes.dialog_recipes().drag_handle_recipe(),
            variants,
            move || crate::DialogState {
                input: state.get().input,
                open: open.get(),
                kind: kivact_interaction::DialogMode::Drawer(edge),
                media: false,
            },
        );
    (handle, pulled)
}
pub(crate) fn translation(edge: Edge, distance: f64) -> kivact_core::style::Style {
    let (x, y) = match edge {
        Edge::Left => (-distance, 0.0),
        Edge::Right => (distance, 0.0),
        Edge::Top => (0.0, -distance),
        Edge::Bottom => (0.0, distance),
    };
    style().translate(x, y)
}
fn toward(edge: Edge, point: Point) -> f64 {
    match edge {
        Edge::Left => -point.x,
        Edge::Right => point.x,
        Edge::Top => -point.y,
        Edge::Bottom => point.y,
    }
}
