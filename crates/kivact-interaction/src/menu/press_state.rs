//! Role: Store the held pointer, surfaces and row highlights of a menu.
use kivact_core::geometry::Point;
use kivact_core::state::{Signal, Value, Var, on_cleanup, signal, var};
use kivact_core::style::{Theme, ThemeExtension};
use kivact_core::task::Task;
use kivact_core::view::{NodeRef, PointerId, PointerKind};

/// A theme's opt-in to native menu press, drag and release selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuPress(pub bool);

impl ThemeExtension for MenuPress {
    fn fallback(_: &Theme) -> Self {
        Self::default()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Source {
    Trigger,
    Menu,
    Context,
}

#[derive(Clone, Copy)]
pub(super) enum Phase {
    Pending { timer: Task, from: Point },
    Open { at: u64, entered: bool },
}

#[derive(Clone, Copy)]
pub(super) struct Gesture {
    pub(super) id: PointerId,
    pub(super) kind: PointerKind,
    pub(super) phase: Phase,
    pub(super) source: Source,
}

#[derive(Clone, Copy)]
struct Row {
    reference: NodeRef,
    disabled: Value<bool>,
}

#[derive(Clone, Copy)]
pub(super) struct Press {
    pub(super) gesture: Var<Option<Gesture>>,
    surfaces: Var<Vec<NodeRef>>,
    rows: Var<Vec<Row>>,
    /// None means normal focus and hover. Some(None) clears row highlighting.
    pub(super) highlight: Signal<Option<Option<NodeRef>>>,
}

impl Press {
    pub(super) fn new() -> Self {
        Self {
            gesture: var(None),
            surfaces: var(Vec::new()),
            rows: var(Vec::new()),
            highlight: signal(None),
        }
    }

    pub(super) fn surface(self, node: NodeRef) {
        self.surfaces.update(|surfaces| surfaces.push(node));
        on_cleanup(move || {
            self.surfaces
                .update(|surfaces| surfaces.retain(|item| *item != node))
        });
    }

    pub(super) fn row(self, reference: NodeRef, disabled: Value<bool>) {
        self.rows.update(|rows| {
            rows.push(Row {
                reference,
                disabled,
            })
        });
        on_cleanup(move || {
            self.rows
                .update(|rows| rows.retain(|row| row.reference != reference))
        });
    }

    pub(super) fn highlighted(self, row: NodeRef) -> Option<bool> {
        self.highlight.get().map(|active| active == Some(row))
    }

    pub(super) fn hit(self, position: Point) -> (bool, Option<NodeRef>) {
        let inside = self
            .surfaces
            .with(|surfaces| surfaces.iter().any(|node| node.hit_test(position)));
        let row = inside
            .then(|| {
                self.rows.with(|rows| {
                    rows.iter()
                        .rev()
                        .find(|row| {
                            !row.disabled.get_untracked() && row.reference.hit_test(position)
                        })
                        .map(|row| row.reference)
                })
            })
            .flatten();
        (inside, row)
    }

    pub(super) fn cancel_open(self) {
        if self
            .gesture
            .get()
            .is_some_and(|gesture| matches!(gesture.phase, Phase::Open { .. }))
        {
            self.cancel();
        }
    }

    pub(super) fn cancel(self) {
        if let Some(gesture) = self.gesture.get()
            && let Phase::Pending { timer, .. } = gesture.phase
        {
            timer.cancel();
        }
        self.gesture.set(None);
        self.highlight.set(None);
    }
}
