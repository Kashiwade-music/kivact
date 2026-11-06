//! Role: Normalize platform input with portable shared rules.
mod cursor;
mod dnd;
mod ime;
mod keys;
mod raw;
pub use cursor::{cursor_to_vk_cursor, cursor_to_x11_name};
pub use keys::{KeyPipeline, code_from_x11_keycode, key_from_keysym, key_from_vk};
pub use raw::{PlatformKey, RawDrop, RawIme, RawInput, RawKey};

pub use dnd::DropSession;
pub use ime::{ImeDriver, ImePort};
