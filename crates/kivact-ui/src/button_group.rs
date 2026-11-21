//! Role: Group arbitrary action controls into one native roving focus scope.
pub(crate) mod look;
mod view;

pub(crate) use view::GroupPosition;
pub use view::{
    ButtonGroup, ButtonGroupCell, button_group, button_group_separator, button_group_text,
};
