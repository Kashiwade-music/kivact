//! Role: Passive state and appearance settings for standard component recipes.
//!
//! Each recipe gets the actual state of the part it styles. Checkbox, switch
//! and radio items get [`CheckState`] (the real `CheckedState` plus size and
//! invalid); a select's trigger gets [`SelectTriggerState`], its surface
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
use kivact_core::primitive::PartState;
use kivact_core::view::Orientation;
use kivact_interaction::{CheckedState, DialogKind};

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

/// The metrics a control is drawn at; the recipe decides what each size
/// means.
///
/// Buttons, text inputs and areas, selects, checkboxes, switches, radio
/// groups and sliders take one with `.size(..)` and pass it to their
/// recipes in their state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ControlSize {
    /// The most compact size, for dense toolbars and inline actions.
    ExtraSmall,
    /// A compact size.
    Small,
    /// The ordinary size.
    #[default]
    Medium,
    /// A roomy size.
    Large,
}

/// What a button's recipes know of it: its root, its content row, its
/// icon boxes and its busy indicator get the same state.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonState {
    /// The root's actual input state. Hover, press and focus looks are
    /// `style().hover(..)`, `.pressed(..)` and `.focus_visible(..)` in the
    /// returned style; read this only for what a state style cannot change
    /// (the default `Link` underlines its label while hovered: a text
    /// decoration is not a state style property).
    pub input: PartState,
    pub size: ControlSize,
    pub loading: bool,
    /// Whether the button opens something and whether that is open: `None`
    /// for a plain button, `Some(open)` when the button publishes an
    /// expanded state (a popup, menu or disclosure attached its trigger to
    /// it, or the caller set `.expanded(..)`). A look draws an open
    /// trigger as such (shadcn's `aria-expanded:bg-muted`) and does not
    /// nudge a popup trigger while pressed.
    pub expanded: Option<bool>,
    /// Whether the button shows a leading icon ([`Button::icon`](super::Button::icon)).
    pub leading_icon: bool,
    /// Whether the button shows a trailing icon
    /// ([`Button::trailing_icon`](super::Button::trailing_icon)).
    pub trailing_icon: bool,
    /// Whether the button shows only its leading icon: it has one and its
    /// label is hidden ([`Button::hide_label`](super::Button::hide_label)).
    /// A look draws it as a square of the button's height.
    pub icon_only: bool,
}

impl ButtonState {
    /// The state of a plain, idle button: for a component that draws one of
    /// its own parts as the theme's button (a kit's toggle, say) by calling
    /// the theme's [`UiRecipes::button`](super::UiRecipes) recipe with
    /// [`Look::new`](super::Look::new) with the button's variant added by
    /// [`Look::with_variant`](super::Look::with_variant).
    pub fn new(input: PartState, size: ControlSize) -> Self {
        ButtonState {
            input,
            size,
            loading: false,
            expanded: None,
            leading_icon: false,
            trailing_icon: false,
            icon_only: false,
        }
    }
}

/// The state of a [`toggle`](super::toggle) for its recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleButtonState {
    /// The actual input state of the toggle.
    pub input: PartState,
    /// Whether it is pressed (on).
    pub pressed: bool,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    /// Whether the toggle shows a leading icon ([`Toggle::icon`](super::Toggle::icon)).
    pub leading_icon: bool,
    /// Whether the toggle shows only its leading icon: it has one and its
    /// label is hidden ([`Toggle::hide_label`](super::Toggle::hide_label)).
    /// A look draws it as a square of the toggle's height.
    pub icon_only: bool,
}

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
    /// from 0 when the bar appears; 0 while indeterminate. A recipe that
    /// paints the fill reads it in the paint, which follows it. The handle
    /// is made at setup and stays the same for the mounted bar; it is not
    /// part of the state's equality.
    pub shown: kivact_core::Value<f64>,
    /// A clock from 0 to 1 that repeats about every 1.3 s while the bar is
    /// indeterminate or [`Progress::moving`](super::Progress::moving) is true,
    /// and rests at 0 otherwise (and under reduced motion). A recipe reads it
    /// in a paint that moves. The handle is made at setup and stays the same
    /// for the mounted bar; it is not part of the state's equality.
    pub phase: kivact_core::Value<f32>,
}

impl ProgressState {
    /// Where an indeterminate bar's moving segment is, as `(start, length)`
    /// in fractions of the track: it enters from the left, crosses and
    /// leaves at the right (the start is below 0 or above 1 while it is partly
    /// or wholly outside). It follows [`phase`](Self::phase), so a paint that
    /// calls it moves; under reduced motion it rests at the track's start.
    pub fn sweep(&self) -> (f32, f32) {
        const LENGTH: f32 = 0.3;
        if kivact_core::platform::reduced_motion() {
            return (0.0, LENGTH);
        }
        let eased = (1.0 - (self.phase.get() * std::f32::consts::PI).cos()) / 2.0;
        (LENGTH * (-1.1 + 4.7 * eased), LENGTH)
    }
}

impl PartialEq for ProgressState {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.size == other.size
    }
}

/// The state of a text input or a text area for its recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextInputState {
    pub input: PartState,
    pub size: ControlSize,
    /// Whether the caller marked the value invalid (`TextInput::invalid`).
    pub invalid: bool,
}

/// The state of a [`field`](super::field) for the recipes of its parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldState {
    /// Whether the field shows an error message (`Field::error` is `Some`).
    pub invalid: bool,
}

/// The state of a check box, a switch or a radio item for its recipe.
#[derive(Clone, Copy, Debug)]
pub struct CheckState {
    /// The actual input state of the control.
    pub input: PartState,
    /// Whether it is on; only a check box is ever
    /// [`Indeterminate`](CheckedState::Indeterminate).
    pub checked: CheckedState,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    /// Whether the caller marked the value invalid (`.invalid(..)`).
    pub invalid: bool,
    /// The localized word for the state, for a recipe that paints it: a
    /// switch's `switch.on` ("ON") or `switch.off` ("OFF") message, `None`
    /// for a check box and a radio item. The handle is made at setup and
    /// stays the same for the mounted control; read it in the paint, which
    /// follows its text. It is not part of the state's equality.
    pub text: Option<kivact_core::Value<String>>,
}

impl PartialEq for CheckState {
    fn eq(&self, other: &Self) -> bool {
        self.input == other.input
            && self.checked == other.checked
            && self.size == other.size
            && self.invalid == other.invalid
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DialogState {
    pub input: PartState,
    pub open: bool,
    pub kind: DialogKind,
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
    pub side: kivact_interaction::Side,
    /// Where the trigger's centre is along that edge, in logical pixels
    /// from the tooltip's left (`Top`, `Bottom`) or top (`Left`, `Right`)
    /// edge, within the tooltip.
    pub offset: f32,
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

/// The state of a part laid along an axis, for its recipe: a slider's
/// track, range and thumb, and a splitter's handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisState {
    /// The actual input state of the part (the thumb's and the handle's
    /// own; the track's and the range's are the track's).
    pub input: PartState,
    /// The direction the value grows along: a horizontal slider's track is
    /// a row, a vertical one's a column. A splitter's is the direction its
    /// panes are laid out in, so the handle between panes in a row is a
    /// vertical bar.
    pub orientation: Orientation,
    /// The size the caller chose with `.size(..)` (default `Medium`).
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
    pub height: Option<f32>,
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
pub struct DataTableCheckState {
    /// The box is in the header and checks the rows of the page.
    pub header: bool,
    /// Whether the check box is on; the header's is mixed while some of the
    /// page's rows are checked.
    pub checked: CheckedState,
}

/// The page a [`data_table`](super::data_table) shows, for the recipe of its
/// pager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataTablePageState {
    /// The page shown, from 0.
    pub index: usize,
    /// How many pages the filtered rows fill; at least 1.
    pub pages: usize,
}
