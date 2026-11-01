//! Role: Color picker values and shared edit transaction.
use super::geometry::Channel;
use super::{color_from_hsv, color_to_hsv};
use kivact::interaction::{Drag, EditSession};
use kivact::style::Color;
use kivact::ui::EditPhase;
use kivact::{Binding, Callback, Signal, Var, signal, var};

type Snapshot = (Color, f64, Option<Vec<Color>>);
#[derive(Clone, Copy)]
pub(super) struct State {
    pub(super) color: Binding<Color>,
    pub(super) hue: Signal<f64>,
    pub(super) palette: Option<Binding<Vec<Color>>>,
    pub(super) edit: EditSession<Snapshot>,
    /// The wheel's drag, once the wheel made it; a drag owns the color.
    pub(super) drag: Var<Option<Drag<Channel>>>,
    pub(super) blocked: Signal<bool>,
}
impl State {
    pub(super) fn new(
        color: Binding<Color>,
        palette: Option<Binding<Vec<Color>>>,
        observer: Option<Callback<EditPhase>>,
    ) -> Self {
        let hue = signal(color_to_hsv(color.into_value().get_untracked()).0);
        let edit = crate::editors::shared::model::observed(
            EditSession::new(move |(original, h, swatches): Snapshot| {
                color.set(original);
                hue.set(h);
                if let (Some(binding), Some(colors)) = (palette, swatches) {
                    binding.set(colors);
                }
            }),
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
    /// A drag of the wheel is in progress.
    pub(super) fn dragging(self) -> bool {
        self.drag.get().is_some_and(|drag| drag.is_active())
    }
    pub(super) fn once(self, write: impl FnOnce()) {
        if self.dragging() || self.blocked.get_untracked() {
            return;
        }
        self.edit.once(self.snapshot(), write);
    }
    pub(super) fn hsv(self) -> (f64, f64, f64, f32) {
        let color = self.color.into_value().get_untracked();
        let (_, s, v) = color_to_hsv(color);
        (self.hue.get_untracked(), s, v, color.components()[3])
    }
    pub(super) fn channel(self, c: Channel) -> (f64, f64) {
        let (h, s, v, a) = self.hsv();
        match c {
            Channel::Hue => (h, h),
            Channel::Sv => (s, v),
            Channel::Alpha => (f64::from(a), f64::from(a)),
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
            Channel::Alpha => a = x.clamp(0.0, 1.0) as f32,
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
    pub(super) fn phase(self, phase: EditPhase) {
        if self.dragging() || self.blocked.get() {
            return;
        }
        match phase {
            EditPhase::Begin => self.edit.begin(self.snapshot()),
            EditPhase::Commit => self.edit.commit(),
            EditPhase::Cancel => {
                self.edit.cancel();
            }
            // Variants added later fall back to being ignored.
            _ => {}
        }
    }
}
use kivact::view::IntoValue;
