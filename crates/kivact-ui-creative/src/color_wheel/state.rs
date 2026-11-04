//! Role: Color picker values and shared edit transaction.
use super::geometry::Channel;
use super::{color_from_hsv, color_to_hsv};
use crate::editors::shared::owned_edit::OwnedEdit;
use kivact::interaction::DragApi;
use kivact::state::IntoValue;
use kivact::state::{Binding, Callback, Signal, Var, signal, var};
use kivact::style::Color;
use kivact::view::EditPhase;

type Snapshot = (Color, f64, Option<Vec<Color>>);
/// Who began the active edit of the shared session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ColorOwner {
    Wheel,
    Field { hsv: bool, index: usize },
    Hex,
    Swatch,
}
#[derive(Clone, Copy)]
pub(super) struct State {
    pub(super) color: Binding<Color>,
    pub(super) hue: Signal<f64>,
    pub(super) palette: Option<Binding<Vec<Color>>>,
    pub(super) edit: OwnedEdit<ColorOwner, Snapshot>,
    /// The wheel's drag, once the wheel made it; a drag owns the color.
    pub(super) drag: Var<Option<DragApi<Channel>>>,
    pub(super) blocked: Signal<bool>,
}
impl State {
    pub(super) fn new(
        color: Binding<Color>,
        palette: Option<Binding<Vec<Color>>>,
        observer: Option<Callback<EditPhase>>,
    ) -> Self {
        let hue = signal(color_to_hsv(color.into_value().get_untracked()).0);
        let edit = OwnedEdit::new(
            move |(original, h, swatches): Snapshot| {
                color.set(original);
                hue.set(h);
                if let (Some(binding), Some(colors)) = (palette, swatches) {
                    binding.set(colors);
                }
            },
            observer,
        );
        Self {
            color,
            hue,
            palette,
            edit,
            drag: var(None),
            blocked: signal(false),
        }
    }
    /// What an edit writes back when it is canceled.
    pub(super) fn snapshot(self) -> Snapshot {
        (
            self.color.into_value().get_untracked(),
            self.hue.get_untracked(),
            self.palette.map(|p| p.into_value().get_untracked()),
        )
    }
    /// One whole edit of `owner` around `write`; nothing while the state is
    /// blocked or another source's edit runs.
    pub(super) fn once(self, owner: ColorOwner, write: impl FnOnce()) {
        if self.blocked.get_untracked() {
            return;
        }
        self.edit.once(owner, || self.snapshot(), write);
    }
    pub(super) fn hsv(self) -> (f64, f64, f64, f64) {
        let color = self.color.into_value().get_untracked();
        let (_, s, v) = color_to_hsv(color);
        (self.hue.get_untracked(), s, v, color.components()[3])
    }
    pub(super) fn channel(self, c: Channel) -> (f64, f64) {
        let (h, s, v, a) = self.hsv();
        match c {
            Channel::Hue => (h, h),
            Channel::Sv => (s, v),
            Channel::Alpha => (a, a),
        }
    }
    pub(super) fn set(self, c: Channel, x: f64, y: f64) {
        let (mut h, mut s, mut v, mut a) = self.hsv();
        match c {
            Channel::Hue => h = x.rem_euclid(360.0),
            Channel::Sv => {
                s = x.clamp(0.0, 1.0);
                v = y.clamp(0.0, 1.0)
            }
            Channel::Alpha => a = x.clamp(0.0, 1.0),
        }
        self.hue.set(h);
        self.color.set(color_from_hsv(h, s, v, a));
    }
    /// Cancel the active edit: a drag of the wheel, or a field's.
    pub(super) fn cancel(self) {
        if let Some(drag) = self.drag.get() {
            drag.cancel();
        }
        self.edit.cancel();
    }
    /// A phase of an edit that `owner` reports.
    pub(super) fn phase(self, owner: ColorOwner, phase: EditPhase) {
        match phase {
            EditPhase::Begin => {
                if !self.blocked.get_untracked() {
                    self.edit.begin(owner, self.snapshot());
                }
            }
            EditPhase::Commit => self.edit.commit(owner),
            EditPhase::Cancel => {
                self.edit.cancel_owned(owner);
            }
        }
    }
}
