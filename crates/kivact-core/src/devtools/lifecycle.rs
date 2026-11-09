//! Role: Write the DevTools recording settings to the runtime and the engine.

use kivact_runtime::Runtime;

use crate::view::devtools_state::DevtoolsState;
use crate::view::state::ViewState;

/// Write the recording settings to the runtime and the engine.
pub(crate) fn apply_recording(rt: &Runtime) {
    let i = DevtoolsState::inspector(rt);
    let recording = i.hooks.recording.borrow().clone();
    let overlays = i.session.overlays.borrow().clone();
    rt.set_detailed_trace(recording.updates);
    ViewState::tree(rt).engine().set_engine_recording(
        recording.updates || overlays.paint_flashing,
        recording.updates || recording.frames,
    );
}
