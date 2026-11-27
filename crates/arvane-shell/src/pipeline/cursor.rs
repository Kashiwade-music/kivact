//! Role: Map portable cursors to native resource identifiers and names.
use kivact_types::Cursor;
use std::ffi::CStr;
/// The Windows system cursor of `cursor`. Grab and Grabbing have no system
/// cursor on Windows; the ports draw them (`hand_cursor`) and do not ask here.
pub fn cursor_to_vk_cursor(cursor: Cursor) -> u16 {
    match cursor {
        Cursor::None => 0,
        Cursor::Text => 32513,
        Cursor::Pointer => 32649,
        Cursor::ColResize | Cursor::EwResize => 32644,
        Cursor::RowResize | Cursor::NsResize => 32645,
        Cursor::Crosshair => 32515,
        Cursor::Move => 32646,
        Cursor::NotAllowed => 32648,
        _ => 32512,
    }
}
pub fn cursor_to_x11_name(cursor: Cursor) -> &'static CStr {
    match cursor {
        Cursor::None => c"none",
        Cursor::Text => c"xterm",
        Cursor::Pointer => c"hand2",
        Cursor::ColResize | Cursor::EwResize => c"sb_h_double_arrow",
        Cursor::RowResize | Cursor::NsResize => c"sb_v_double_arrow",
        Cursor::Grab => c"hand1",
        Cursor::Grabbing | Cursor::Move => c"fleur",
        Cursor::Crosshair => c"crosshair",
        Cursor::NotAllowed => c"X_cursor",
        _ => c"left_ptr",
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the Windows cursor identifiers.
    use super::*;

    #[test]
    fn grab_has_no_system_cursor_and_falls_to_the_arrow() {
        assert_eq!(cursor_to_vk_cursor(Cursor::Grab), 32512);
    }
}
