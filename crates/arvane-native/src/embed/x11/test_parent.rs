//! Role: Provides an independent X11 parent and synthetic child input for E2E.

use super::conn::Native;
use arvane_shell::ShellError;
use kivact_types::{CpuFrame, KeyCode, MouseButton, ParentWindow};
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::properties::{WmHints, WmHintsState, WmSizeHints, WmSizeHintsSpecification};
use x11rb::protocol::ErrorKind;
use x11rb::protocol::xproto::{
    self, ButtonPressEvent, ConnectionExt, EventMask, ImageFormat, ImageOrder, KeyButMask,
    KeyPressEvent, MotionNotifyEvent,
};
use x11rb::wrapper::ConnectionExt as _;

/// A top-level X11 host on a separate connection from its embedded child.
pub struct TestParent {
    point: Cell<(f32, f32)>,
    covers: RefCell<Vec<u32>>,
    wm_frame: Cell<Option<u32>>,
    native: Arc<Native>,
    root: u32,
    protocols: u32,
    delete: u32,
}

impl TestParent {
    /// Open a visible host with a physical client extent.
    pub fn open(width: u32, height: u32) -> Result<Self, ShellError> {
        let (connection, screen) =
            x11rb::xcb_ffi::XCBConnection::connect(None).map_err(super::error::platform)?;
        let root = connection.setup().roots[screen].root;
        let (native, _, _, _) =
            Native::create_unmapped(root, width as f32, height as f32, Some(1.0))?;
        let protocols = native
            .connection
            .intern_atom(false, b"WM_PROTOCOLS")
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?
            .atom;
        let delete = native
            .connection
            .intern_atom(false, b"WM_DELETE_WINDOW")
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?
            .atom;
        native
            .connection
            .change_property32(
                xproto::PropMode::REPLACE,
                native.id.get(),
                protocols,
                xproto::AtomEnum::ATOM,
                &[delete],
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        map_host(&native, width, height)?;
        let parent = Self {
            point: Cell::new((0.0, 0.0)),
            covers: RefCell::new(Vec::new()),
            wm_frame: Cell::new(None),
            native,
            root,
            protocols,
            delete,
        };
        // A checked MapWindow confirms the request, not the WM's asynchronous
        // frame mapping. Do not create embedded children inside an unviewable host.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let attributes = parent
                .native
                .connection
                .get_window_attributes(parent.native.id.get())
                .map_err(super::error::platform)?
                .reply()
                .map_err(super::error::platform)?;
            if attributes.map_state == xproto::MapState::VIEWABLE {
                let tree = parent
                    .native
                    .connection
                    .query_tree(parent.native.id.get())
                    .map_err(super::error::platform)?
                    .reply()
                    .map_err(super::error::platform)?;
                parent
                    .wm_frame
                    .set((tree.parent != root && tree.parent != 0).then_some(tree.parent));
                return Ok(parent);
            }
            if Instant::now() >= deadline {
                return Err(super::error::platform(
                    "test host did not become viewable after MapWindow",
                ));
            }
            parent.pump(Duration::from_millis(5));
        }
    }
    /// Press the requested button at physical child coordinates.
    pub fn press_button(&self, button: MouseButton, x: f32, y: f32) -> Result<(), ShellError> {
        self.point.set((x, y));
        self.send_button(button, x, y, true)
    }
    /// Release the requested button at physical child coordinates.
    pub fn release_button(&self, button: MouseButton, x: f32, y: f32) -> Result<(), ShellError> {
        self.point.set((x, y));
        self.send_button(button, x, y, false)
    }
    /// Place a solid host-owned sibling above the editor.
    pub fn cover(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        rgb: [u8; 3],
    ) -> Result<(), ShellError> {
        let ParentWindow::X11(id) = self.parent_in(x, y, width, height)? else {
            unreachable!()
        };
        self.covers.borrow_mut().push(id.get());
        let connection = &self.native.connection;
        let attributes = connection
            .get_window_attributes(id.get())
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?;
        let visual = connection
            .setup()
            .roots
            .iter()
            .flat_map(|r| &r.allowed_depths)
            .flat_map(|d| &d.visuals)
            .find(|v| v.visual_id == attributes.visual)
            .ok_or_else(|| super::error::platform("cover visual missing"))?;
        let component = |value: u8, mask: u32| {
            if mask == 0 {
                0
            } else {
                ((u32::from(value) * (mask >> mask.trailing_zeros()) / 255)
                    << mask.trailing_zeros())
                    & mask
            }
        };
        let pixel = component(rgb[0], visual.red_mask)
            | component(rgb[1], visual.green_mask)
            | component(rgb[2], visual.blue_mask);
        connection
            .change_window_attributes(
                id.get(),
                &xproto::ChangeWindowAttributesAux::new().background_pixel(pixel),
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        connection
            .clear_area(false, id.get(), 0, 0, 0, 0)
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        connection
            .configure_window(
                id.get(),
                &xproto::ConfigureWindowAux::new().stack_mode(xproto::StackMode::ABOVE),
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        connection.flush().map_err(super::error::platform)
    }
    /// Send an XDND file selection and wait for the receiver's completion.
    pub fn drop_files(&self, files: &[PathBuf], x: f32, y: f32) -> Result<(), ShellError> {
        use std::os::unix::ffi::OsStrExt;
        let connection = &self.native.connection;
        let child = self.child()?;
        let atom = |name: &[u8]| -> Result<u32, ShellError> {
            Ok(connection
                .intern_atom(false, name)
                .map_err(super::error::platform)?
                .reply()
                .map_err(super::error::platform)?
                .atom)
        };
        let selection = atom(b"XdndSelection")?;
        let uri = atom(b"text/uri-list")?;
        let status = atom(b"XdndStatus")?;
        let finished = atom(b"XdndFinished")?;
        let source = self.native.id.get();
        connection
            .set_selection_owner(source, selection, x11rb::CURRENT_TIME)
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        let send = |kind: u32, data: [u32; 5]| -> Result<(), ShellError> {
            connection
                .send_event(
                    false,
                    child,
                    EventMask::NO_EVENT,
                    xproto::ClientMessageEvent::new(32, child, kind, data),
                )
                .map_err(super::error::platform)?
                .check()
                .map_err(super::error::platform)?;
            connection.flush().map_err(super::error::platform)
        };
        let origin = connection
            .translate_coordinates(child, self.root, x as i16, y as i16)
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?;
        send(atom(b"XdndEnter")?, [source, 5 << 24, uri, 0, 0])?;
        send(
            atom(b"XdndPosition")?,
            [
                source,
                0,
                (u32::from(origin.dst_x as u16) << 16) | u32::from(origin.dst_y as u16),
                x11rb::CURRENT_TIME,
                atom(b"XdndActionCopy")?,
            ],
        )?;
        let mut payload = Vec::new();
        for file in files {
            if !file.is_absolute() {
                return Err(super::error::platform("drop paths must be absolute"));
            }
            payload.extend_from_slice(b"file://");
            for &b in file.as_os_str().as_bytes() {
                if b.is_ascii_alphanumeric() || b"/-._~".contains(&b) {
                    payload.push(b);
                } else {
                    payload.extend_from_slice(format!("%{b:02X}").as_bytes());
                }
            }
            payload.extend_from_slice(b"\r\n");
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match connection
                .poll_for_event()
                .map_err(super::error::platform)?
            {
                Some(x11rb::protocol::Event::ClientMessage(e)) if e.type_ == status => {
                    if e.data.as_data32()[1] & 1 == 0 {
                        return Err(super::error::platform("XDND receiver refused files"));
                    }
                    send(atom(b"XdndDrop")?, [source, 0, x11rb::CURRENT_TIME, 0, 0])?;
                }
                Some(x11rb::protocol::Event::SelectionRequest(e)) if e.selection == selection => {
                    let property = if e.property == 0 {
                        e.target
                    } else {
                        e.property
                    };
                    connection
                        .change_property8(
                            xproto::PropMode::REPLACE,
                            e.requestor,
                            property,
                            uri,
                            &payload,
                        )
                        .map_err(super::error::platform)?
                        .check()
                        .map_err(super::error::platform)?;
                    connection
                        .send_event(
                            false,
                            e.requestor,
                            EventMask::NO_EVENT,
                            xproto::SelectionNotifyEvent {
                                response_type: xproto::SELECTION_NOTIFY_EVENT,
                                sequence: 0,
                                time: e.time,
                                requestor: e.requestor,
                                selection,
                                target: e.target,
                                property,
                            },
                        )
                        .map_err(super::error::platform)?
                        .check()
                        .map_err(super::error::platform)?;
                    connection.flush().map_err(super::error::platform)?;
                }
                Some(x11rb::protocol::Event::ClientMessage(e)) if e.type_ == finished => {
                    return if e.data.as_data32()[1] & 1 != 0 {
                        Ok(())
                    } else {
                        Err(super::error::platform("XDND transfer failed"))
                    };
                }
                _ => {}
            }
            if Instant::now() >= deadline {
                return Err(super::error::platform("XDND receiver timed out"));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    /// Create a host-owned child container using physical coordinates.
    pub fn parent_in(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<ParentWindow, ShellError> {
        if x > i16::MAX as u32
            || y > i16::MAX as u32
            || width == 0
            || height == 0
            || width > u16::MAX as u32
            || height > u16::MAX as u32
        {
            return Err(super::error::platform(
                "invalid test parent container coordinates",
            ));
        }
        let connection = &*self.native.connection;
        let id = connection.generate_id().map_err(super::error::platform)?;
        connection
            .create_window(
                x11rb::COPY_DEPTH_FROM_PARENT,
                id,
                self.native.id.get(),
                x as i16,
                y as i16,
                width as u16,
                height as u16,
                0,
                xproto::WindowClass::INPUT_OUTPUT,
                x11rb::COPY_FROM_PARENT,
                &xproto::CreateWindowAux::new(),
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        connection
            .map_window(id)
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        connection.flush().map_err(super::error::platform)?;
        Ok(ParentWindow::X11(
            std::num::NonZeroU32::new(id).expect("generated X11 id is nonzero"),
        ))
    }
    /// Borrow this host's portable parent identity while it remains open.
    pub fn parent(&self) -> ParentWindow {
        ParentWindow::X11(self.native.id)
    }
    /// X11 has no Windows HWND.
    pub fn win32_hwnd(&self) -> Option<*mut std::ffi::c_void> {
        None
    }
    /// Raw X11 host window identity for platform-specific checks.
    pub fn x11_window(&self) -> Option<u32> {
        Some(self.native.id.get())
    }
    /// Give the independent child UI thread time to process input and redraw.
    pub fn pump(&self, duration: Duration) {
        let started = Instant::now();
        while started.elapsed() < duration {
            loop {
                match self.native.connection.poll_for_event() {
                    Ok(Some(x11rb::protocol::Event::ClientMessage(event)))
                        if event.type_ == self.protocols
                            && event.data.as_data32()[0] == self.delete =>
                    {
                        self.close()
                    }
                    Ok(Some(x11rb::protocol::Event::DestroyNotify(event)))
                        if event.window == self.native.id.get() =>
                    {
                        self.native.alive.store(false, Ordering::Release);
                    }
                    Ok(Some(x11rb::protocol::Event::Expose(event)))
                        if event.window == self.native.id.get() =>
                    {
                        if let Ok(cookie) = self.native.connection.clear_area(
                            false,
                            self.native.id.get(),
                            0,
                            0,
                            0,
                            0,
                        ) {
                            let _ = cookie.check();
                        }
                    }
                    Ok(Some(_)) => {}
                    Ok(None) => break,
                    Err(_) => {
                        self.native.alive.store(false, Ordering::Release);
                        break;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    /// Click the primary button at physical child coordinates.
    pub fn click(&self, x: f32, y: f32) -> Result<(), ShellError> {
        self.press(x, y)?;
        self.release(x, y)
    }
    /// Press the primary button at physical child coordinates.
    pub fn press(&self, x: f32, y: f32) -> Result<(), ShellError> {
        self.point.set((x, y));
        self.button(x, y, true)
    }
    /// Move the child mouse to physical coordinates.
    pub fn move_to(&self, x: f32, y: f32) -> Result<(), ShellError> {
        let child = self.child()?;
        self.point.set((x, y));
        let event = MotionNotifyEvent {
            response_type: xproto::MOTION_NOTIFY_EVENT,
            detail: xproto::Motion::NORMAL,
            sequence: 0,
            time: x11rb::CURRENT_TIME,
            root: self.root,
            event: child,
            child: 0,
            root_x: x as i16,
            root_y: y as i16,
            event_x: x as i16,
            event_y: y as i16,
            state: KeyButMask::default(),
            same_screen: true,
        };
        self.native
            .connection
            .send_event(false, child, EventMask::POINTER_MOTION, event)
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)
    }
    /// Release the primary button at physical child coordinates.
    pub fn release(&self, x: f32, y: f32) -> Result<(), ShellError> {
        self.point.set((x, y));
        self.button(x, y, false)
    }
    /// Send whole wheel notches at the most recently injected mouse point.
    /// Positive values scroll right/down; units are native wheel notches.
    pub fn wheel(&self, lines_x: f32, lines_y: f32) -> Result<(), ShellError> {
        if [lines_x, lines_y]
            .iter()
            .any(|v| !v.is_finite() || v.fract() != 0.0 || v.abs() > u16::MAX as f32)
        {
            return Err(ShellError::unsupported(
                "X11 wheel injection requires whole native notches",
            ));
        }
        let child = self.child()?;
        let (x, y) = self.point.get();
        for (count, negative, positive) in [(lines_x, 6, 7), (lines_y, 4, 5)] {
            for _ in 0..count.abs() as u32 {
                for down in [true, false] {
                    let event = ButtonPressEvent {
                        response_type: if down {
                            xproto::BUTTON_PRESS_EVENT
                        } else {
                            xproto::BUTTON_RELEASE_EVENT
                        },
                        detail: if count < 0.0 { negative } else { positive },
                        sequence: 0,
                        time: x11rb::CURRENT_TIME,
                        root: self.root,
                        event: child,
                        child: 0,
                        root_x: x as i16,
                        root_y: y as i16,
                        event_x: x as i16,
                        event_y: y as i16,
                        state: KeyButMask::default(),
                        same_screen: true,
                    };
                    let mask = if down {
                        EventMask::BUTTON_PRESS
                    } else {
                        EventMask::BUTTON_RELEASE
                    };
                    self.native
                        .connection
                        .send_event(false, child, mask, event)
                        .map_err(super::error::platform)?
                        .check()
                        .map_err(super::error::platform)?;
                }
            }
        }
        self.native
            .connection
            .flush()
            .map_err(super::error::platform)
    }
    /// Press and release one physical keyboard position.
    pub fn key(&self, key: KeyCode) -> Result<(), ShellError> {
        let detail = (8..=u8::MAX)
            .find(|&code| super::keyboard::physical(code) == key)
            .ok_or_else(|| ShellError::unsupported("physical key has no X11 evdev position"))?;
        self.send_key(detail, KeyButMask::default())
    }
    /// Type characters represented by the server's current keyboard map.
    pub fn text(&self, text: &str) -> Result<(), ShellError> {
        let setup = self.native.connection.setup();
        let first = setup.min_keycode;
        let count = setup.max_keycode.saturating_sub(first).saturating_add(1);
        let mapping = self
            .native
            .connection
            .get_keyboard_mapping(first, count)
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?;
        let columns = usize::from(mapping.keysyms_per_keycode);
        if columns == 0 {
            return Err(super::error::platform("empty server keymap"));
        }
        for character in text.chars() {
            let scalar = character as u32;
            let sym = match character {
                '\n' | '\r' => 0xff0d,
                '\t' => 0xff09,
                _ if scalar <= 0xff => scalar,
                _ => 0x0100_0000 | scalar,
            };
            // Restrict to the current group's ordinary and shifted levels. Do
            // not change the server keymap, which belongs to the user's session.
            let found = mapping
                .keysyms
                .chunks(columns)
                .enumerate()
                .find_map(|(index, symbols)| {
                    symbols
                        .iter()
                        .take(2)
                        .position(|&candidate| candidate == sym)
                        .map(|level| (first + index as u8, level))
                });
            let Some((detail, level)) = found else {
                return Err(ShellError::unsupported(format!(
                    "character {character:?} absent from current X11 keymap"
                )));
            };
            self.send_key(
                detail,
                if level == 1 {
                    KeyButMask::SHIFT
                } else {
                    KeyButMask::default()
                },
            )?;
        }
        Ok(())
    }
    /// Read the child's displayed physical pixels with X11 GetImage.
    pub fn capture(&self) -> Result<CpuFrame, ShellError> {
        self.capture_window(self.child()?)
    }
    /// Read the host client pixels, including overlapping children.
    pub fn capture_host(&self) -> Result<CpuFrame, ShellError> {
        self.capture_window(self.native.id.get())
    }
    fn capture_window(&self, child: u32) -> Result<CpuFrame, ShellError> {
        let connection = &*self.native.connection;
        let geometry = connection
            .get_geometry(child)
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?;
        let image = connection
            .get_image(
                ImageFormat::Z_PIXMAP,
                child,
                0,
                0,
                geometry.width,
                geometry.height,
                u32::MAX,
            )
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?;
        let setup = connection.setup();
        let format = setup
            .pixmap_formats
            .iter()
            .find(|format| format.depth == image.depth)
            .ok_or_else(|| super::error::platform("no pixel format for captured depth"))?;
        let visual = setup
            .roots
            .iter()
            .flat_map(|screen| &screen.allowed_depths)
            .flat_map(|depth| &depth.visuals)
            .find(|visual| visual.visual_id == image.visual)
            .ok_or_else(|| super::error::platform("no visual for captured image"))?;
        let bytes = usize::from(format.bits_per_pixel).div_ceil(8);
        if bytes == 0 || bytes > 4 {
            return Err(super::error::platform("unsupported captured pixel width"));
        }
        let pad = usize::from(format.scanline_pad);
        if pad == 0 {
            return Err(super::error::platform("invalid scanline alignment"));
        }
        let stride =
            (usize::from(geometry.width) * usize::from(format.bits_per_pixel)).div_ceil(pad) * pad
                / 8;
        let mut rgba =
            Vec::with_capacity(usize::from(geometry.width) * usize::from(geometry.height) * 4);
        let component = |pixel: u32, mask: u32| -> u8 {
            if mask == 0 {
                return 0;
            }
            let shift = mask.trailing_zeros();
            (((pixel & mask) >> shift) * 255 / (mask >> shift)) as u8
        };
        for y in 0..usize::from(geometry.height) {
            for x in 0..usize::from(geometry.width) {
                let offset = y * stride + x * bytes;
                let pixel = image
                    .data
                    .get(offset..offset + bytes)
                    .ok_or_else(|| super::error::platform("truncated captured image"))?;
                let value = if setup.image_byte_order == ImageOrder::LSB_FIRST {
                    pixel.iter().enumerate().fold(0, |value, (index, byte)| {
                        value | (u32::from(*byte) << (index * 8))
                    })
                } else {
                    pixel
                        .iter()
                        .fold(0, |value, byte| (value << 8) | u32::from(*byte))
                };
                rgba.extend_from_slice(&[
                    component(value, visual.red_mask),
                    component(value, visual.green_mask),
                    component(value, visual.blue_mask),
                    255,
                ]);
            }
        }
        Ok(CpuFrame {
            width: geometry.width.into(),
            height: geometry.height.into(),
            rgba,
        })
    }
    /// Destroy the host and its children, then wait boundedly for its WM frame.
    pub fn close(&self) {
        if self.native.alive.swap(false, Ordering::AcqRel) {
            let destroyed = self
                .native
                .connection
                .destroy_window(self.native.id.get())
                .map_err(ReplyError::from)
                .and_then(|cookie| cookie.check());
            if let Err(error) = destroyed
                && !matches!(&error, ReplyError::X11Error(error) if error.error_kind == ErrorKind::Window)
            {
                tracing::warn!(%error, "test host destruction failed");
            }
        }
        if let Err(error) = self.native.connection.flush() {
            tracing::warn!(%error, "test host teardown flush failed");
        }
        // DestroyNotify may have already marked the client dead. Its WM can
        // still be processing that client's frame, so this wait is independent.
        let Some(frame) = self.wm_frame.take() else {
            return;
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let exists = self
                .native
                .connection
                .get_window_attributes(frame)
                .map_err(ReplyError::from)
                .and_then(|cookie| cookie.reply());
            match exists {
                Err(ReplyError::X11Error(error)) if error.error_kind == ErrorKind::Window => return,
                Err(error) => {
                    tracing::warn!(frame, %error, "could not observe test host WM-frame teardown");
                    return;
                }
                Ok(_) => {}
            }
            if Instant::now() >= deadline {
                tracing::warn!(
                    frame,
                    "test host WM frame remained after bounded teardown wait"
                );
                return;
            }
            // Keep the client connection and its XID namespace alive until this
            // saved frame disappears. Do not wait for any shared WM/root window.
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    /// Whether this host has been explicitly closed.
    pub fn is_closed(&self) -> bool {
        !self.native.alive.load(Ordering::Acquire)
    }

    fn child(&self) -> Result<u32, ShellError> {
        let covers = self.covers.borrow();
        self.native
            .connection
            .query_tree(self.native.id.get())
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?
            .children
            // QueryTree orders siblings from bottom to top. Target the upper
            // editor while keeping our deliberately overlapping covers separate.
            .into_iter()
            .rev()
            .find(|child| !covers.contains(child))
            .ok_or_else(|| super::error::platform("host has no embedded child"))
    }
    fn button(&self, x: f32, y: f32, down: bool) -> Result<(), ShellError> {
        self.send_button(MouseButton::Primary, x, y, down)
    }
    fn send_button(
        &self,
        button: MouseButton,
        x: f32,
        y: f32,
        down: bool,
    ) -> Result<(), ShellError> {
        let child = self.child()?;
        if down {
            self.focus_child(child)?;
        }
        let event = ButtonPressEvent {
            response_type: if down {
                xproto::BUTTON_PRESS_EVENT
            } else {
                xproto::BUTTON_RELEASE_EVENT
            },
            detail: match button {
                MouseButton::Primary => 1,
                MouseButton::Middle => 2,
                MouseButton::Secondary => 3,
                MouseButton::Back => 8,
                MouseButton::Forward => 9,
            },
            sequence: 0,
            time: x11rb::CURRENT_TIME,
            root: self.root,
            event: child,
            child: 0,
            root_x: x as i16,
            root_y: y as i16,
            event_x: x as i16,
            event_y: y as i16,
            state: if down {
                KeyButMask::default()
            } else {
                match button {
                    MouseButton::Primary => KeyButMask::BUTTON1,
                    MouseButton::Middle => KeyButMask::BUTTON2,
                    MouseButton::Secondary => KeyButMask::BUTTON3,
                    _ => KeyButMask::default(),
                }
            },
            same_screen: true,
        };
        let mask = if down {
            EventMask::BUTTON_PRESS
        } else {
            EventMask::BUTTON_RELEASE
        };
        self.native
            .connection
            .send_event(false, child, mask, event)
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)
    }
    fn send_key(&self, detail: u8, state: KeyButMask) -> Result<(), ShellError> {
        let child = self.child()?;
        self.focus_child(child)?;
        for down in [true, false] {
            let event = KeyPressEvent {
                response_type: if down {
                    xproto::KEY_PRESS_EVENT
                } else {
                    xproto::KEY_RELEASE_EVENT
                },
                detail,
                sequence: 0,
                time: x11rb::CURRENT_TIME,
                root: self.root,
                event: child,
                child: 0,
                root_x: 0,
                root_y: 0,
                event_x: 0,
                event_y: 0,
                state,
                same_screen: true,
            };
            let mask = if down {
                EventMask::KEY_PRESS
            } else {
                EventMask::KEY_RELEASE
            };
            self.native
                .connection
                .send_event(false, child, mask, event)
                .map_err(super::error::platform)?
                .check()
                .map_err(super::error::platform)?;
        }
        Ok(())
    }
    fn focus_child(&self, child: u32) -> Result<(), ShellError> {
        // SendEvent bypasses the WM's normal click-to-focus path. Finish that
        // focus change before queuing Down/Up so its FocusOut cannot cancel
        // the synthetic gesture between those two events.
        self.native
            .connection
            .set_input_focus(xproto::InputFocus::PARENT, child, x11rb::CURRENT_TIME)
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)
    }
}

impl Drop for TestParent {
    fn drop(&mut self) {
        self.close();
    }
}

// A host is a normal, painted top-level window. Set its ICCCM/EWMH identity and
// initial state before mapping; the production embedded client keeps XEmbed info.
fn map_host(native: &Native, width: u32, height: u32) -> Result<(), ShellError> {
    let connection = &*native.connection;
    let id = native.id.get();
    let atom = |name: &[u8]| {
        connection
            .intern_atom(false, name)
            .map_err(super::error::platform)?
            .reply()
            .map(|reply| reply.atom)
            .map_err(super::error::platform)
    };
    for (property, data) in [
        (
            xproto::AtomEnum::WM_NAME,
            b"Arvane embedded test host".as_slice(),
        ),
        (
            xproto::AtomEnum::WM_CLASS,
            b"arvane-test-parent\0ArvaneTestParent\0".as_slice(),
        ),
    ] {
        connection
            .change_property8(
                xproto::PropMode::REPLACE,
                id,
                property,
                xproto::AtomEnum::STRING,
                data,
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
    }
    for (property, kind, value) in [
        (
            atom(b"_NET_WM_PID")?,
            xproto::AtomEnum::CARDINAL,
            std::process::id(),
        ),
        (
            atom(b"_NET_WM_WINDOW_TYPE")?,
            xproto::AtomEnum::ATOM,
            atom(b"_NET_WM_WINDOW_TYPE_NORMAL")?,
        ),
    ] {
        connection
            .change_property32(xproto::PropMode::REPLACE, id, property, kind, &[value])
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
    }
    connection
        .delete_property(id, atom(b"_XEMBED_INFO")?)
        .map_err(super::error::platform)?
        .check()
        .map_err(super::error::platform)?;
    WmHints {
        input: Some(true),
        initial_state: Some(WmHintsState::Normal),
        window_group: Some(id),
        ..Default::default()
    }
    .set(connection, id)
    .map_err(super::error::platform)?
    .check()
    .map_err(super::error::platform)?;
    WmSizeHints {
        position: Some((WmSizeHintsSpecification::UserSpecified, 32, 32)),
        size: Some((
            WmSizeHintsSpecification::ProgramSpecified,
            width as i32,
            height as i32,
        )),
        ..Default::default()
    }
    .set_normal_hints(connection, id)
    .map_err(super::error::platform)?
    .check()
    .map_err(super::error::platform)?;
    connection
        .change_window_attributes(
            id,
            &xproto::ChangeWindowAttributesAux::new().background_pixel(0x202020),
        )
        .map_err(super::error::platform)?
        .check()
        .map_err(super::error::platform)?;
    connection
        .configure_window(id, &xproto::ConfigureWindowAux::new().x(32).y(32))
        .map_err(super::error::platform)?
        .check()
        .map_err(super::error::platform)?;
    connection
        .map_window(id)
        .map_err(super::error::platform)?
        .check()
        .map_err(super::error::platform)?;
    connection
        .clear_area(false, id, 0, 0, 0, 0)
        .map_err(super::error::platform)?
        .check()
        .map_err(super::error::platform)?;
    connection.flush().map_err(super::error::platform)
}
