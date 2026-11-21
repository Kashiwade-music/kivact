//! Role: Render themed dialog frames while shared parts own their lifetime.
//!
//! The builder is itself the view: the body comes from `.content(factory)`
//! and the actions row from `.footer(factory)`, and the same renderer draws
//! Modal, AlertModal, Drawer and Popover from `use_dialog`'s parts with
//! [`DialogState`] (`input`, `open`, `kind`, `media`). A popover gets no
//! header close target. The header is a column of the optional media view,
//! the title and the description (its recipe may lay them out as a grid,
//! knowing [`DialogState::media`]); a hidden title leaves the heading out,
//! and a header with nothing left to show is left out. The close button is
//! the panel's last child (its recipe places it, usually at the top right),
//! so the initial focus goes to the content's first control and the close
//! button is the last tab stop. A dialog without `.content(..)` has no
//! body. When only the footer has a close,
//! `closable(false)` leaves the header's target out without changing the
//! dismissal rules. A drawer's body scrolls and its panel is fixed to its
//! edge. Entering and leaving are the `enter` and `exit` styles the surface
//! recipe returns.
mod drawer_swipe;
pub(crate) mod look;
mod view;

pub use view::{Dialog, DialogKind, dialog};
