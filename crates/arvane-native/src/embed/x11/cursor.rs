//! Role: Installs core X11 font cursors without a process-global cursor library.

use super::conn::Native;
use arvane_shell::ShellError;
use kivact_types::Cursor;
use std::{cell::RefCell, collections::HashMap};
use x11rb::{
    connection::Connection,
    protocol::xproto::{ChangeWindowAttributesAux, ConnectionExt, CreateGCAux, Rectangle},
};

#[derive(Default)]
pub(super) struct Cursors {
    cache: RefCell<HashMap<Cursor, u32>>,
}

impl Cursors {
    pub(super) fn set(&self, native: &Native, cursor: Cursor) -> Result<(), ShellError> {
        let mut cache = self.cache.borrow_mut();
        let id = if let Some(id) = cache.get(&cursor) {
            *id
        } else {
            let id = create(native, cursor)?;
            cache.insert(cursor, id);
            id
        };
        let result = native
            .connection
            .change_window_attributes(
                native.id.get(),
                &ChangeWindowAttributesAux::new().cursor(id),
            )
            .map_err(super::error::platform)?
            .check();
        native.settle(result)?;
        native.connection.flush().map_err(super::error::platform)
    }
    pub(super) fn release(&self, native: &Native) {
        for (_, id) in self.cache.borrow_mut().drain() {
            let _ = native.connection.free_cursor(id);
        }
    }
}

fn create(native: &Native, cursor: Cursor) -> Result<u32, ShellError> {
    let connection = &*native.connection;
    let id = connection.generate_id().map_err(super::error::platform)?;
    if cursor == Cursor::None {
        let pixmap = connection.generate_id().map_err(super::error::platform)?;
        let gc = connection.generate_id().map_err(super::error::platform)?;
        connection
            .create_pixmap(1, pixmap, native.id.get(), 1, 1)
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        let result = (|| {
            connection
                .create_gc(gc, pixmap, &CreateGCAux::new().foreground(0))
                .map_err(super::error::platform)?
                .check()
                .map_err(super::error::platform)?;
            connection
                .poly_fill_rectangle(
                    pixmap,
                    gc,
                    &[Rectangle {
                        x: 0,
                        y: 0,
                        width: 1,
                        height: 1,
                    }],
                )
                .map_err(super::error::platform)?
                .check()
                .map_err(super::error::platform)?;
            connection
                .create_cursor(id, pixmap, pixmap, 0, 0, 0, 0, 0, 0, 0, 0)
                .map_err(super::error::platform)?
                .check()
                .map_err(super::error::platform)
        })();
        let _ = connection.free_gc(gc);
        let _ = connection.free_pixmap(pixmap);
        result?;
    } else {
        let font = connection.generate_id().map_err(super::error::platform)?;
        connection
            .open_font(font, b"cursor")
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        let glyph = match cursor {
            Cursor::Auto | Cursor::Default => 68,
            Cursor::Text => 152,
            Cursor::Pointer => 60,
            Cursor::ColResize | Cursor::EwResize => 108,
            Cursor::RowResize | Cursor::NsResize => 116,
            Cursor::Grab => 58,
            Cursor::Grabbing => 52,
            Cursor::Crosshair => 34,
            Cursor::Move => 52,
            Cursor::NotAllowed => 0,
            Cursor::None => unreachable!(),
        };
        let result = connection
            .create_glyph_cursor(
                id,
                font,
                font,
                glyph,
                glyph + 1,
                0,
                0,
                0,
                u16::MAX,
                u16::MAX,
                u16::MAX,
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform);
        let _ = connection.close_font(font);
        result?;
    }
    Ok(id)
}
