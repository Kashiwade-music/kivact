//! Role: Receives XDND v5 file drops through the child's own X11 selection.

use super::conn::Native;
use arvane_shell::{ShellError, ShellEvent};
use kivact_types::{FilePhase, Point};
use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};
use x11rb::{
    connection::Connection,
    errors::ReplyError,
    protocol::{
        Event,
        xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, PropMode},
    },
    wrapper::ConnectionExt as _,
};

pub(super) struct Dnd {
    aware: u32,
    incr: u32,
    incremental: Option<Vec<u8>>,
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
    source: u32,
    accepts: bool,
    point: Point,
    pending: bool,
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
        let dnd = Self {
            aware: atom(b"XdndAware")?,
            incr: atom(b"INCR")?,
            incremental: None,
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
            source: 0,
            accepts: false,
            point: Point::default(),
            pending: false,
        };
        native
            .connection
            .change_property32(
                PropMode::REPLACE,
                native.id.get(),
                dnd.aware,
                AtomEnum::ATOM,
                &[5],
            )
            .map_err(super::error::platform)?
            .check()
            .map_err(super::error::platform)?;
        native.connection.flush().map_err(super::error::platform)?;
        Ok(dnd)
    }
    fn send(&self, native: &Native, kind: u32, data: [u32; 5]) -> Result<(), ShellError> {
        if self.source != 0 {
            native
                .connection
                .send_event(
                    false,
                    self.source,
                    EventMask::NO_EVENT,
                    ClientMessageEvent::new(32, self.source, kind, data),
                )
                .map_err(super::error::platform)?;
            native.connection.flush().map_err(super::error::platform)?;
        }
        Ok(())
    }
    /// Drops the current drag. Later messages from that source are ignored.
    fn forget(&mut self) {
        self.pending = false;
        self.incremental = None;
        self.source = 0;
        self.accepts = false;
    }
    fn finish(
        &mut self,
        native: &Native,
        files: Vec<PathBuf>,
        out: &mut Vec<ShellEvent>,
    ) -> Result<(), ShellError> {
        let success = !files.is_empty();
        self.send(
            native,
            self.finished,
            [
                native.id.get(),
                u32::from(success),
                if success { self.copy } else { 0 },
                0,
                0,
            ],
        )?;
        if success {
            out.push(ShellEvent::File {
                phase: FilePhase::Drop,
                files,
                position: self.point,
            });
        }
        self.forget();
        Ok(())
    }
    pub(super) fn event(
        &mut self,
        native: &Native,
        event: &Event,
        scale: f32,
    ) -> Result<Vec<ShellEvent>, ShellError> {
        let mut out = Vec::new();
        match event {
            Event::ClientMessage(e) if e.window == native.id.get() && e.format == 32 => {
                let d = e.data.as_data32();
                if e.type_ == self.enter {
                    self.source = d[0];
                    self.pending = false;
                    self.incremental = None;
                    self.accepts = (d[1] >> 24) >= 3 && (d[1] >> 24) <= 5;
                    let offered = if d[1] & 1 == 0 {
                        d[2..5].to_vec()
                    } else {
                        let reply = native
                            .connection
                            .get_property(false, self.source, self.types, AtomEnum::ATOM, 0, 4096)
                            .map_err(super::error::platform)?
                            .reply();
                        match reply {
                            Ok(reply) => reply.value32().map(Iterator::collect).unwrap_or_default(),
                            // The source is another client's window: its being gone or
                            // unreadable refuses this drag, not the App.
                            Err(ReplyError::X11Error(_)) => {
                                self.forget();
                                return Ok(out);
                            }
                            Err(error) => return Err(super::error::platform(error)),
                        }
                    };
                    self.accepts &= offered.contains(&self.uri);
                } else if e.type_ == self.position && d[0] == self.source {
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
                            native
                                .alive
                                .store(false, std::sync::atomic::Ordering::Release);
                            self.forget();
                            return Ok(out);
                        }
                        Err(ReplyError::X11Error(_)) => {
                            self.accepts = false;
                            self.send(native, self.status, [native.id.get(), 0, 0, 0, 0])?;
                            self.forget();
                            return Ok(out);
                        }
                        Err(error) => return Err(super::error::platform(error)),
                    };
                    self.point = Point {
                        x: f32::from(translated.dst_x) / scale,
                        y: f32::from(translated.dst_y) / scale,
                    };
                    self.send(
                        native,
                        self.status,
                        [
                            native.id.get(),
                            u32::from(self.accepts) | 2,
                            0,
                            0,
                            if self.accepts { self.copy } else { 0 },
                        ],
                    )?;
                } else if e.type_ == self.leave && d[0] == self.source {
                    self.forget();
                } else if e.type_ == self.drop && d[0] == self.source {
                    if self.accepts {
                        self.pending = true;
                        native
                            .connection
                            .convert_selection(
                                native.id.get(),
                                self.selection,
                                self.uri,
                                self.property,
                                d[2],
                            )
                            .map_err(super::error::platform)?;
                        native.connection.flush().map_err(super::error::platform)?;
                    } else {
                        self.send(native, self.finished, [native.id.get(), 0, 0, 0, 0])?;
                    }
                }
            }
            Event::SelectionNotify(e)
                if self.pending
                    && e.requestor == native.id.get()
                    && e.selection == self.selection =>
            {
                if e.property != self.property {
                    self.finish(native, Vec::new(), &mut out)?;
                } else {
                    let reply = native
                        .connection
                        .get_property(
                            false,
                            native.id.get(),
                            self.property,
                            AtomEnum::ANY,
                            0,
                            4_194_304,
                        )
                        .map_err(super::error::platform)?
                        .reply()
                        .map_err(super::error::platform)?;
                    native
                        .connection
                        .delete_property(native.id.get(), self.property)
                        .map_err(super::error::platform)?;
                    native.connection.flush().map_err(super::error::platform)?;
                    if reply.type_ == self.incr && reply.format == 32 {
                        self.incremental = Some(Vec::new());
                    } else {
                        let files = if reply.type_ == self.uri
                            && reply.format == 8
                            && reply.bytes_after == 0
                        {
                            paths(&reply.value)
                        } else {
                            Vec::new()
                        };
                        self.finish(native, files, &mut out)?;
                    }
                }
            }
            Event::PropertyNotify(e)
                if self.pending
                    && self.incremental.is_some()
                    && e.window == native.id.get()
                    && e.atom == self.property
                    && e.state == x11rb::protocol::xproto::Property::NEW_VALUE =>
            {
                let reply = native
                    .connection
                    .get_property(true, native.id.get(), self.property, self.uri, 0, 4_194_304)
                    .map_err(super::error::platform)?
                    .reply()
                    .map_err(super::error::platform)?;
                native.connection.flush().map_err(super::error::platform)?;
                let buffer = self
                    .incremental
                    .as_mut()
                    .expect("incremental transfer is active");
                if reply.type_ != self.uri
                    || reply.format != 8
                    || reply.bytes_after != 0
                    || buffer.len().saturating_add(reply.value.len()) > 16_777_216
                {
                    self.finish(native, Vec::new(), &mut out)?;
                } else if reply.value.is_empty() {
                    let files = paths(buffer);
                    self.finish(native, files, &mut out)?;
                } else {
                    buffer.extend_from_slice(&reply.value);
                }
            }
            _ => {}
        }
        Ok(out)
    }
}
/// This machine's name, which a source may put in a `file://` host.
#[allow(unsafe_code, reason = "gethostname fills a caller-owned buffer")]
fn host_name() -> Option<&'static [u8]> {
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
