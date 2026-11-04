//! Role: Preserve a hover-open surface while the mouse travels towards it.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use kivact_core::draw::{Point, Rect};
use kivact_core::task::{Task, sleep, spawn};
use kivact_core::view::{IntoValue, NodeRef, mouse_position};
use kivact_core::{Value, effect, signal};

use crate::{Delayed, Delays, use_delayed};

/// Hold a submenu while the mouse moves from an anchor toward the surface.
///
/// Geometry is measured in window coordinates. The hold starts on departure
/// from the anchor, renews only with forward movement inside the triangle,
/// and expires after `timeout` without progress. It does not open anything.
#[track_caller]
pub fn use_hover_corridor(
    anchor: impl IntoValue<Option<Rect>>,
    surface: impl IntoValue<Option<Rect>>,
    enabled: impl IntoValue<bool>,
    timeout: Duration,
) -> Value<bool> {
    hover_corridor(
        anchor.into_value(),
        surface.into_value(),
        enabled.into_value(),
        timeout,
    )
    .0
}

/// Delay opening and closing a hover surface. A diagonal path from the
/// anchor toward the open surface restarts the existing close delay.
#[track_caller]
pub fn use_hover_surface(
    anchor: NodeRef,
    surface: NodeRef,
    enabled: impl IntoValue<bool>,
    delays: Delays,
) -> Delayed {
    let enabled = enabled.into_value();
    let over_anchor = anchor.hovered();
    let over_surface = surface.hovered();
    let delayed = use_delayed(
        move || enabled.get() && (over_anchor.get() || over_surface.get()),
        delays,
    );
    let shown = delayed.shown();
    let (_, progress) = hover_corridor(
        anchor.rect(),
        surface.rect(),
        (move || enabled.get() && shown.get()).into_value(),
        delays.close,
    );
    effect(move || {
        let progress = progress.get();
        if progress != 0
            && shown.get_untracked()
            && !over_anchor.get_untracked()
            && !over_surface.get_untracked()
        {
            delayed.pulse();
        }
    });
    delayed
}

fn hover_corridor(
    anchor: Value<Option<Rect>>,
    surface: Value<Option<Rect>>,
    enabled: Value<bool>,
    timeout: Duration,
) -> (Value<bool>, Value<u64>) {
    let mouse = mouse_position();
    let holding = signal(false);
    let progress = signal(0_u64);
    let path = Rc::new(Cell::new(None::<(Point, Point)>));
    let timer = Rc::new(Cell::new(None::<Task>));
    effect(move || {
        let available = enabled.get() && !timeout.is_zero();
        let geometry = anchor.get().zip(surface.get());
        let sample = mouse.get();
        let mut advancing = false;
        match (available, geometry, sample) {
            (true, Some((anchor, _)), Some(at)) if anchor.contains(at) => path.set(Some((at, at))),
            (true, Some((_, surface)), Some(at)) if !surface.contains(at) => {
                if let Some((origin, previous)) = path.get() {
                    if let Some(edge) = Edge::facing(origin, surface)
                        && edge.contains(at, origin)
                        && edge.distance(at) <= edge.distance(previous)
                    {
                        advancing = edge.distance(at) < edge.distance(previous);
                        path.set(Some((origin, at)));
                        if !advancing {
                            return;
                        }
                    } else {
                        path.set(None);
                    }
                }
            }
            _ => path.set(None),
        }
        if let Some(task) = timer.take() {
            task.cancel();
        }
        holding.set(advancing);
        if advancing {
            progress.update(|progress| *progress = progress.wrapping_add(1));
            let path = path.clone();
            let timer_slot = timer.clone();
            timer.set(Some(spawn(async move {
                sleep(timeout).await;
                path.set(None);
                timer_slot.set(None);
                holding.set(false);
            })));
        }
    });
    (holding.into(), progress.into())
}

struct Edge {
    first: Point,
    last: Point,
    vertical: bool,
}

impl Edge {
    fn facing(origin: Point, rect: Rect) -> Option<Self> {
        const PAD: f32 = 8.0;
        let right = rect.x + rect.width;
        let bottom = rect.y + rect.height;
        let vertical = |x| Self {
            first: Point::new(x, rect.y - PAD),
            last: Point::new(x, bottom + PAD),
            vertical: true,
        };
        let horizontal = |y| Self {
            first: Point::new(rect.x - PAD, y),
            last: Point::new(right + PAD, y),
            vertical: false,
        };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            None
        } else if origin.x < rect.x {
            Some(vertical(rect.x))
        } else if origin.x > right {
            Some(vertical(right))
        } else if origin.y < rect.y {
            Some(horizontal(rect.y))
        } else if origin.y > bottom {
            Some(horizontal(bottom))
        } else {
            None
        }
    }

    fn distance(&self, point: Point) -> f32 {
        if self.vertical {
            (self.first.x - point.x).abs()
        } else {
            (self.first.y - point.y).abs()
        }
    }

    fn contains(&self, point: Point, origin: Point) -> bool {
        fn cross(a: Point, b: Point, p: Point) -> f32 {
            (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
        }
        let sides = [
            cross(origin, self.first, point),
            cross(self.first, self.last, point),
            cross(self.last, origin, point),
        ];
        sides.iter().all(|&side| side >= 0.0) || sides.iter().all(|&side| side <= 0.0)
    }
}
