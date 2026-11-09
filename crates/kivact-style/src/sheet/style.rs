//! Role: The `style()` entry point and the group access macro.

use super::fields::Style;

/// The group of `style` named, or the group's defaults when it is unset:
/// `let size = group_or_default!(style.size, SizeFields);`.
macro_rules! group_or_default {
    ($group:expr, $ty:ident) => {
        match $group.as_deref() {
            Some(group) => std::borrow::Cow::Borrowed(group),
            None => std::borrow::Cow::Owned($ty::default()),
        }
    };
}

pub(crate) use group_or_default;

/// Start a style chain.
pub fn style() -> Style {
    Style::default()
}
