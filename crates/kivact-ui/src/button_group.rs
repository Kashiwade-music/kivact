//! Role: Group arbitrary action controls into one native roving focus scope.
pub(crate) mod look;
pub(crate) mod tokens;
mod view;

pub use view::{
    ButtonGroup, ButtonGroupCell, ButtonGroupMember, button_group, button_group_separator,
    button_group_text,
};
