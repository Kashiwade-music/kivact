//! Role: Build themed binary toggles from their actual interaction part state.
//!
//! A checkbox or a switch takes its value as the constructor's one
//! `Signal<bool>`, like a select and a text input; there is no hidden local
//! binding and no separate constant `checked` mode. Its recipe gets the real
//! checked state in a `CheckableState`, and `.size` only reaches the recipe,
//! which decides the dimensions.
pub(crate) mod look;
mod parts;
mod switch_track;
mod view;

pub use view::{Checkbox, Switch, checkbox, switch};

use crate::control_size::ControlSize;
use kivact_core::custom::Part;
use kivact_core::state::{Callback, Signal, Value};
use kivact_core::view::EditPhase;
use kivact_interaction::ToggleState;

/// The live visual of a switch's opted-in drag: `None` when the drag is off.
type Visual = Option<Value<Option<kivact_interaction::ThumbDrag>>>;

/// A switch's opted-in drag on its control part, and the thumb it moves.
fn switch_gesture(
    control: Part<ToggleState>,
    value: Signal<bool>,
    size: Value<ControlSize>,
    on_edit: Option<Callback<EditPhase>>,
) -> (Part<ToggleState>, Visual) {
    let drag =
        kivact_interaction::use_switch_drag(value, move || switch_track::switch_track(size.get()));
    if !drag.is_enabled() {
        return (control, None);
    }
    if let Some(observer) = on_edit {
        drag.on_edit(move |phase| observer.call(phase));
    }
    (drag.control(control), Some(drag.thumb()))
}
