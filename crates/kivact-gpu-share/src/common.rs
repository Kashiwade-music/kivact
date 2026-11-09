//! Role: Adapter identities, contract mismatches and shared graphics API errors.

use kivact_types::GpuBackend;
#[cfg(feature = "import")]
use kivact_types::SharedTextures;
use std::fmt;

/// An adapter's identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Identity {
    /// The graphics API.
    pub backend: GpuBackend,
    // xtask-allow(state-fields): identity value - DXGI's (or Vulkan's) adapter LUID
    /// DXGI's (or Vulkan's) adapter LUID.
    pub luid: Option<[u8; 8]>,
    // xtask-allow(state-fields): identity value - Vulkan's `deviceUUID`
    /// Vulkan's `deviceUUID`.
    pub device_uuid: Option<[u8; 16]>,
    // xtask-allow(state-fields): identity value - Vulkan's `driverUUID`
    /// Vulkan's `driverUUID`.
    pub driver_uuid: Option<[u8; 16]>,
}

/// A sharing contract, graphics API, or bounded-wait failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A bounded wait ran out: the GPU did not reach what is named (for a
    /// release, often a window that lost its device or stopped
    /// presenting).
    Timeout(&'static str),
    /// No adapter of the graphics API is the one the window reported.
    NoAdapter,
    /// The arguments disagree with each other or with the contract
    /// (another adapter or graphics API, an empty size, a texture the
    /// window acquired twice or released without acquiring it, a texture
    /// of another format).
    Mismatch(String),
    /// A call of the graphics API failed.
    Api {
        /// The function called.
        call: &'static str,
        /// The API's error.
        message: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timeout(what) => write!(f, "timed out waiting for {what}"),
            Self::NoAdapter => f.write_str("no adapter is the one the window reported"),
            Self::Mismatch(what) => write!(f, "mismatch: {what}"),
            Self::Api { call, message } => write!(f, "{call} failed: {message}"),
        }
    }
}

impl std::error::Error for Error {}

/// `call`'s failure as an [`Error::Api`].
#[cfg(any(
    feature = "import",
    all(feature = "export", any(windows, target_os = "linux"))
))]
pub(crate) fn api<E: fmt::Display>(call: &'static str) -> impl Fn(E) -> Error {
    move |e| Error::Api {
        call,
        message: e.to_string(),
    }
}

impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::Mismatch(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Mismatch(message.into())
    }
}

#[cfg(feature = "import")]
/// The backend a device runs on, named for errors.
fn backend_name(backend: GpuBackend) -> &'static str {
    match backend {
        GpuBackend::Dx12 => "Direct3D 12",
        GpuBackend::Vulkan => "Vulkan",
    }
}

/// Fail unless `shared` was made with `backend`.
#[cfg_attr(
    not(any(windows, target_os = "linux")),
    expect(dead_code, reason = "no backend imports here")
)]
#[cfg(feature = "import")]
pub(crate) fn same_backend(shared: &SharedTextures, backend: GpuBackend) -> Result<(), Error> {
    if shared.backend == backend {
        return Ok(());
    }
    Err(Error::Mismatch(format!(
        "{} textures cannot be imported into a {} device",
        backend_name(shared.backend),
        backend_name(backend)
    )))
}
