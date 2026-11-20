//! Role: What an embedded App tells its host.

use super::EmbedStatus;

/// A push notice of an embedded App (`Parent::on_notice`). It runs on the
/// thread that observed the change; it must not wait or panic.
#[derive(Clone, Debug, PartialEq)]
pub enum EmbedNotice {
    /// The applied scale changed for a reason other than the host's
    /// `set_scale` (an operating-system scale change). Keep the logical
    /// size by asking for `logical size * scale`.
    ScaleChanged(f64),
    /// The App stopped, once: closed by the host, failed, or panicked.
    /// [`Embedded::status`](super::Embedded::status) keeps reporting it.
    Stopped(EmbedStatus),
}
