//! Role: Canonical node semantics, relation mapping and accessible value choices.

mod finite;
mod node;
mod validation;
mod values;

pub use finite::Finite;
pub use node::Semantics;
pub use validation::check_text;
pub use values::{
    A11yAction, A11yRequest, CheckedState, CurrentItem, DismissReason, LiveRegion, Orientation,
    PopupKind, Role, SortDirection,
};
