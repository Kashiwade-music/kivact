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
use kivact_core::custom::PartState;
use kivact_core::view::CheckedState;
use kivact_core::view::Orientation;
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

#[derive(Clone, Copy)]
struct InheritedControlSize(ControlSize);

/// Set the default size of standard controls created in `content`.
/// A control's explicit `.size(..)` wins, including `ControlSize::Medium`.
/// Nested providers use the nearest default.
pub fn provide_control_size<V: kivact_core::view::IntoView>(
    size: ControlSize,
    content: impl FnOnce() -> V + 'static,
) -> kivact_core::view::View {
    kivact_core::state::provide(InheritedControlSize(size), content)
}

pub(super) fn default_control_size() -> ControlSize {
    kivact_core::state::try_use_context::<InheritedControlSize>()
        .map_or_else(ControlSize::default, |size| size.get().0)
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
    // xtask-allow(state-fields): independent: busy comes from the caller's busy value and is valid with any icon or label layout
    pub busy: bool,
    // xtask-allow(state-fields): option: None for a plain button, Some(open) when a popup or the caller publishes an expanded state
    /// Whether the button opens something and whether that is open: `None`
    /// for a plain button, `Some(open)` when the button publishes an
    /// expanded state (a popup, menu or disclosure attached its trigger to
    /// it, or the caller set `.expanded(..)`). A look draws an open
    /// trigger as such (shadcn's `aria-expanded:bg-muted`) and does not
    /// nudge a popup trigger while pressed.
    pub expanded: Option<bool>,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    /// Whether the button shows a leading icon ([`Button::icon`](super::Button::icon)).
    pub leading_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    /// Whether the button shows a trailing icon
    /// ([`Button::trailing`](super::Button::trailing)).
    pub trailing_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon, the trailing icon and the hidden label are set by separate builder calls and all eight combinations are drawn
    /// Whether the label is hidden ([`Button::hide_label`](super::Button::hide_label));
    /// it stays the accessible name.
    pub label_hidden: bool,
}

impl ButtonState {
    /// Whether only the leading icon shows: the button has one and its
    /// label is hidden. A look draws it as a square of the button's height.
    pub fn is_icon_only(&self) -> bool {
        self.leading_icon && self.label_hidden
    }

    /// The state of a plain, idle button: for a component that draws one of
    /// its own parts as the theme's button (a kit's toggle, say) by calling
    /// the theme's [`UiRecipes::button`](super::UiRecipes) recipe with
    /// [`Look::new`](super::Look::new) with the button's variant added by
    /// [`Look::with_variant`](super::Look::with_variant).
    pub fn new(input: PartState, size: ControlSize) -> Self {
        ButtonState {
            input,
            size,
            busy: false,
            expanded: None,
            leading_icon: false,
            trailing_icon: false,
            label_hidden: false,
        }
    }
}

/// The state of a [`toggle_button`](super::toggle) for its recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToggleButtonState {
    /// The actual input state of the toggle.
    pub input: PartState,
    // xtask-allow(state-fields): independent: the pressed state follows the toggle and is valid with any icon or label layout
    /// Whether it is pressed (on).
    pub pressed: bool,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    // xtask-allow(state-fields): independent: the leading icon and the hidden label are set separately; all four combinations are drawn
    /// Whether the toggle shows a leading icon ([`ToggleButton::icon`](super::ToggleButton::icon)).
    pub leading_icon: bool,
    // xtask-allow(state-fields): independent: the leading icon and the hidden label are set separately; all four combinations are drawn
    /// Whether the label is hidden ([`ToggleButton::hide_label`](super::ToggleButton::hide_label),
    /// or [`ToggleGroup::icon_only`](super::ToggleGroup::icon_only) for a group's item).
    pub label_hidden: bool,
}

impl ToggleButtonState {
    /// Whether only the leading icon shows: the toggle has one and its label
    /// is hidden. A look draws it as a square of the toggle's height.
    pub fn is_icon_only(&self) -> bool {
        self.leading_icon && self.label_hidden
    }
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

/// The visual position of a switch thumb during an opt-in gesture.
///
/// Only the combinations the gesture machine reaches exist: a touch press, a
/// tracked contact (touch with its offset, or a pointer), and a released touch
/// thumb which is still raised or has settled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThumbDrag {
    /// A touch contact pressed the switch before moving it.
    TouchPressed,
    /// The contact moves the thumb.
    Tracking(TrackInput),
    /// A touch contact was released.
    TouchReleased(Lift),
}

impl ThumbDrag {
    /// Whether the gesture follows a touch contact.
    pub fn is_touch(self) -> bool {
        match self {
            Self::TouchPressed | Self::TouchReleased(_) => true,
            Self::Tracking(TrackInput::Touch { .. }) => true,
            Self::Tracking(TrackInput::Pointer { .. }) => false,
        }
    }
}

/// What a tracked switch gesture follows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TrackInput {
    /// A touch contact, at a physical offset along the writing direction.
    Touch {
        /// Physical translation along the writing direction, in logical pixels.
        offset: f64,
        /// The side the thumb shows before the binding commits.
        on: bool,
    },
    /// A mouse or pen pointer.
    Pointer {
        /// The side the thumb shows before the binding commits.
        on: bool,
    },
}

/// Whether a released touch thumb is still raised or has settled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lift {
    /// The thumb is still raised above the track.
    Raised,
    /// The thumb has settled back onto the track.
    Settled,
}

/// Which control a [`CheckableState`] describes, with the state only that
/// control has.
#[derive(Clone, Copy, Debug)]
pub enum CheckControl {
    /// A check box: on, off or mixed
    /// ([`Checkbox::indeterminate`](super::Checkbox::indeterminate)).
    CheckBox(CheckedState),
    /// A switch: on or off.
    Switch {
        // xtask-allow(state-fields): independent: the switch's bound value; the opted-in drag starts from either value and moves the thumb whichever one the binding holds
        /// Whether the switch is on.
        on: bool,
        /// The localized word for the state: `switch.on` ("ON") or
        /// `switch.off` ("OFF"). The handle is made at setup and stays the
        /// same for the mounted control; read it in the paint, which follows
        /// its text. It is not part of the state's equality.
        text: kivact_core::state::Value<String>,
        // xtask-allow(state-fields): independent: the live visual of the opted-in drag gesture, None when the switch has no drag or none is running; it runs while the switch is on or off
        /// Live visual state while the opted-in drag gesture moves the thumb.
        drag: Option<ThumbDrag>,
    },
    /// A radio item: whether it is the group's chosen option.
    Radio {
        /// Whether this option is chosen.
        selected: bool,
    },
}

impl PartialEq for CheckControl {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::CheckBox(a), Self::CheckBox(b)) => a == b,
            (
                Self::Switch {
                    on: a,
                    drag: drag_a,
                    ..
                },
                Self::Switch {
                    on: b,
                    drag: drag_b,
                    ..
                },
            ) => a == b && drag_a == drag_b,
            (Self::Radio { selected: a }, Self::Radio { selected: b }) => a == b,
            _ => false,
        }
    }
}

/// The state of a check box, a switch or a radio item for its recipe.
///
/// Most recipes read [`is_checked`](Self::is_checked) and
/// [`is_mixed`](Self::is_mixed); `match state.control` tells the three
/// controls apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CheckableState {
    /// The actual input state of the control.
    pub input: PartState,
    /// Which control this is, with its checked state.
    pub control: CheckControl,
    /// The size the caller chose with `.size(..)` (default `Medium`).
    pub size: ControlSize,
    /// Whether the caller marked the value invalid: `.invalid(..)` on the
    /// check box or switch, or on the radio group around a radio item
    /// (a `field` with an error sets it too).
    pub invalid: bool,
}

impl CheckableState {
    /// The checked state as assistive technology reports it; only a check
    /// box is ever [`Indeterminate`](CheckedState::Indeterminate).
    pub fn checked(&self) -> CheckedState {
        match self.control {
            CheckControl::CheckBox(checked) => checked,
            CheckControl::Switch { on, .. } => CheckedState::from_bool(on),
            CheckControl::Radio { selected } => CheckedState::from_bool(selected),
        }
    }

    /// Whether it is on: a checked box, a switch that is on, or the chosen
    /// radio item.
    pub fn is_checked(&self) -> bool {
        self.checked().is_checked()
    }

    /// Whether a check box shows its mixed (partially checked) state.
    pub fn is_mixed(&self) -> bool {
        matches!(
            self.control,
            CheckControl::CheckBox(CheckedState::Indeterminate)
        )
    }

    /// A switch's localized word for its state; `None` for a check box and
    /// a radio item.
    pub fn switch_text(&self) -> Option<kivact_core::state::Value<String>> {
        match self.control {
            CheckControl::Switch { text, .. } => Some(text),
            CheckControl::CheckBox(_) | CheckControl::Radio { .. } => None,
        }
    }

    /// The live visual of a switch's opted-in drag; `None` otherwise.
    pub fn switch_drag(&self) -> Option<ThumbDrag> {
        match self.control {
            CheckControl::Switch { drag, .. } => drag,
            CheckControl::CheckBox(_) | CheckControl::Radio { .. } => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DialogState {
    pub input: PartState,
    // xtask-allow(state-fields): independent: the popup's open state (Shape::state) or the drawer's open binding; a dialog with or without media opens and closes
    pub open: bool,
    pub kind: DialogMode,
    // xtask-allow(state-fields): independent: Dialog::media gave a header view (the drawer's drag handle reports false); it is fixed while the dialog opens and closes
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
    // xtask-allow(state-fields): independent: the select's popup or the selection field's FieldState::Open; the list opens with or without a selected option
    /// Whether the list is open.
    pub open: bool,
    // xtask-allow(state-fields): independent: no listed option is selected (trigger_projection; a selection field reports false); the trigger shows it while the list is open or closed
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
    // xtask-allow(state-fields): option: DataTable::row_color for this row; absent keeps the theme's row background
    /// The accent the caller gave the row with
    /// [`DataTable::row_color`](super::DataTable::row_color); the recipe
    /// resolves it in the theme in effect.
    pub color: Option<kivact_core::style::Color>,
    // xtask-allow(state-fields): option: DataTable::row_height (at least 18); absent uses the theme's row height
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

kivact_core::state::constant_value!(ControlSize);

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
