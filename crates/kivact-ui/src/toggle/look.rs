//! Role: The toggle recipe state, recipe table and default looks.
use crate::look_common::{font_size, gap, height};
use crate::recipe::recipe_table;
use crate::theme::Look;

use crate::control_size::ControlSize;
use kivact_core::custom::PartState;
use kivact_core::draw::{Draw, LineCap, LineJoin, Path, Stroke};
use kivact_core::geometry::{Point, Rect, Size};
use kivact_core::style::{Align, FlexDirection, Style, Theme, style};
use kivact_core::view::CheckedState;
/// The side of a check box and the height of a switch's track.
pub(crate) fn indicator(theme: &Theme, size: ControlSize) -> f64 {
    font_size(theme, size).round() + 2.0
}

/// The room a toggle keeps at its left edge for its drawing.
pub(crate) fn leading(theme: &Theme) -> f64 {
    theme.get(&kivact_core::style::space::SM_VAR)
}

/// The width of the drawn part of a toggle.
pub(crate) fn indicator_width(theme: &Theme, size: ControlSize, switch: bool) -> f64 {
    let side = indicator(theme, size);
    if switch { (side * 1.8).round() } else { side }
}

/// The root of a check box or a switch: a row whose left edge keeps room
/// for the drawing of the recipe.
pub(crate) fn toggle(theme: &Theme, size: ControlSize, switch: bool) -> Style {
    style()
        .flex_direction(FlexDirection::Row)
        .align_items(Align::Center)
        .min_height(height(theme, size))
        .padding_left(leading(theme) + indicator_width(theme, size, switch) + gap(theme, size))
        .padding_right(theme.get(&kivact_core::style::space::SM_VAR))
        .border_radius(theme.get(&kivact_core::style::radius::MD_VAR))
        .font_size(font_size(theme, size))
        .color(kivact_core::style::color::TEXT)
        .hover(style().background(kivact_core::style::color::HOVER))
        .active(style().background(kivact_core::style::color::PRESSED))
}

pub(crate) fn checkbox(theme: &Theme, look: &Look<CheckableState>) -> Style {
    let (theme, state) = (theme.clone(), **look);
    toggle(&theme, state.size, false).paint(move |d, size| draw_checkbox(&theme, &state, d, size))
}

pub(crate) fn switch(theme: &Theme, look: &Look<CheckableState>) -> Style {
    let (theme, state) = (theme.clone(), **look);
    toggle(&theme, state.size, true).paint(move |d, size| draw_switch(&theme, &state, d, size))
}

/// The box of a check box: filled with the accent and a check mark (or a
/// bar when mixed) while on, an outlined box otherwise.
pub(crate) fn draw_checkbox(theme: &Theme, look: &CheckableState, d: &mut Draw, size: Size) {
    let side = indicator(theme, look.size);
    let r = theme.get(&kivact_core::style::radius::SM_VAR);
    let rect = Rect::new(leading(theme), (size.height - side) / 2.0, side, side);
    if look.checked() != CheckedState::Unchecked {
        d.rounded_rect(rect, r, kivact_core::style::color::ACCENT);
        let at = |x: f64, y: f64| Point::new(rect.x + side * x, rect.y + side * y);
        let mark = Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round);
        if look.is_mixed() {
            d.polyline(
                [at(0.25, 0.5), at(0.75, 0.5)],
                &mark,
                kivact_core::style::color::ACCENT_TEXT,
            );
        } else {
            d.polyline(
                [at(0.25, 0.52), at(0.43, 0.7), at(0.76, 0.3)],
                &mark,
                kivact_core::style::color::ACCENT_TEXT,
            );
        }
    } else {
        d.rounded_rect(rect, r, kivact_core::style::color::SURFACE);
        let inner = Rect::new(rect.x + 0.5, rect.y + 0.5, side - 1.0, side - 1.0);
        d.stroke_path(
            &Path::new().rounded_rect(inner, [(r, r); 4]),
            &Stroke::new(1.0),
            if look.invalid {
                kivact_core::style::color::DANGER
            } else {
                kivact_core::style::color::BORDER
            },
        );
    }
}

/// The track and thumb of a switch.
pub(crate) fn draw_switch(theme: &Theme, look: &CheckableState, d: &mut Draw, size: Size) {
    let on = look.is_checked();
    let h = indicator(theme, look.size);
    let w = indicator_width(theme, look.size, true);
    let track = Rect::new(leading(theme), (size.height - h) / 2.0, w, h);
    d.rounded_rect(
        track,
        h / 2.0,
        if on {
            kivact_core::style::color::ACCENT
        } else {
            kivact_core::style::color::BORDER
        },
    );
    let inset = 2.0;
    let thumb = h - 2.0 * inset;
    let x = if on {
        track.x + w - inset - thumb
    } else {
        track.x + inset
    };
    d.rounded_rect(
        Rect::new(x, track.y + inset, thumb, thumb),
        thumb / 2.0,
        if on {
            kivact_core::style::color::ACCENT_TEXT
        } else {
            kivact_core::style::color::SURFACE
        },
    );
}

/// The standard checkbox and switch draw their indicators in the root.
pub(crate) fn empty_toggle_part(_: &Theme, _: &Look<CheckableState>) -> Style {
    style().size(0, 0)
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
        /// Whether the switch is on.
        on: bool,
        /// The localized word for the state: `switch.on` ("ON") or
        /// `switch.off` ("OFF"). The handle is made at setup and stays the
        /// same for the mounted control; read it in the paint, which follows
        /// its text. It is not part of the state's equality.
        text: kivact_core::state::Value<String>,
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

recipe_table! {
    /// The recipes of a checkbox: its root, its indicator box and its marks.
    CheckboxTheme {
        root(root_recipe): CheckableState = checkbox,
        indicator(indicator_recipe): CheckableState = empty_toggle_part,
        check(check_recipe): CheckableState = empty_toggle_part,
        mixed(mixed_recipe): CheckableState = empty_toggle_part,
    }
}

recipe_table! {
    /// The recipes of a switch: its root, its track and its thumb.
    SwitchTheme {
        root(root_recipe): CheckableState = switch,
        track(track_recipe): CheckableState = empty_toggle_part,
        thumb(thumb_recipe): CheckableState = empty_toggle_part,
    }
}

impl CheckboxTheme {
    /// Whether a theme gave the checkbox its own indicator, check or mixed parts.
    pub(crate) fn has_parts(&self) -> bool {
        self.indicator.is_overridden() || self.check.is_overridden() || self.mixed.is_overridden()
    }
}

impl SwitchTheme {
    /// Whether a theme gave the switch its own track or thumb.
    pub(crate) fn has_parts(&self) -> bool {
        self.track.is_overridden() || self.thumb.is_overridden()
    }
}
