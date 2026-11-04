//! Role: Start the engine's native window loop; report its errors as diagnostics.

use kivact_engine::api::Client;
use kivact_reactive::{ErrorKind, UiError};

/// Run the client in a window of the engine. Its errors become Kivact
/// platform diagnostics.
pub(crate) fn run_native(
    client: Client,
    options: kivact_engine::api::HostOptions,
) -> crate::Result<()> {
    kivact_engine::run_native(client, options)
        .map_err(|e| UiError::new(ErrorKind::Platform, e.message, None))
}
