//! Role: Submit multiline drafts through keyboard and send/stop controls.

pub(crate) mod look;
mod view;

pub use look::ComposerStatus;
pub use view::{ChatComposer, ComposerStatusPosition, chat_composer};
