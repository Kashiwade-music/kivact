//! Role: Identify the active presentation of a selection surface.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Surface {
    Popover,
    Sheet,
}
