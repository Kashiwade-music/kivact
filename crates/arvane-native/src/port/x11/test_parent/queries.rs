//! Role: Queries of the embedded child's visibility and keyboard focus.

use super::TestParent;
use arvane_shell::ShellError;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt;

impl TestParent {
    /// Whether the embedded editor's child window is visible.
    pub fn editor_visible(&self) -> Result<bool, ShellError> {
        let child = self.child()?;
        let attributes = self
            .native
            .connection
            .get_window_attributes(child)
            .map_err(super::super::error::platform)?
            .reply()
            .map_err(super::super::error::platform)?;
        Ok(attributes.map_state == xproto::MapState::VIEWABLE)
    }

    /// Whether the embedded editor has the keyboard focus.
    pub fn editor_focused(&self) -> Result<bool, ShellError> {
        let child = self.child()?;
        let focus = self
            .native
            .connection
            .get_input_focus()
            .map_err(super::super::error::platform)?
            .reply()
            .map_err(super::super::error::platform)?
            .focus;
        Ok(focus == child)
    }
}
