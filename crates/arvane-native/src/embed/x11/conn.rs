//! Role: Owns the independent XCB connection and retained native window target.
//!
//! Invariants: raw handles are exposed only on the creating UI thread. The
//! connection and window outlive every retained CPU or GPU presentation target.
#![allow(
    unsafe_code,
    reason = "Borrowed raw handles describe our owned XCB window"
)]

use arvane_shell::ShellError;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle, XcbDisplayHandle, XcbWindowHandle,
};
use std::num::NonZeroU32;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::ThreadId;
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt, CreateWindowAux, EventMask, PropMode, WindowClass,
};
use x11rb::wrapper::ConnectionExt as _;
use x11rb::xcb_ffi::XCBConnection;

pub(super) struct Native {
    pub(super) connection: Arc<XCBConnection>,
    pub(super) id: NonZeroU32,
    pub(super) xembed: u32,
    /// The X server still has this window (no DestroyNotify or BadWindow for it yet).
    pub(super) exists: AtomicBool,
    pub(super) screen: i32,
    visual: NonZeroU32,
    owner: ThreadId,
}

impl Native {
    pub(super) fn show(&self) -> Result<(), ShellError> {
        self.connection
            .map_window(self.id.get())
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        self.connection.flush().map_err(super::error::platform)
    }

    /// Settles a checked request on our own window. A destroyed window means
    /// the host destroyed the parent first: mark it dead so the loop ends
    /// cleanly instead of reporting a failure.
    pub(super) fn settle(&self, result: Result<(), ReplyError>) -> Result<(), ShellError> {
        match result {
            Err(error) if super::error::destroyed(&error, self.id.get()) => {
                self.exists.store(false, Ordering::Release);
                Ok(())
            }
            other => other.map_err(super::error::platform),
        }
    }

    pub(super) fn create_unmapped(
        parent: u32,
        logical_width: f32,
        logical_height: f32,
        requested_scale: Option<f32>,
    ) -> Result<(Arc<Self>, u32, u32, f32), ShellError> {
        let (connection, _) = XCBConnection::connect(None).map_err(super::error::platform)?;
        let connection = Arc::new(connection);
        // A checked request reports stale or non-window host handles before start.
        let attributes = connection
            .get_window_attributes(parent)
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?;
        let geometry = connection
            .get_geometry(parent)
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?;
        let root = geometry.root;
        let screen = connection
            .setup()
            .roots
            .iter()
            .position(|screen| screen.root == root)
            .ok_or_else(|| super::error::platform("parent belongs to an unknown screen"))?;
        let scale = requested_scale
            .or_else(|| resource_scale(&connection, root))
            .unwrap_or(1.0);
        if !scale.is_finite() || scale <= 0.0 {
            return Err(super::error::platform("invalid scale"));
        }
        let (width, height) = super::super::geometry::physical_size(
            kivact_types::Size {
                width: logical_width,
                height: logical_height,
            },
            scale,
            super::notifier::MAX_DIMENSION,
        )?;
        let (width, height) = (width as u16, height as u16);
        let id = connection.generate_id().map_err(super::error::platform)?;
        let id = NonZeroU32::new(id).ok_or_else(|| super::error::platform("zero window id"))?;
        let xembed = atom(&connection, b"_XEMBED")?;
        let info = atom(&connection, b"_XEMBED_INFO")?;
        let visual = NonZeroU32::new(attributes.visual)
            .ok_or_else(|| super::error::platform("parent has no visual"))?;
        let native = Arc::new(Self {
            connection,
            id,
            xembed,
            screen: screen as i32,
            visual,
            owner: std::thread::current().id(),
            exists: AtomicBool::new(true),
        });
        let mask = EventMask::EXPOSURE
            | EventMask::STRUCTURE_NOTIFY
            | EventMask::POINTER_MOTION
            | EventMask::BUTTON_PRESS
            | EventMask::BUTTON_RELEASE
            | EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE
            | EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW
            | EventMask::FOCUS_CHANGE
            | EventMask::PROPERTY_CHANGE;
        native
            .connection
            .create_window(
                x11rb::COPY_DEPTH_FROM_PARENT,
                id.get(),
                parent,
                0,
                0,
                width,
                height,
                0,
                WindowClass::INPUT_OUTPUT,
                x11rb::COPY_FROM_PARENT,
                &CreateWindowAux::new().event_mask(mask),
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        native
            .connection
            .change_property32(PropMode::REPLACE, id.get(), info, info, &[0, 1])
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        Ok((native, width.into(), height.into(), scale))
    }
}

fn atom(connection: &XCBConnection, name: &[u8]) -> Result<u32, ShellError> {
    Ok(connection
        .intern_atom(false, name)
        .map_err(super::error::platform)?
        .reply()
        .map_err(super::error::platform)?
        .atom)
}

fn resource_scale(connection: &XCBConnection, root: u32) -> Option<f32> {
    let property = atom(connection, b"RESOURCE_MANAGER").ok()?;
    let reply = connection
        .get_property(false, root, property, AtomEnum::STRING, 0, 1_048_576)
        .ok()?
        .reply()
        .ok()?;
    let resources = std::str::from_utf8(&reply.value).ok()?;
    resources.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.trim() != "Xft.dpi" {
            return None;
        }
        let scale = value.trim().parse::<f32>().ok()? / 96.0;
        (scale.is_finite() && scale > 0.0).then_some(scale)
    })
}

impl HasWindowHandle for Native {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        if std::thread::current().id() != self.owner || !self.exists.load(Ordering::Acquire) {
            return Err(HandleError::Unavailable);
        }
        let mut handle = XcbWindowHandle::new(self.id);
        handle.visual_id = Some(self.visual);
        // SAFETY: this borrow retains the live owned window and matching visual.
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xcb(handle)) })
    }
}

impl HasDisplayHandle for Native {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        if std::thread::current().id() != self.owner || !self.exists.load(Ordering::Acquire) {
            return Err(HandleError::Unavailable);
        }
        let connection = NonNull::new(self.connection.get_raw_xcb_connection());
        let handle = XcbDisplayHandle::new(connection, self.screen);
        // SAFETY: the Arc retains our live XCB connection for this borrow.
        Ok(unsafe { DisplayHandle::borrow_raw(RawDisplayHandle::Xcb(handle)) })
    }
}

impl Drop for Native {
    fn drop(&mut self) {
        if self.exists.swap(false, Ordering::AcqRel) {
            // A completed external drop must acknowledge destruction on the
            // server, before another connection can still observe this child.
            if let Ok(destroyed) = self.connection.destroy_window(self.id.get()) {
                let _ = destroyed.check();
            }
        }
        let _ = self.connection.flush();
    }
}
