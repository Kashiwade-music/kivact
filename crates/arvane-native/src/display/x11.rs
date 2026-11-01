//! Role: Read the X11 desktop work area without backend types escaping the shell.

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt};

/// The monitor's usable area as `(x, y, width, height)` in physical pixels.
pub(super) fn work_area(bounds: (i32, i32, u32, u32)) -> Option<(i32, i32, u32, u32)> {
    let (connection, screen) = x11rb::connect(None).ok()?;
    let root = connection.setup().roots.get(screen)?.root;
    let atom = |name: &[u8]| {
        connection
            .intern_atom(true, name)
            .ok()?
            .reply()
            .ok()
            .map(|r| r.atom)
    };
    let desktop = connection
        .get_property(
            false,
            root,
            atom(b"_NET_CURRENT_DESKTOP")?,
            AtomEnum::CARDINAL,
            0,
            1,
        )
        .ok()?
        .reply()
        .ok()?;
    let desktop = desktop.value32()?.next()?;
    let area = connection
        .get_property(
            false,
            root,
            atom(b"_NET_WORKAREA")?,
            AtomEnum::CARDINAL,
            desktop.checked_mul(4)?,
            4,
        )
        .ok()?
        .reply()
        .ok()?;
    let mut values = area.value32()?;
    let (x, y, width, height) = (
        i64::from(values.next()? as i32),
        i64::from(values.next()? as i32),
        i64::from(values.next()?),
        i64::from(values.next()?),
    );
    let left = x.max(i64::from(bounds.0));
    let top = y.max(i64::from(bounds.1));
    let right = (x + width).min(i64::from(bounds.0) + i64::from(bounds.2));
    let bottom = (y + height).min(i64::from(bounds.1) + i64::from(bounds.3));
    (right > left && bottom > top).then_some((
        left as i32,
        top as i32,
        (right - left) as u32,
        (bottom - top) as u32,
    ))
}
