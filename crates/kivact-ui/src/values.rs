//! Role: Passive state and appearance settings for standard component recipes.
//!
//! Each recipe gets the actual state of the part it styles. Checkbox, switch
//! and radio items get [`CheckableState`] (which control it is and its checked
//! state, [`CheckControl`], plus size and invalid); a select's trigger gets [`SelectTriggerState`], its surface
//! `PopupState` and its options `ListItemState`; a tooltip's optional arrow
//! gets [`TooltipArrowState`], found from the anchor's and the surface's
//! rects after the host's flip (the requested side before placement); a
//! menu item gets `MenuItemState`, and headings, separators and inset boxes
//! `PartState`; a tab list gets `TabListState` and a tab or panel
//! `TabState`; a radio group's group gets its `Orientation`; a slider's
//! track, range and thumb and a splitter's handle get [`AxisState`]; a toast
//! gets `ToastState` and each live region's column [`ToastGroupState`]; a
//! dialog gets [`DialogState`], a text field [`TextInputState`] and each
//! part of a form field [`FieldState`].
//!
//! A [`data_table`](super::data_table)'s rows, cells, check boxes and pager
//! get [`DataTableRowState`], [`DataTableCellState`], [`DataTableCheckState`]
//! and [`DataTablePageState`], its headers the table part's
//! `TableHeaderState`, and a [`list`](super::list)'s items `ListItemState`.
//!
//! A localized word a recipe paints (a switch's `switch.on` and
//! `switch.off`) is a handle made at setup and kept for the mount; the paint
//! reads its text reactively, the handle is not part of the state's
//! equality, and no message handle is created while drawing.
use super::variants::Variant;
use crate::control_size::ControlSize;

/// What a button does, which its recipe draws (shadcn's `variant`).
///
/// These are meanings every theme draws; a kit's own looks (a tone, a
/// shape) are further [`Variant`](super::Variant) values:
/// [`Button::variant`](super::Button::variant) takes this and a kit's type
/// alike, and a recipe reads each with
/// [`Look::variant`](super::Look::variant) (a button without a
/// `ButtonVariant` is the default, `Primary`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// The main action of its area, filled with the accent.
    #[default]
    Primary,
    /// A secondary action on the surface.
    Secondary,
    /// A bordered button on the surface for a secondary action beside a
    /// primary one; the default look is a 1px border on the surface.
    Outline,
    /// A button with no fill until it is hovered, for toolbars and quiet actions.
    Ghost,
    /// An action that destroys or removes something.
    Danger,
    /// A text-only button in the accent color that reads as an inline link;
    /// the default look underlines it only while hovered.
    Link,
}

impl Variant for ButtonVariant {}

/// The state of a [`progress`](super::progress) bar for the recipes of its
/// track and fill.
#[derive(Clone, Copy, Debug)]
pub struct ProgressState {
    /// The completed fraction from 0 to 1 (a value outside it is clamped),
    /// or `None` while the bar is indeterminate.
    pub value: Option<f64>,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    /// The fraction the bar is shown at, from 0 to 1: it moves toward
    /// `value` over a short time (at once under reduced motion), starting
    /// from 0 by default when the bar appears; 0 while indeterminate. A recipe that
    /// paints the fill reads it in the paint, which follows it. The handle
    /// is made at setup and stays the same for the mounted bar; it is not
    /// part of the state's equality.
    pub shown: kivact_core::state::Value<f64>,
    /// A clock from 0 to 1 that repeats with the theme's
    /// [`ProgressMotion`](super::ProgressMotion) period while the bar is
    /// indeterminate or [`Progress::moving`](super::Progress::moving) is true.
    /// The clock rests at 0 under reduced motion by default. A recipe reads it
    /// in a paint that moves. The handle is made at setup and stays the same
    /// for the mounted bar; it is not part of the state's equality.
    pub phase: kivact_core::state::Value<f64>,
}

impl ProgressState {
    /// Where an indeterminate bar's moving segment is, as `(start, length)`
    /// in fractions of the track: it enters from the left, crosses and
    /// leaves at the right (the start is below 0 or above 1 while it is partly
    /// or wholly outside). It follows [`phase`](Self::phase), so a paint that
    /// calls it moves; under reduced motion it rests at the track's start.
    pub fn sweep(&self) -> (f64, f64) {
        const LENGTH: f64 = 0.3;
        if kivact_core::platform::is_motion_reduced().get() {
            return (0.0, LENGTH);
        }
        let eased = (1.0 - (self.phase.get() * std::f64::consts::PI).cos()) / 2.0;
        (LENGTH * (-1.1 + 4.7 * eased), LENGTH)
    }
}

impl PartialEq for ProgressState {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.size == other.size
    }
}

/// Standard progress appearances, read through the existing variant contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ProgressVariant {
    /// A horizontal bar (the default).
    #[default]
    Bar,
    /// A circular track with visible percentage text.
    Ring,
    /// A segmented horizontal gauge.
    Gauge,
}
impl Variant for ProgressVariant {}
