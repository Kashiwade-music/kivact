//! Role: Runs XIM on the child's XCB connection, with XKB when no server exists.
//!
//! Invariants: at most one synchronous request is in flight and it waits at
//! most 500 ms for its acknowledgement; later native keys wait in a queue of at
//! most 1024 events. Returned keys reach Keyboard once and are never forwarded
//! again. Canceled raw releases are locally reconciled only against delivered
//! downs. A canceled flight remains a tombstone until its old acknowledgement
//! arrives. Only SyncReply is peeked here; normal transport handling and
//! protocol acknowledgements belong to the locked client. XIM is an optional
//! service: every failure of the server, the transport or the protocol (and
//! every panic of the client library on server input) ends in `degrade`, which
//! drops the client and gives the undelivered keys back to Keyboard in order.

use super::conn::Native;
use arvane_shell::{ShellError, ShellEvent};
use std::{
    cell::RefCell,
    collections::VecDeque,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
    time::{Duration, Instant},
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{AtomEnum, ConnectionExt, KeyPressEvent},
    },
    x11_utils::X11Error,
    xcb_ffi::XCBConnection,
};
use xim::{
    AttributeName, Client, ClientCore, ClientError, ClientHandler, ForwardEventFlag, InputStyle,
    x11rb::{HasConnection, X11rbClient},
};

type ClientConnection = X11rbClient<Arc<XCBConnection>>;

/// How long a synchronous forward may wait for its acknowledgement.
const FLIGHT_TIMEOUT: Duration = Duration::from_millis(500);

fn locale() -> String {
    // XIM Open takes the character locale, not a BCP 47 UI language tag.
    // Read POSIX precedence without changing the host's process locale.
    ["LC_ALL", "LC_CTYPE", "LANG"]
        .into_iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
        .unwrap_or_else(|| "C".into())
}

/// Runs one call into the xim crate, which panics on some malformed server
/// input. An error and a panic both mean the server cannot be used any more.
fn guarded<T>(call: impl FnOnce() -> Result<T, ClientError>) -> Result<T, String> {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(error.to_string()),
        Err(_) => Err("the xim client panicked".into()),
    }
}

pub(super) struct Ime {
    client: Option<ClientConnection>,
    handler: Handler,
    protocol: u32,
    xconnect: u32,
    protocol_window: Option<u32>,
    server_window: Option<u32>,
    pending: VecDeque<(KeyPressEvent, bool)>,
    flight: Option<Flight>,
    cancelled_releases: Vec<KeyPressEvent>,
}
struct Flight {
    im: u16,
    ic: u16,
    canceled: bool,
    event: KeyPressEvent,
    down: bool,
    deadline: Instant,
}
struct Handler {
    window: u32,
    locale: String,
    im: u16,
    ic: u16,
    allowed: bool,
    focused: bool,
    forward: u32,
    spot: xim::Point,
    /// The spot the server last learned of, by IC creation or SetIcValues.
    sent_spot: Option<(i16, i16)>,
    preedit: String,
    caret: i32,
    disconnected: bool,
    events: Vec<ShellEvent>,
    returned: Vec<KeyPressEvent>,
}
impl Ime {
    /// IME is optional: when the server cannot be reached the window opens
    /// without it.
    pub(super) fn new(native: &Native) -> RefCell<Self> {
        let (client, protocol, xconnect) = match connect(native) {
            Ok(parts) => parts,
            Err(error) => {
                tracing::warn!(target: "kivact", "X11 IME unavailable: {error}");
                (None, 0, 0)
            }
        };
        RefCell::new(Self {
            client,
            protocol,
            xconnect,
            protocol_window: None,
            server_window: None,
            pending: VecDeque::new(),
            flight: None,
            cancelled_releases: Vec::new(),
            handler: Handler {
                window: native.id.get(),
                locale: locale(),
                im: 0,
                ic: 0,
                allowed: false,
                focused: false,
                forward: u32::from(x11rb::protocol::xproto::EventMask::KEY_PRESS),
                spot: xim::Point { x: 0, y: 0 },
                sent_spot: None,
                preedit: String::new(),
                caret: 0,
                disconnected: false,
                events: Vec::new(),
                returned: Vec::new(),
            },
        })
    }
    /// Gives up XIM for the rest of the window's life: warns once, drops the
    /// client, reports IME disabled and hands the in-flight and queued keys to
    /// Keyboard in their original order.
    pub(super) fn degrade(&mut self, reason: &str) {
        if self.client.take().is_none() {
            return;
        }
        tracing::warn!(target: "kivact", "X11 IME disabled, using the keyboard directly: {reason}");
        self.protocol_window = None;
        self.server_window = None;
        let h = &mut self.handler;
        h.im = 0;
        h.ic = 0;
        h.sent_spot = None;
        h.preedit.clear();
        h.disconnected = false;
        h.events.push(ShellEvent::ImeDisabled);
        if let Some(flight) = self.flight.take()
            && !flight.canceled
            // A server that already returned the key must not have it twice.
            && !h.returned.iter().any(|key| same_key(key, &flight.event))
        {
            h.returned.push(flight.event);
        }
        h.returned
            .extend(self.pending.drain(..).map(|(event, _)| event));
    }
    /// A lost XIM server surfaces as an X error on its window.
    pub(super) fn x_error(&mut self, error: &X11Error) {
        if self.client.is_some()
            && error.error_kind == x11rb::protocol::ErrorKind::Window
            && self.server_window == Some(error.bad_value)
        {
            self.degrade("XIM server window is gone");
        }
    }
    /// Nanoseconds until the in-flight request is given up, if one is waiting.
    pub(super) fn remaining_ns(&self) -> Option<u64> {
        let flight = self.flight.as_ref()?;
        let remaining = flight.deadline.saturating_duration_since(Instant::now());
        Some(u64::try_from(remaining.as_nanos()).unwrap_or(u64::MAX))
    }
    pub(super) fn filter(&mut self, event: &Event) -> bool {
        let Some(client) = &mut self.client else {
            return false;
        };
        let observe = self.flight.is_some()
            && matches!(event, Event::ClientMessage(message)
            if Some(message.window) == self.protocol_window);
        let protocol = self.protocol;
        let handler = &mut self.handler;
        let result = guarded(|| {
            let reply = if observe {
                sync_reply(client, protocol, event)?
            } else {
                None
            };
            let filtered = client.filter_event(event, handler)?;
            Ok((reply, filtered))
        });
        let (reply, filtered) = match result {
            Ok(parts) => parts,
            Err(reason) => {
                self.degrade(&reason);
                return false;
            }
        };
        if self.handler.disconnected {
            self.degrade("XIM server disconnected");
            return filtered;
        }
        if filtered && let Event::ClientMessage(message) = event {
            self.protocol_window = Some(message.window);
            if message.type_ == self.xconnect && message.format == 32 {
                self.server_window = Some(message.data.as_data32()[0]);
            }
        }
        let canceled = self.flight.as_ref().is_some_and(|flight| flight.canceled);
        // A canceled request's delayed output must not edit the newly focused input.
        if canceled {
            self.handler
                .events
                .retain(|event| matches!(event, ShellEvent::ImeDisabled));
            // Known canceled releases were reconciled at cancellation time.
            // Delayed output must not release a newer local down of that key.
            self.handler.returned.clear();
            self.handler.preedit.clear();
        }
        if filtered
            && let Some((im, ic)) = reply
            && self
                .flight
                .as_ref()
                .is_some_and(|flight| flight.im == im && flight.ic == ic)
        {
            self.flight = None;
            self.advance();
        }
        filtered
    }
    /// Whether XIM took the key; a refused key goes to Keyboard directly.
    pub(super) fn forward(&mut self, event: &KeyPressEvent, down: bool) -> bool {
        let h = &self.handler;
        if self.client.is_none() || h.ic == 0 || !h.allowed || !h.focused {
            return false;
        }
        let mask = if down { 1 } else { 2 };
        if self.flight.is_none() && self.pending.is_empty() && h.forward & mask == 0 {
            return false;
        }
        if self.pending.len() >= 1024 {
            // The new key follows the queued ones it was waiting behind.
            self.degrade("XIM pending key queue exceeded 1024 events");
            self.handler.returned.push(*event);
            return true;
        }
        self.pending.push_back((*event, down));
        self.advance();
        true
    }
    fn advance(&mut self) {
        while self.flight.is_none() {
            let Some((event, down)) = self.pending.pop_front() else {
                break;
            };
            let h = &mut self.handler;
            if !h.allowed || !h.focused || h.ic == 0 {
                self.pending.clear();
                break;
            }
            let mask = if down { 1 } else { 2 };
            if h.forward & mask == 0 {
                // Local releases wait behind the forwarded press, then use the
                // same returned-key path without ever being sent back to XIM.
                h.returned.push(event);
                continue;
            }
            let Some(client) = &mut self.client else {
                break;
            };
            let (im, ic) = (h.im, h.ic);
            let sent = guarded(|| {
                client.forward_event(im, ic, ForwardEventFlag::SYNCHRONOUS, &event)?;
                client.conn().flush()?;
                Ok(())
            });
            match sent {
                Ok(()) => {
                    self.flight = Some(Flight {
                        im,
                        ic,
                        canceled: false,
                        event,
                        down,
                        deadline: Instant::now() + FLIGHT_TIMEOUT,
                    });
                }
                Err(reason) => {
                    self.pending.push_front((event, down));
                    self.degrade(&reason);
                    return;
                }
            }
        }
    }
    fn collect_returned_releases(&mut self) {
        self.cancelled_releases
            .extend(self.handler.returned.drain(..).filter(|event| {
                event.response_type & 0x7f == x11rb::protocol::xproto::KEY_RELEASE_EVENT
            }));
    }
    fn cancel_pending(&mut self) {
        // The in-flight event precedes the queued events; preserve release
        // order so their native modifier masks are applied in the same order.
        if let Some(flight) = &mut self.flight {
            if !flight.canceled && !flight.down {
                self.cancelled_releases.push(flight.event);
            }
            flight.canceled = true;
        }
        self.collect_returned_releases();
        self.cancelled_releases.extend(
            self.pending
                .drain(..)
                .filter_map(|(event, down)| (!down).then_some(event)),
        );
        self.handler
            .events
            .retain(|event| matches!(event, ShellEvent::ImeDisabled));
        self.handler.preedit.clear();
    }
    pub(super) fn drain(&mut self) -> (Vec<ShellEvent>, Vec<KeyPressEvent>, Vec<KeyPressEvent>) {
        // The loop drains once per pass and wakes for the flight deadline, so
        // a reply that never comes is noticed here.
        if self
            .flight
            .as_ref()
            .is_some_and(|flight| Instant::now() >= flight.deadline)
        {
            self.degrade("XIM server did not answer");
        }
        (
            std::mem::take(&mut self.handler.events),
            std::mem::take(&mut self.handler.returned),
            std::mem::take(&mut self.cancelled_releases),
        )
    }
    pub(super) fn focus(&mut self, focused: bool) {
        if self.handler.focused == focused {
            return;
        }
        if !focused {
            self.cancel_pending();
        }
        self.handler.focused = focused;
        self.sync_focus();
    }
    // Never fails: a broken server is degraded away. The window's shared call
    // site still takes a Result.
    pub(super) fn allow(&mut self, allowed: bool) -> Result<(), ShellError> {
        if self.handler.allowed == allowed {
            return Ok(());
        }
        if !allowed {
            self.cancel_pending();
        }
        self.handler.allowed = allowed;
        self.sync_focus();
        Ok(())
    }
    fn sync_focus(&mut self) {
        let h = &mut self.handler;
        let Some(client) = &mut self.client else {
            return;
        };
        if h.ic == 0 {
            return;
        }
        let on = h.allowed && h.focused;
        if !on {
            h.preedit.clear();
            h.events.push(ShellEvent::ImeDisabled);
        }
        let (im, ic) = (h.im, h.ic);
        let result = guarded(|| {
            if on {
                client.set_focus(im, ic)?;
            } else {
                client.unset_focus(im, ic)?;
            }
            client.conn().flush()?;
            Ok(())
        });
        if let Err(reason) = result {
            self.degrade(&reason);
        }
    }
    // Never fails, like `allow`.
    pub(super) fn cursor(
        &mut self,
        area: kivact_types::Rect,
        scale: f32,
    ) -> Result<(), ShellError> {
        let spot = xim::Point {
            x: (area.x * scale).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
            y: ((area.y + area.height) * scale).clamp(i16::MIN as f32, i16::MAX as f32) as i16,
        };
        let key = (spot.x, spot.y);
        self.handler.spot = spot.clone();
        // The engine reports the cursor area on every turn; the server needs
        // to hear only about a change.
        if self.handler.sent_spot == Some(key) {
            return Ok(());
        }
        let (im, ic) = (self.handler.im, self.handler.ic);
        if let Some(client) = &mut self.client
            && ic != 0
        {
            let result = guarded(|| {
                let attrs = client
                    .build_ic_attributes()
                    .nested_list(AttributeName::PreeditAttributes, |b| {
                        b.push(AttributeName::SpotLocation, spot.clone());
                    })
                    .build();
                client.set_ic_values(im, ic, attrs)?;
                client.conn().flush()?;
                Ok(())
            });
            match result {
                Ok(()) => self.handler.sent_spot = Some(key),
                Err(reason) => self.degrade(&reason),
            }
        }
        Ok(())
    }
}
fn same_key(a: &KeyPressEvent, b: &KeyPressEvent) -> bool {
    a.detail == b.detail && a.time == b.time && a.response_type & 0x7f == b.response_type & 0x7f
}
/// The client with its protocol atoms, or `None` when no server is announced.
fn connect(native: &Native) -> Result<(Option<ClientConnection>, u32, u32), ShellError> {
    let connection = native.connection.clone();
    let root = connection.setup().roots[native.screen as usize].root;
    let servers = connection
        .intern_atom(true, b"XIM_SERVERS")
        .map_err(super::error::platform)?
        .reply()
        .map_err(super::error::platform)?
        .atom;
    let present = if servers == 0 {
        false
    } else {
        let reply = connection
            .get_property(false, root, servers, AtomEnum::ATOM, 0, 4096)
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?;
        reply.type_ == u32::from(AtomEnum::ATOM) && reply.format == 32 && !reply.value.is_empty()
    };
    if !present {
        return Ok((None, 0, 0));
    }
    let init = catch_unwind(AssertUnwindSafe(|| {
        ClientConnection::init(connection, native.screen as usize, None)
    }));
    let client = match init {
        Ok(Ok(client)) => client,
        Ok(Err(ClientError::NoXimServer)) => return Ok((None, 0, 0)),
        Ok(Err(error)) => return Err(super::error::platform(error)),
        Err(_) => return Err(super::error::platform("the xim client panicked")),
    };
    let atom = |name: &[u8]| -> Result<u32, ShellError> {
        Ok(client
            .conn()
            .intern_atom(false, name)
            .map_err(super::error::platform)?
            .reply()
            .map_err(super::error::platform)?
            .atom)
    };
    let protocol = atom(b"_XIM_PROTOCOL")?;
    let xconnect = atom(b"_XIM_XCONNECT")?;
    Ok((Some(client), protocol, xconnect))
}
// The locked client consumes SyncReply without a handler callback. Observe
// only that request; the library remains the owner of parsing and transport.
fn sync_reply(
    client: &ClientConnection,
    protocol: u32,
    event: &Event,
) -> Result<Option<(u16, u16)>, ClientError> {
    let Event::ClientMessage(message) = event else {
        return Ok(None);
    };
    if message.type_ != protocol {
        return Ok(None);
    }
    let request = if message.format == 8 {
        xim::read::<xim::Request>(&message.data.as_data8()).ok()
    } else if message.format == 32 {
        let [length, property, ..] = message.data.as_data32();
        let reply = client
            .conn()
            .get_property(false, message.window, property, AtomEnum::ANY, 0, length)?
            .reply()?;
        xim::read::<xim::Request>(&reply.value).ok()
    } else {
        None
    };
    Ok(match request {
        Some(xim::Request::SyncReply {
            input_method_id,
            input_context_id,
        }) => Some((input_method_id, input_context_id)),
        _ => None,
    })
}
// XIM uses character offsets; the shell's selection offsets are UTF-8 bytes.
fn byte(text: &str, character: i32) -> usize {
    text.char_indices()
        .nth(character.max(0) as usize)
        .map_or(text.len(), |(index, _)| index)
}
impl ClientHandler<ClientConnection> for Handler {
    fn handle_connect(&mut self, client: &mut ClientConnection) -> Result<(), ClientError> {
        client.open(&self.locale)
    }
    fn handle_open(&mut self, client: &mut ClientConnection, im: u16) -> Result<(), ClientError> {
        self.im = im;
        client.get_im_values(im, &[AttributeName::QueryInputStyle])
    }
    fn handle_get_im_values(
        &mut self,
        client: &mut ClientConnection,
        im: u16,
        attrs: xim::AHashMap<AttributeName, Vec<u8>>,
    ) -> Result<(), ClientError> {
        let styles: xim::InputStyleList =
            xim::read(attrs.get(&AttributeName::QueryInputStyle).ok_or_else(|| {
                ClientError::Other("the server did not report its input styles".into())
            })?)?;
        let desired = InputStyle::PREEDIT_CALLBACKS | InputStyle::STATUS_NOTHING;
        if !styles.styles.contains(&desired) {
            return Err(ClientError::Other(
                "the server does not support preedit callbacks".into(),
            ));
        }
        let attrs = client
            .build_ic_attributes()
            .push(
                AttributeName::InputStyle,
                InputStyle::PREEDIT_CALLBACKS | InputStyle::STATUS_NOTHING,
            )
            .push(AttributeName::ClientWindow, self.window)
            .push(AttributeName::FocusWindow, self.window)
            .nested_list(AttributeName::PreeditAttributes, |b| {
                b.push(AttributeName::SpotLocation, self.spot.clone());
            })
            .build();
        self.sent_spot = Some((self.spot.x, self.spot.y));
        client.create_ic(im, attrs)
    }
    fn handle_create_ic(
        &mut self,
        client: &mut ClientConnection,
        im: u16,
        ic: u16,
    ) -> Result<(), ClientError> {
        self.ic = ic;
        if self.allowed && self.focused {
            client.set_focus(im, ic)?;
        }
        Ok(())
    }
    fn handle_set_event_mask(
        &mut self,
        _: &mut ClientConnection,
        _: u16,
        _: u16,
        forward: u32,
        _: u32,
    ) -> Result<(), ClientError> {
        self.forward = forward;
        Ok(())
    }
    fn handle_forward_event(
        &mut self,
        _: &mut ClientConnection,
        _: u16,
        _: u16,
        _: ForwardEventFlag,
        event: KeyPressEvent,
    ) -> Result<(), ClientError> {
        self.returned.push(event);
        Ok(())
    }
    fn handle_commit(
        &mut self,
        _: &mut ClientConnection,
        _: u16,
        _: u16,
        text: &str,
    ) -> Result<(), ClientError> {
        self.preedit.clear();
        self.events.push(ShellEvent::Commit(text.into()));
        Ok(())
    }
    fn handle_preedit_start(
        &mut self,
        client: &mut ClientConnection,
        im: u16,
        ic: u16,
    ) -> Result<(), ClientError> {
        self.preedit.clear();
        client.send_req(xim::Request::PreeditStartReply {
            input_method_id: im,
            input_context_id: ic,
            return_value: -1,
        })
    }
    fn handle_preedit_draw(
        &mut self,
        _: &mut ClientConnection,
        _: u16,
        _: u16,
        caret: i32,
        first: i32,
        length: i32,
        _: xim::PreeditDrawStatus,
        text: &str,
        _: Vec<xim::Feedback>,
    ) -> Result<(), ClientError> {
        let start = byte(&self.preedit, first);
        let end = byte(&self.preedit, first.saturating_add(length.max(0)));
        self.preedit.replace_range(start..end.max(start), text);
        self.caret = caret.clamp(0, self.preedit.chars().count() as i32);
        let cursor = byte(&self.preedit, caret);
        self.events.push(ShellEvent::Preedit {
            text: self.preedit.clone(),
            selection: Some((cursor, cursor)),
        });
        Ok(())
    }
    fn handle_preedit_caret(
        &mut self,
        _: &mut ClientConnection,
        _: u16,
        _: u16,
        position: &mut i32,
        direction: xim::CaretDirection,
        _: xim::CaretStyle,
    ) -> Result<(), ClientError> {
        let length = self.preedit.chars().count() as i32;
        self.caret = match direction {
            xim::CaretDirection::ForwardChar => self.caret.saturating_add(1),
            xim::CaretDirection::BackwardChar => self.caret.saturating_sub(1),
            xim::CaretDirection::LineStart => 0,
            xim::CaretDirection::LineEnd => length,
            xim::CaretDirection::AbsolutePosition => *position,
            _ => self.caret,
        }
        .clamp(0, length);
        *position = self.caret;
        let cursor = byte(&self.preedit, self.caret);
        self.events.push(ShellEvent::Preedit {
            text: self.preedit.clone(),
            selection: Some((cursor, cursor)),
        });
        Ok(())
    }
    fn handle_preedit_done(
        &mut self,
        _: &mut ClientConnection,
        _: u16,
        _: u16,
    ) -> Result<(), ClientError> {
        self.preedit.clear();
        self.events.push(ShellEvent::Preedit {
            text: String::new(),
            selection: None,
        });
        Ok(())
    }
    // We never send Disconnect, so this is the server ending the connection.
    // The handler cannot degrade the Ime that owns it; `filter` does.
    fn handle_disconnect(&mut self) {
        self.disconnected = true;
    }
}
