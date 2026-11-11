//! Role: Receives XDND v5 file drops through the child's own X11 selection.

use super::conn::Native;
use arvane_shell::ShellError;
use arvane_shell::pipeline::RawDrop;
use kivact_types::Point;
use std::ffi::OsString;
use std::num::NonZeroU32;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, PropMode};
use x11rb::wrapper::ConnectionExt as _;

/// How long a dropped selection may take to arrive, or between its INCR
/// increments, before the drop is given up (a source that stops answering).
const CONVERSION_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) struct Dnd {
    atoms: Atoms,
    drag: Drag,
}
struct Atoms {
    aware: u32,
    incr: u32,
    enter: u32,
    position: u32,
    status: u32,
    drop: u32,
    leave: u32,
    finished: u32,
    selection: u32,
    property: u32,
    uri: u32,
    types: u32,
    copy: u32,
}
enum Drag {
    /// No drag from another client is over the window.
    Idle,
    /// `source` drags over the window; `accepts`: it offers a URI list we take; `point`: the last
    /// XdndPosition (`None` before the first one, so no status was sent yet).
    Hovering {
        source: NonZeroU32,
        // xtask-allow(state-fields): independent: decided once at XdndEnter from the protocol version and the offered types; point is filled by every XdndPosition whether or not we accept (a refusing status is still sent), so all four pairs occur
        accepts: bool,
        // xtask-allow(state-fields): independent: the last XdndPosition, absent until the first one arrives (no status sent yet); it is filled whatever accepts says, and drop takes the URIs only for accepts with a point, so all four pairs occur
        point: Option<Point>,
    },
    /// `source` dropped at `point`; its selection is being converted, given up at `deadline`.
    Converting {
        source: NonZeroU32,
        point: Point,
        transfer: Transfer,
        deadline: Instant,
    },
}
enum Transfer {
    /// The selection was requested; its notification has not arrived.
    Requested,
    /// The source sends the list in increments (INCR); the bytes so far.
    Incremental(Vec<u8>),
}
impl Dnd {
    pub(super) fn new(native: &Native) -> Result<Self, ShellError> {
        let atom = |name: &[u8]| -> Result<u32, ShellError> {
            Ok(native
                .connection
                .intern_atom(false, name)
                .map_err(super::error::platform)?
                .reply()
                .map_err(super::error::platform)?
                .atom)
        };
        let atoms = Atoms {
            aware: atom(b"XdndAware")?,
            incr: atom(b"INCR")?,
            enter: atom(b"XdndEnter")?,
            position: atom(b"XdndPosition")?,
            status: atom(b"XdndStatus")?,
            drop: atom(b"XdndDrop")?,
            leave: atom(b"XdndLeave")?,
            finished: atom(b"XdndFinished")?,
            selection: atom(b"XdndSelection")?,
            property: atom(b"KIVACT_XDND")?,
            uri: atom(b"text/uri-list")?,
            types: atom(b"XdndTypeList")?,
            copy: atom(b"XdndActionCopy")?,
        };
        native
            .connection
            .change_property32(
                PropMode::REPLACE,
                native.id.get(),
                atoms.aware,
                AtomEnum::ATOM,
                &[5],
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        native.connection.flush().map_err(super::error::platform)?;
        Ok(Self {
            atoms,
            drag: Drag::Idle,
        })
    }
    fn send(
        &self,
        native: &Native,
        source: NonZeroU32,
        kind: u32,
        data: [u32; 5],
    ) -> Result<(), ShellError> {
        native
            .connection
            .send_event(
                false,
                source.get(),
                EventMask::NO_EVENT,
                ClientMessageEvent::new(32, source.get(), kind, data),
            )
            .map_err(super::error::platform)?;
        native.connection.flush().map_err(super::error::platform)
    }
    /// Ends the conversion: tells the source whether the drop succeeded and reports the files.
    fn finish(
        &mut self,
        native: &Native,
        files: Vec<PathBuf>,
        out: &mut Vec<RawDrop>,
    ) -> Result<(), ShellError> {
        let Drag::Converting { source, point, .. } = std::mem::replace(&mut self.drag, Drag::Idle)
        else {
            return Ok(());
        };
        let success = !files.is_empty();
        self.send(
            native,
            source,
            self.atoms.finished,
            [
                native.id.get(),
                u32::from(success),
                if success { self.atoms.copy } else { 0 },
                0,
                0,
            ],
        )?;
        if success {
            out.push(RawDrop::Hover {
                files,
                at: Some(point),
            });
            out.push(RawDrop::Drop);
        }
        Ok(())
    }
    /// Nanoseconds until a conversion in progress is given up, if one is.
    pub(super) fn remaining_ns(&self) -> Option<u64> {
        let Drag::Converting { deadline, .. } = &self.drag else {
            return None;
        };
        let remaining = deadline.saturating_duration_since(Instant::now());
        Some(u64::try_from(remaining.as_nanos()).unwrap_or(u64::MAX))
    }
    /// Give up a conversion past its deadline: the source is told the drop failed.
    pub(super) fn expire(&mut self, native: &Native) -> Result<(), ShellError> {
        match &self.drag {
            Drag::Converting { deadline, .. } if *deadline <= Instant::now() => {
                self.finish(native, Vec::new(), &mut Vec::new())
            }
            _ => Ok(()),
        }
    }
    /// The conversion's property on our window, or `None` when the window is gone (the host
    /// destroyed our parent: a close, not a failure; the drag ends).
    fn read_property(
        &mut self,
        native: &Native,
        delete: bool,
        kind: u32,
    ) -> Result<Option<x11rb::protocol::xproto::GetPropertyReply>, ShellError> {
        let reply = native
            .connection
            .get_property(
                delete,
                native.id.get(),
                self.atoms.property,
                kind,
                0,
                4_194_304,
            )
            .map_err(super::error::platform)?
            .reply();
        match reply {
            Ok(reply) => Ok(Some(reply)),
            Err(error) if super::error::destroyed(&error, native.id.get()) => {
                native.exists.end();
                self.drag = Drag::Idle;
                Ok(None)
            }
            Err(error) => Err(super::error::platform(error)),
        }
    }
    pub(super) fn event(
        &mut self,
        native: &Native,
        event: &Event,
        scale: f64,
    ) -> Result<Vec<RawDrop>, ShellError> {
        let mut out = Vec::new();
        match event {
            Event::ClientMessage(e) if e.window == native.id.get() && e.format == 32 => {
                self.client_message(native, e, scale)?;
            }
            Event::SelectionNotify(e)
                if matches!(
                    self.drag,
                    Drag::Converting {
                        transfer: Transfer::Requested,
                        ..
                    }
                ) && e.requestor == native.id.get()
                    && e.selection == self.atoms.selection =>
            {
                self.selection(native, e, &mut out)?;
            }
            Event::PropertyNotify(e)
                if matches!(
                    self.drag,
                    Drag::Converting {
                        transfer: Transfer::Incremental(_),
                        ..
                    }
                ) && e.window == native.id.get()
                    && e.atom == self.atoms.property
                    && e.state == x11rb::protocol::xproto::Property::NEW_VALUE =>
            {
                self.increment(native, &mut out)?;
            }
            _ => {}
        }
        Ok(out)
    }
    /// Receive an XDND selection transfer notification.
    fn selection(
        &mut self,
        native: &Native,
        e: &x11rb::protocol::xproto::SelectionNotifyEvent,
        out: &mut Vec<RawDrop>,
    ) -> Result<(), ShellError> {
        if e.property != self.atoms.property {
            self.finish(native, Vec::new(), out)?;
        } else {
            let Some(reply) = self.read_property(native, false, AtomEnum::ANY.into())? else {
                return Ok(());
            };
            native
                .connection
                .delete_property(native.id.get(), self.atoms.property)
                .map_err(super::error::platform)?;
            native.connection.flush().map_err(super::error::platform)?;
            if reply.type_ == self.atoms.incr && reply.format == 32 {
                if let Drag::Converting {
                    transfer, deadline, ..
                } = &mut self.drag
                {
                    *transfer = Transfer::Incremental(Vec::new());
                    *deadline = Instant::now() + CONVERSION_TIMEOUT;
                }
            } else {
                let files =
                    if reply.type_ == self.atoms.uri && reply.format == 8 && reply.bytes_after == 0
                    {
                        paths(&reply.value)
                    } else {
                        Vec::new()
                    };
                self.finish(native, files, out)?;
            }
        }
        Ok(())
    }

    /// Receive an XDND increment transfer notification.
    fn increment(&mut self, native: &Native, out: &mut Vec<RawDrop>) -> Result<(), ShellError> {
        let Some(reply) = self.read_property(native, true, self.atoms.uri)? else {
            return Ok(());
        };
        native.connection.flush().map_err(super::error::platform)?;
        let Drag::Converting {
            transfer: Transfer::Incremental(buffer),
            deadline,
            ..
        } = &mut self.drag
        else {
            return Ok(());
        };
        if reply.type_ != self.atoms.uri
            || reply.format != 8
            || reply.bytes_after != 0
            || buffer.len().saturating_add(reply.value.len()) > 16_777_216
        {
            self.finish(native, Vec::new(), out)?;
        } else if reply.value.is_empty() {
            let files = paths(buffer);
            self.finish(native, files, out)?;
        } else {
            buffer.extend_from_slice(&reply.value);
            *deadline = Instant::now() + CONVERSION_TIMEOUT;
        }
        Ok(())
    }

    /// Dispatch client messages against the current drag source.
    fn client_message(
        &mut self,
        native: &Native,
        e: &x11rb::protocol::xproto::ClientMessageEvent,
        scale: f64,
    ) -> Result<(), ShellError> {
        let d = e.data.as_data32();
        // The drag this message belongs to, when its source is the one hovering.
        let hovering = match self.drag {
            Drag::Hovering {
                source,
                accepts,
                point,
            } if source.get() == d[0] => Some((source, accepts, point)),
            _ => None,
        };
        if e.type_ == self.atoms.enter {
            self.enter(native, d)?;
        } else if e.type_ == self.atoms.position
            && let Some((source, accepts, _)) = hovering
        {
            self.position(native, source, accepts, d, scale)?;
        } else if e.type_ == self.atoms.leave && hovering.is_some() {
            self.drag = Drag::Idle;
        } else if e.type_ == self.atoms.drop
            && let Some((source, accepts, point)) = hovering
        {
            match (accepts, point) {
                (true, Some(point)) => {
                    self.drag = Drag::Converting {
                        source,
                        point,
                        transfer: Transfer::Requested,
                        deadline: Instant::now() + CONVERSION_TIMEOUT,
                    };
                    native
                        .connection
                        .convert_selection(
                            native.id.get(),
                            self.atoms.selection,
                            self.atoms.uri,
                            self.atoms.property,
                            d[2],
                        )
                        .map_err(super::error::platform)?;
                    native.connection.flush().map_err(super::error::platform)?;
                }
                _ => {
                    self.drag = Drag::Idle;
                    self.send(
                        native,
                        source,
                        self.atoms.finished,
                        [native.id.get(), 0, 0, 0, 0],
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Begin a drag and inspect its offered MIME types.
    fn enter(&mut self, native: &Native, d: [u32; 5]) -> Result<(), ShellError> {
        let Some(source) = NonZeroU32::new(d[0]) else {
            return Ok(());
        };
        // A new drag abandons a conversion in progress: its source is told it failed.
        if let Drag::Converting { source: old, .. } = std::mem::replace(&mut self.drag, Drag::Idle)
        {
            self.send(
                native,
                old,
                self.atoms.finished,
                [native.id.get(), 0, 0, 0, 0],
            )?;
        }
        let mut accepts = (d[1] >> 24) >= 3 && (d[1] >> 24) <= 5;
        let offered = if d[1] & 1 == 0 {
            d[2..5].to_vec()
        } else {
            let reply = native
                .connection
                .get_property(
                    false,
                    source.get(),
                    self.atoms.types,
                    AtomEnum::ATOM,
                    0,
                    4096,
                )
                .map_err(super::error::platform)?
                .reply();
            match reply {
                Ok(reply) => reply.value32().map(Iterator::collect).unwrap_or_default(),
                // The source is another client's window: its being gone or
                // unreadable refuses this drag, not the App.
                Err(ReplyError::X11Error(_)) => return Ok(()),
                Err(error) => return Err(super::error::platform(error)),
            }
        };
        accepts &= offered.contains(&self.atoms.uri);
        self.drag = Drag::Hovering {
            source,
            accepts,
            point: None,
        };
        Ok(())
    }

    /// Translate the drag position and acknowledge receiver acceptance.
    fn position(
        &mut self,
        native: &Native,
        source: NonZeroU32,
        accepts: bool,
        d: [u32; 5],
        scale: f64,
    ) -> Result<(), ShellError> {
        let x = (d[2] >> 16) as i16;
        let y = d[2] as i16;
        let root = native.connection.setup().roots[native.screen as usize].root;
        let translated = native
            .connection
            .translate_coordinates(root, native.id.get(), x, y)
            .map_err(super::error::platform)?
            .reply();
        let translated = match translated {
            Ok(translated) => translated,
            Err(error) if super::error::destroyed(&error, native.id.get()) => {
                // The host destroyed our parent: a close, not a failure.
                native.exists.end();
                self.drag = Drag::Idle;
                return Ok(());
            }
            Err(ReplyError::X11Error(_)) => {
                self.drag = Drag::Idle;
                self.send(
                    native,
                    source,
                    self.atoms.status,
                    [native.id.get(), 0, 0, 0, 0],
                )?;
                return Ok(());
            }
            Err(error) => return Err(super::error::platform(error)),
        };
        self.drag = Drag::Hovering {
            source,
            accepts,
            point: Some(Point {
                x: f64::from(translated.dst_x) / scale,
                y: f64::from(translated.dst_y) / scale,
            }),
        };
        self.send(
            native,
            source,
            self.atoms.status,
            [
                native.id.get(),
                u32::from(accepts) | 2,
                0,
                0,
                if accepts { self.atoms.copy } else { 0 },
            ],
        )?;
        Ok(())
    }
}
/// This machine's name, which a source may put in a `file://` host.
#[allow(unsafe_code, reason = "gethostname fills a caller-owned buffer")]
fn host_name() -> Option<&'static [u8]> {
    #[expect(
        clippy::disallowed_types,
        reason = "process-wide hostname snapshot initialized once"
    )]
    static NAME: std::sync::OnceLock<Option<Vec<u8>>> = std::sync::OnceLock::new();
    NAME.get_or_init(|| {
        let mut buffer = [0_u8; 256];
        // SAFETY: the buffer is writable for its whole length.
        let result = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
        // A truncated name is not NUL-terminated.
        *buffer.last_mut()? = 0;
        let length = buffer.iter().position(|byte| *byte == 0)?;
        (result == 0 && length > 0).then(|| buffer[..length].to_vec())
    })
    .as_deref()
}
/// Whether a `file://` host names this machine: empty, `localhost` or its own
/// host name, compared without regard to ASCII case.
fn local(host: &[u8], own: Option<&[u8]>) -> bool {
    host.is_empty()
        || host.eq_ignore_ascii_case(b"localhost")
        || own.is_some_and(|own| host.eq_ignore_ascii_case(own))
}
fn paths(data: &[u8]) -> Vec<PathBuf> {
    paths_on(data, host_name())
}
fn paths_on(data: &[u8], own: Option<&[u8]>) -> Vec<PathBuf> {
    data.split(|b| *b == b'\n')
        .filter_map(|line| {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.starts_with(b"#") {
                return None;
            }
            let uri = line.strip_prefix(b"file://")?;
            let (host, path) = uri.split_at(uri.iter().position(|b| *b == b'/')?);
            if !local(host, own) {
                return None;
            }
            let mut bytes = Vec::new();
            let mut i = 0;
            while i < path.len() {
                if path[i] == b'%' {
                    let digits = std::str::from_utf8(path.get(i + 1..i + 3)?).ok()?;
                    bytes.push(u8::from_str_radix(digits, 16).ok()?);
                    i += 3;
                } else {
                    bytes.push(path[i]);
                    i += 1;
                }
            }
            if bytes.contains(&0) {
                return None;
            }
            Some(PathBuf::from(OsString::from_vec(bytes)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    //! Role: Checks text/uri-list parsing of file drops.

    use super::*;

    fn one(uri: &str, own: Option<&[u8]>) -> Option<PathBuf> {
        paths_on(uri.as_bytes(), own).into_iter().next()
    }

    #[test]
    fn local_hosts_are_accepted() {
        let path = Some(PathBuf::from("/tmp/a"));
        assert_eq!(one("file:///tmp/a", None), path);
        assert_eq!(one("file://localhost/tmp/a", None), path);
        assert_eq!(one("file://LocalHost/tmp/a", None), path);
        assert_eq!(one("file://Build-Box/tmp/a", Some(b"build-box")), path);
    }

    #[test]
    fn the_real_host_name_is_accepted() {
        let Some(name) = host_name() else {
            return;
        };
        let uri = format!("file://{}/tmp/a", String::from_utf8_lossy(name));
        assert_eq!(
            paths(uri.as_bytes()),
            vec![PathBuf::from("/tmp/a")],
            "{uri}"
        );
    }

    #[test]
    fn foreign_hosts_and_malformed_uris_are_rejected() {
        assert_eq!(one("file://other-host/tmp/a", Some(b"build-box")), None);
        assert_eq!(one("file://localhost.example/tmp/a", None), None);
        assert_eq!(one("file://", None), None);
        assert_eq!(one("file://localhost", None), None);
        assert_eq!(one("http://localhost/tmp/a", None), None);
        assert_eq!(one("file:///tmp/%zz", None), None);
        assert_eq!(one("file:///tmp/%2", None), None);
        assert_eq!(one("file:///tmp/a%00b", None), None);
    }

    #[test]
    fn paths_are_percent_decoded_and_comments_skipped() {
        let list =
            b"# comment\r\nfile:///tmp/a%20b%E6%97%A5.txt\r\nfile://other/x\nfile:///tmp/c%23\n";
        assert_eq!(
            paths_on(list, None),
            vec![
                PathBuf::from("/tmp/a b\u{65e5}.txt"),
                PathBuf::from("/tmp/c#")
            ]
        );
    }
}
