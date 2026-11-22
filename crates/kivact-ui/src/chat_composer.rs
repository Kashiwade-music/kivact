//! Role: Submit multiline drafts through keyboard and send/stop controls.
pub(crate) mod look;
pub use look::ComposerStatus;
mod view;
pub use view::{ChatComposer, ComposerStatusPosition, chat_composer};
