//! Role: Turn pointer travel into value or position scrubs.

use kivact::geometry::{Point, Rect};
use kivact::interaction::{DragApi, DragPhase, DragUpdate, ScalarMapping, ScalarRange, ScalarStep};
use std::rc::Rc;

use super::builder::NumericKind;
use super::machine::{self, Anchor, ScrubSpace};
use super::view::NumericView;

/// The space and units per pixel of a scrub.
///
/// A valid `drag_distance` moves the whole range over that many pixels; a
/// number field otherwise moves one step per pixel; every other kind moves the
/// range over its own extent along the drag, at least 60 pixels.
pub(super) fn scrub_units(
    kind: NumericKind,
    drag_distance: Option<f64>,
    step: ScalarStep,
    extent: f64,
) -> (ScrubSpace, f64) {
    if let Some(distance) = drag_distance.filter(|distance| distance.is_finite() && *distance > 0.0)
    {
        return (ScrubSpace::Position, 1.0 / distance);
    }
    match (kind, step) {
        (NumericKind::Number, ScalarStep::Value(amount)) => (ScrubSpace::Value, amount),
        (NumericKind::Number, ScalarStep::Position(amount)) => (ScrubSpace::Position, amount),
        _ => (ScrubSpace::Position, 1.0 / extent.max(60.0)),
    }
}

/// The pointer travel of a scrub along its drag axis: upward for a knob or a
/// vertical fader, rightward otherwise.
pub(super) fn scrub_along(kind: NumericKind, horizontal: bool, delta: Point) -> f64 {
    if matches!(kind, NumericKind::Knob | NumericKind::Fader) && !horizontal {
        -delta.y
    } else {
        delta.x
    }
}

/// The length a kind moves its range over when no `drag_distance` is set.
pub(super) fn scrub_extent(kind: NumericKind, horizontal: bool, rect: Option<Rect>) -> f64 {
    rect.map_or(160.0, |rect| {
        if kind == NumericKind::Slider || horizontal {
            rect.width
        } else {
            rect.height
        }
    })
}

/// Take a new anchor at `along` from the value last written, so movement so
/// far keeps the factor that applied to it and a saturated or quantized value
/// restarts from where it is.
pub(super) fn reanchor(
    anchor: Anchor,
    along: f64,
    factor: f64,
    applied: f64,
    range: ScalarRange,
    mapping: ScalarMapping,
) -> Anchor {
    let origin = match anchor.space {
        ScrubSpace::Value => applied,
        ScrubSpace::Position => mapping.position(applied, range),
    };
    Anchor {
        along,
        origin,
        factor,
        space: anchor.space,
    }
}

impl NumericView {
    pub(super) fn scrub(view: &Rc<NumericView>) -> DragApi<f64> {
        let start = Rc::clone(view);
        let update = Rc::clone(view);
        let modifiers = Rc::clone(view);
        let end = Rc::clone(view);
        view.pointer
            .on_start(move |event| start.scrub_start(event))
            .on_update(move |event| update.scrub_update(event))
            .on_modifiers_changed(move |event| modifiers.scrub_modifiers(event))
            .on_end(move |event| end.scrub_end(event))
    }

    /// The space and units per pixel of the scrub, from the current geometry.
    fn scrub_units_now(&self) -> (ScrubSpace, f64) {
        let distance = self.drag_distance.map(|distance| distance.get_untracked());
        let extent = scrub_extent(self.kind, self.horizontal.get_untracked(), self.rect.get());
        scrub_units(self.kind, distance, self.step.get_untracked(), extent)
    }

    fn scrub_factor(&self, modifiers: kivact::view::Modifiers) -> f64 {
        self.scrub_policy().factor(modifiers)
    }

    /// The pointer's travel along the drag axis of this update.
    fn scrub_along(&self, update: &DragUpdate<f64>) -> f64 {
        scrub_along(self.kind, self.horizontal.get_untracked(), update.delta)
    }

    fn scrub_start(&self, update: DragUpdate<f64>) {
        let (space, _) = self.scrub_units_now();
        let start = update.payload;
        let origin = match space {
            ScrubSpace::Value => start,
            ScrubSpace::Position => self.mapping.position(start, self.range.get_untracked()),
        };
        let factor = self.scrub_factor(update.modifiers);
        self.mode.send(machine::Event::Press(Anchor {
            along: 0.0,
            origin,
            factor,
            space,
        }));
    }

    fn scrub_update(&self, update: DragUpdate<f64>) {
        let along = self.scrub_along(&update);
        let (space, units) = self.scrub_units_now();
        let machine::State::Scrubbing { moved, anchor } = self.mode.get() else {
            return;
        };
        // The dead zone only applies before the first move, so dragging back
        // near the start keeps writing.
        if !moved && along.abs() <= 2.0 {
            return;
        }
        let factor = self.scrub_factor(update.modifiers);
        let anchor = if factor != anchor.factor {
            let anchor = self.reanchor(anchor, along, factor);
            self.mode.send(machine::Event::Reanchor(anchor));
            anchor
        } else {
            anchor
        };
        let target = anchor.origin + (along - anchor.along) * units * factor;
        match space {
            ScrubSpace::Value => {
                self.scalar.set(target);
            }
            ScrubSpace::Position => {
                self.scalar.set_position(target);
            }
        }
        self.mode.send(machine::Event::Move);
    }

    fn scrub_modifiers(&self, update: DragUpdate<f64>) {
        let along = self.scrub_along(&update);
        let machine::State::Scrubbing { anchor, .. } = self.mode.get() else {
            return;
        };
        let factor = self.scrub_factor(update.modifiers);
        if factor == anchor.factor {
            return;
        }
        let anchor = self.reanchor(anchor, along, factor);
        self.mode.send(machine::Event::Reanchor(anchor));
    }

    fn scrub_end(&self, update: DragUpdate<f64>) {
        let moved = matches!(
            self.mode.get(),
            machine::State::Scrubbing { moved: true, .. }
        );
        self.mode.send(machine::Event::End);
        if update.phase != DragPhase::Cancel
            && !moved
            && self.text_entry
            && matches!(self.kind, NumericKind::Number | NumericKind::Slider)
        {
            self.typing.show(());
        }
    }

    /// Re-anchor at the value last written, which keeps saturation and
    /// quantization in force.
    fn reanchor(&self, anchor: Anchor, along: f64, factor: f64) -> Anchor {
        reanchor(
            anchor,
            along,
            factor,
            self.scalar.value().get_untracked(),
            self.range.get_untracked(),
            self.mapping.clone(),
        )
    }
}
