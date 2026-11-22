//! Role: Build the themed button from a momentary toggle part.
//!
//! The root, the content row, the icon boxes and the busy indicator all get
//! one [`look::ButtonState`]. Its `expanded` is read from the caller's
//! modifiers (`Caller::expanded`): `Some(open)` when a popup, menu or
//! disclosure trigger part was attached or the caller set `.expanded`,
//! `None` otherwise. The `.icon` and `.trailing` views go
//! into `button_icon` boxes, where a kit sizes the icon by the button's size.
//! While busy, the busy part is timed by the theme variables `theme::button::BUSY_*_VAR`.
//! A [`ButtonVariant`] is only a meaning,
//! given with `.variant(..)` like a kit's tone or slanted shape and read by
//! the recipe with `look.variant::<ButtonVariant>()` (a button without one is
//! the default, `Primary`). A kit drawing its own part as a button calls the
//! recipe with
//! `Look::new(ButtonState::new(input, size)).with_variant(variant)`.
//! The caller's view in `.icon`, `.trailing` and `.child` is composed
//! in, without reimplementing activation.
pub(crate) mod look;
pub use look::ButtonVariant;
mod view;
pub use view::{Button, IconButton, button, icon_button};
