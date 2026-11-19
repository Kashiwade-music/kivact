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
use kivact_core::custom::PartState;
use kivact_core::view::CheckedState;
use kivact_interaction::DialogMode;

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

/// Standard presentation of a radio group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RadioGroupVariant {
    /// Adjacent choices drawn as a segmented control.
    Segmented,
}
impl Variant for RadioGroupVariant {}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DialogState {
    pub input: PartState,
    pub open: bool,
    pub kind: DialogMode,
    /// The dialog has a [`media`](super::Dialog::media) view in its header,
    /// which the header recipe lays out beside or above the title.
    pub media: bool,
}

/// Where a tooltip's arrow points, for the tooltip's arrow recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TooltipArrowState {
    /// The side of the trigger the tooltip shows on (base-ui's
    /// `data-side`): `Top` when it is above the trigger, so the arrow sits
    /// at the tooltip's bottom edge. It follows the host's flip once the
    /// tooltip is placed; before that, it is the requested placement.
    pub side: kivact_core::geometry::Edge,
    /// Where the trigger's centre is along that edge, in logical pixels
    /// from the tooltip's left (`Top`, `Bottom`) or top (`Left`, `Right`)
    /// edge, within the tooltip.
    pub offset: f64,
}

/// The state of a select's trigger for its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectTriggerState {
    /// The actual input state of the trigger.
    pub input: PartState,
    /// Whether the list is open.
    pub open: bool,
    /// Whether the trigger shows the placeholder: no option is selected
    /// (the selected key is `None` or not among the options).
    pub placeholder: bool,
    /// The size the caller chose with `Select::size` (default `Medium`).
    pub size: ControlSize,
}

/// The state of one live region's column of toasts in a
/// [`toast_region`](super::toast_region), for its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToastGroupState {
    /// The actual input state of the column.
    pub input: PartState,
    /// The assertive group (warnings and errors), below the polite one.
    pub assertive: bool,
    /// How many toasts the group shows.
    pub count: usize,
    /// How many toasts the groups above this one show (the polite group's
    /// for the assertive group, 0 for the polite group), so a look puts
    /// space between two groups only when both show toasts.
    pub above: usize,
}

/// The state of a main row of a [`data_table`](super::data_table) for its
/// recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataTableRowState {
    /// The actual state of the row's attached table part: its input state,
    /// whether it is selected and whether it holds the tab stop.
    pub interaction: kivact_interaction::TableRowState,
    /// The accent the caller gave the row with
    /// [`DataTable::row_color`](super::DataTable::row_color); the recipe
    /// resolves it in the theme in effect.
    pub color: Option<kivact_core::style::Color>,
    /// The height the caller gave the rows with
    /// [`DataTable::row_height`](super::DataTable::row_height), at least 18.
    pub height: Option<f64>,
}

/// The state of a cell of a [`data_table`](super::data_table) for its
/// recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DataTableCellState {
    /// The actual state of the cell's attached table part.
    pub interaction: kivact_interaction::TableCellState,
    /// Whether the column is [`numeric`](super::DataColumn::numeric).
    pub numeric: bool,
}

/// The state of the box around a check box of a
/// [`data_table`](super::data_table), for its recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataTableCheckState {
    /// The header's box, which checks the page's rows; Indeterminate while
    /// some of them are checked.
    Header(CheckedState),
    /// A row's box.
    Row { checked: bool },
}

impl DataTableCheckState {
    /// Whether this is the header's box, which checks the page's rows.
    pub fn is_header(&self) -> bool {
        matches!(self, Self::Header(_))
    }
    /// The box's checked state; only the header's is ever Indeterminate.
    pub fn checked(&self) -> CheckedState {
        match *self {
            Self::Header(checked) => checked,
            Self::Row { checked } => CheckedState::from_bool(checked),
        }
    }
}

/// The page a [`data_table`](super::data_table) shows, for the recipe of its
/// pager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataTablePageState {
    /// The page shown, from 0.
    pub index: usize,
    /// How many pages the filtered rows fill. Zero for a remote page with unknown total.
    pub pages: usize,
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
