//! Role: Expose file acceptance and picker lifecycle without rendering.
mod api;
mod machine;
pub use api::{DropzoneApi, use_dropzone};
pub use kivact_types::platform::FileInfo;
pub use machine::{DropzoneState, Validity};
