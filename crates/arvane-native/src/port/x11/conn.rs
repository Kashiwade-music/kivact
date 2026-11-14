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

pub(super) struct ConnectionAlive(AtomicBool);
impl ConnectionAlive {
    pub(super) fn is_alive(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    pub(super) fn end(&self) -> bool {
        self.0.swap(false, Ordering::AcqRel)
    }
}

pub(super) struct Native {
    pub(super) connection: Arc<XCBConnection>,
    pub(super) id: NonZeroU32,
    pub(super) xembed: u32,
    pub(super) xembed_info: u32,
    /// The host window the child was created in (synthetic keys go here).
    pub(super) parent: u32,
    /// The X server still has this window (no DestroyNotify or BadWindow for it yet).
    pub(super) exists: ConnectionAlive,
    pub(super) screen: i32,
    visual: NonZeroU32,
    owner: ThreadId,
}

impl Native {
    pub(super) fn show(&self) -> Result<(), ShellError> {
        self.set_mapped_flag(true)?;
        self.connection
            .map_window(self.id.get())
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        self.connection.flush().map_err(super::error::platform)
    }

    /// Unmap the child and tell XEmbed toolkits it is unmapped.
    pub(super) fn hide(&self) -> Result<(), ShellError> {
        self.connection
            .unmap_window(self.id.get())
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        self.set_mapped_flag(false)?;
        self.connection.flush().map_err(super::error::platform)
    }

    /// Write the XEMBED_MAPPED flag and settle the request.
    pub(super) fn set_mapped_flag(&self, mapped: bool) -> Result<(), ShellError> {
        self.connection
            .change_property32(
                PropMode::REPLACE,
                self.id.get(),
                self.xembed_info,
                self.xembed_info,
                &[0, u32::from(mapped)],
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)
    }

    /// Settles a checked request on our own window. A destroyed window means
    /// the host destroyed the parent first: mark it dead so the loop ends
    /// cleanly instead of reporting a failure.
    pub(super) fn settle(&self, result: Result<(), ReplyError>) -> Result<(), ShellError> {
        match result {
            Err(error) if super::error::destroyed(&error, self.id.get()) => {
                self.exists.end();
                Ok(())
            }
            other => other.map_err(super::error::platform),
        }
    }

    pub(super) fn create_unmapped(
        parent: u32,
        logical_width: f64,
        logical_height: f64,
        requested_scale: Option<f64>,
        visibility: kivact_types::embed::Visibility,
    ) -> Result<(Arc<Self>, u32, u32, f64), ShellError> {
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
        let (width, height) = crate::port::embed::geometry::physical_size(
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
            xembed_info: info,
            parent,
            screen: screen as i32,
            visual,
            owner: std::thread::current().id(),
            exists: ConnectionAlive(AtomicBool::new(true)),
        });
        native.create_child(parent, width, height, info, visibility)?;
        Ok((native, width.into(), height.into(), scale))
    }
    /// Install the event mask and XEmbed identity on the new child.
    fn create_child(
        &self,
        parent: u32,
        width: u16,
        height: u16,
        info: u32,
        visibility: kivact_types::embed::Visibility,
    ) -> Result<(), ShellError> {
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
        self.connection
            .create_window(
                x11rb::COPY_DEPTH_FROM_PARENT,
                self.id.get(),
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
        let mapped = u32::from(visibility == kivact_types::embed::Visibility::Shown);
        self.connection
            .change_property32(PropMode::REPLACE, self.id.get(), info, info, &[0, mapped])
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        Ok(())
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

fn resource_scale(connection: &XCBConnection, root: u32) -> Option<f64> {
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
        let scale = value.trim().parse::<f64>().ok()? / 96.0;
        (scale.is_finite() && scale > 0.0).then_some(scale)
    })
}

impl HasWindowHandle for Native {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        if std::thread::current().id() != self.owner || !self.exists.is_alive() {
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
        if std::thread::current().id() != self.owner || !self.exists.is_alive() {
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
        if self.exists.end() {
            // A completed external drop must acknowledge destruction on the
            // server, before another connection can still observe this child.
            if let Ok(destroyed) = self.connection.destroy_window(self.id.get()) {
                let _ = destroyed.check();
            }
        }
        let _ = self.connection.flush();
    }
}
