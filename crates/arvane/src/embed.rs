//! Role: Embedded window creation with the shared rendering driver.

use crate::Handler;
use crate::presentation::Observer;
use arvane_shell::MakeHandler;
pub use arvane_shell::{ChildStop, EmbedNotice, PanicSlot};

pub use arvane_shell::ChildHooks;
pub use kivact_types::EmbedOptions;

/// The driver of an embedded window, made on the window's UI thread:
/// `make` builds the handler and the optional renderer observer there.
pub fn embed_driver<H: Handler + 'static>(
    gpu: kivact_types::GpuPreference,
    make: impl FnOnce() -> (H, Option<Observer>) + Send + 'static,
) -> MakeHandler {
    Box::new(move || {
        let (handler, observer) = make();
        Box::new(crate::window::driver(
            handler,
            crate::window::DriverOptions { gpu, observer },
        )) as Box<dyn arvane_shell::ShellHandler>
    })
}
