//! Role: Runs XIM on the child's XCB connection, with XKB when no server exists.
//!
//! Invariants: at most one synchronous request is in flight and it waits at
//! most 500 ms for its acknowledgement; later native keys wait in a queue of at
//! most 1024 events. Returned keys reach Keyboard once and are never forwarded
//! again. Canceled raw releases are locally reconciled only against delivered
//! downs. A canceled flight remains a tombstone until its old acknowledgement
//! arrives. Only SyncReply is peeked here; normal transport handling and
//! protocol acknowledgements belong to the locked client. XIM is an optional
//! service: every failure of the server, the transport or the protocol ends in
//! `degrade`, which drops the client and gives the undelivered keys back to
//! Keyboard in order. Degrading drops the whole connection (`Xim::Off`) for the
//! rest of the window's life.

use super::conn::Native;
use super::event::WindowFocus;
use arvane_shell::ShellError;
use arvane_shell::pipeline::{ImePort, RawIme};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use x11rb::connection::Connection as _;
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, KeyPressEvent};
use x11rb::x11_utils::X11Error;
use x11rb::xcb_ffi::XCBConnection;
use xim::x11rb::{HasConnection, X11rbClient};
use xim::{
    AttributeName, Client, ClientCore, ClientError, ClientHandler, ForwardEventFlag, InputStyle,
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

pub(super) struct Ime {
    xim: Xim,
    app: App,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum ImeAllowance {
    Disallowed,
    Allowed,
}

/// What the App asked for and what goes back to the window loop; kept across XIM's life.
struct App {
    window: u32,
    locale: String,
    allowed: ImeAllowance,
    focused: WindowFocus,
    spot: xim::Point,
    events: Vec<RawIme>,
    returned: Vec<KeyPressEvent>,
    cancelled_releases: Vec<KeyPressEvent>,
}
enum Xim {
    /// No XIM server, or XIM was given up: keys go to Keyboard.
    Off,
    On(Box<XimConnection>),
}
// Transport establishment is separate from opening the input method and its context.
enum Connection {
    Handshake {
        protocol_window: u32,
    },
    Established {
        protocol_window: u32,
        server_window: u32,
    },
}
impl Connection {
    fn protocol_window(&self) -> u32 {
        match self {
            Self::Handshake { protocol_window }
            | Self::Established {
                protocol_window, ..
            } => *protocol_window,
        }
    }
    fn server_window(&self) -> Option<u32> {
        match self {
            Self::Handshake { .. } => None,
            Self::Established { server_window, .. } => Some(*server_window),
        }
    }
}
struct XimConnection {
    client: ClientConnection,
    protocol: u32,
    xconnect: u32,
    phase: Connection,
    session: Session,
}
/// What the server's callbacks change.
struct Session {
    context: Context,
    forward: u32,
    preedit: String,
    caret: i32,
}
enum Context {
    /// Connected; the input method is not open yet.
    Connecting,
    Open,
    Ready {
        im: u16,
        ic: u16,
        keys: Keys,
    },
    /// The server ended the connection (`handle_disconnect`); `filter` degrades.
    Disconnected {
        keys: Keys,
    },
}
/// The keys of a ready context: at most one synchronous forward waits for its
/// acknowledgement, and later keys wait behind it.
#[derive(Default)]
enum Keys {
    /// No forward waits and no key waits behind one.
    #[default]
    Idle,
    /// A forward waits for its acknowledgement; `pending` waits behind it.
    Waiting {
        flight: Flight,
        pending: VecDeque<(KeyPressEvent, bool)>,
    },
}
impl Keys {
    fn flight(&self) -> Option<&Flight> {
        match self {
            Self::Idle => None,
            Self::Waiting { flight, .. } => Some(flight),
        }
    }
    fn pending_len(&self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Waiting { pending, .. } => pending.len(),
        }
    }
    fn push(&mut self, key: (KeyPressEvent, bool)) {
        if let Self::Waiting { pending, .. } = self {
            pending.push_back(key);
        }
    }
    fn into_parts(self) -> (Option<Flight>, VecDeque<(KeyPressEvent, bool)>) {
        match self {
            Self::Idle => (None, VecDeque::new()),
            Self::Waiting { flight, pending } => (Some(flight), pending),
        }
    }
}
impl Context {
    /// The input method and context ids, once the context exists.
    fn ids(&self) -> Option<(u16, u16)> {
        match self {
            Self::Ready { im, ic, .. } => Some((*im, *ic)),
            Self::Connecting | Self::Open | Self::Disconnected { .. } => None,
        }
    }
    /// The keys a ready context keeps, or the keys of a disconnected one.
    fn keys(&self) -> Option<&Keys> {
        match self {
            Self::Ready { keys, .. } | Self::Disconnected { keys } => Some(keys),
            Self::Connecting | Self::Open => None,
        }
    }
    fn keys_mut(&mut self) -> Option<&mut Keys> {
        match self {
            Self::Ready { keys, .. } | Self::Disconnected { keys } => Some(keys),
            Self::Connecting | Self::Open => None,
        }
    }
    /// Replace the keys of a context that keeps them (ready or disconnected).
    fn set_keys(&mut self, next: Keys) {
        if let Some(keys) = self.keys_mut() {
            *keys = next;
        }
    }
    /// Take the keys out of a context that keeps them (ready or disconnected).
    fn take_keys(&mut self) -> Keys {
        self.keys_mut().map_or(Keys::Idle, std::mem::take)
    }
    /// The flight and the keys behind it, for a context that is being dropped.
    fn into_keys(self) -> (Option<Flight>, VecDeque<(KeyPressEvent, bool)>) {
        match self {
            Self::Ready { keys, .. } | Self::Disconnected { keys } => keys.into_parts(),
            Self::Connecting | Self::Open => (None, VecDeque::new()),
        }
    }
}
enum Flight {
    /// A forwarded key waiting for its acknowledgement.
    Live {
        im: u16,
        ic: u16,
        event: KeyPressEvent,
        down: bool,
        deadline: Instant,
    },
    /// Canceled by a focus or allow change; kept until its old acknowledgement arrives.
    Canceled { im: u16, ic: u16, deadline: Instant },
}
impl Flight {
    fn ids(&self) -> (u16, u16) {
        match self {
            Self::Live { im, ic, .. } | Self::Canceled { im, ic, .. } => (*im, *ic),
        }
    }
    fn deadline(&self) -> Instant {
        match self {
            Self::Live { deadline, .. } | Self::Canceled { deadline, .. } => *deadline,
        }
    }
}
/// The handler handed to the client for one `filter_event` call.
struct Callbacks<'a> {
    app: &'a mut App,
    session: &'a mut Session,
}
impl Ime {
    /// IME is optional: when the server cannot be reached the window opens
    /// without it.
    pub(super) fn new(native: &Native) -> RefCell<Self> {
        let xim = match connect(native) {
            Ok(Some(connection)) => Xim::On(Box::new(connection)),
            Ok(None) => Xim::Off,
            Err(error) => {
                tracing::warn!(target: "kivact", "X11 IME unavailable: {error}");
                Xim::Off
            }
        };
        RefCell::new(Self {
            xim,
            app: App {
                window: native.id.get(),
                locale: locale(),
                allowed: ImeAllowance::Disallowed,
                focused: WindowFocus::Blurred,
                spot: xim::Point { x: 0, y: 0 },
                events: Vec::new(),
                returned: Vec::new(),
                cancelled_releases: Vec::new(),
            },
        })
    }
    /// Gives up XIM for the rest of the window's life: warns once, drops the
    /// connection, reports IME disabled and hands the in-flight and queued keys to
    /// Keyboard in their original order.
    pub(super) fn degrade(&mut self, reason: &str) {
        self.degrade_with(reason, VecDeque::new());
    }
    /// [`Ime::degrade`], with `extra` keys (not yet in the connection) going
    /// back to Keyboard after the flight and before the queued keys.
    fn degrade_with(&mut self, reason: &str, extra: VecDeque<(KeyPressEvent, bool)>) {
        let Xim::On(connection) = std::mem::replace(&mut self.xim, Xim::Off) else {
            return;
        };
        tracing::warn!(target: "kivact", "X11 IME disabled, using the keyboard directly: {reason}");
        self.app.events.push(RawIme::End);
        let XimConnection { session, .. } = *connection;
        let (flight, pending) = session.context.into_keys();
        if let Some(Flight::Live { event, .. }) = flight
            // A server that already returned the key must not have it twice.
            && !self.app.returned.iter().any(|key| same_key(key, &event))
        {
            self.app.returned.push(event);
        }
        self.app
            .returned
            .extend(extra.into_iter().map(|(event, _)| event));
        self.app
            .returned
            .extend(pending.into_iter().map(|(event, _)| event));
    }
    /// A lost XIM server surfaces as an X error on its window.
    pub(super) fn x_error(&mut self, error: &X11Error) {
        let Xim::On(connection) = &self.xim else {
            return;
        };
        if error.error_kind == x11rb::protocol::ErrorKind::Window
            && connection.phase.server_window() == Some(error.bad_value)
        {
            self.degrade("XIM server window is gone");
        }
    }
    /// Nanoseconds until the in-flight request is given up, if one is waiting.
    pub(super) fn remaining_ns(&self) -> Option<u64> {
        let Xim::On(connection) = &self.xim else {
            return None;
        };
        let flight = connection.session.context.keys()?.flight()?;
        let remaining = flight.deadline().saturating_duration_since(Instant::now());
        Some(u64::try_from(remaining.as_nanos()).unwrap_or(u64::MAX))
    }
    pub(super) fn filter(&mut self, event: &Event) -> bool {
        let Xim::On(connection) = &mut self.xim else {
            return false;
        };
        let observe = connection
            .session
            .context
            .keys()
            .is_some_and(|keys| keys.flight().is_some())
            && matches!(event, Event::ClientMessage(message)
            if message.window == connection.phase.protocol_window());
        let (reply, filtered) = match feed(connection, &mut self.app, event, observe) {
            Ok(parts) => parts,
            Err(error) => {
                self.degrade(&error.to_string());
                return false;
            }
        };
        if matches!(connection.session.context, Context::Disconnected { .. }) {
            self.degrade("XIM server disconnected");
            return filtered;
        }
        if filtered && let Event::ClientMessage(message) = event {
            let server_window = if message.type_ == connection.xconnect && message.format == 32 {
                Some(message.data.as_data32()[0])
            } else {
                connection.phase.server_window()
            };
            connection.phase = match server_window {
                Some(server_window) => Connection::Established {
                    protocol_window: message.window,
                    server_window,
                },
                None => Connection::Handshake {
                    protocol_window: message.window,
                },
            };
        }
        // A canceled request's delayed output must not edit the newly focused input.
        if connection
            .session
            .context
            .keys()
            .is_some_and(|keys| matches!(keys.flight(), Some(Flight::Canceled { .. })))
        {
            self.app.events.retain(|event| matches!(event, RawIme::End));
            // Known canceled releases were reconciled at cancellation time.
            // Delayed output must not release a newer local down of that key.
            self.app.returned.clear();
            connection.session.preedit.clear();
        }
        if filtered
            && let Some(replied) = reply
            && connection
                .session
                .context
                .keys()
                .and_then(Keys::flight)
                .is_some_and(|flight| flight.ids() == replied)
        {
            let (_, pending) = connection.session.context.take_keys().into_parts();
            connection.session.context.set_keys(Keys::Idle);
            self.advance(pending);
        }
        filtered
    }
    /// Whether XIM took the key; a refused key goes to Keyboard directly.
    pub(super) fn forward(&mut self, event: &KeyPressEvent, down: bool) -> bool {
        let Xim::On(connection) = &mut self.xim else {
            return false;
        };
        if connection.session.context.ids().is_none()
            || self.app.allowed == ImeAllowance::Disallowed
            || self.app.focused == WindowFocus::Blurred
        {
            return false;
        }
        let mask = if down { 1 } else { 2 };
        let Some(keys) = connection.session.context.keys() else {
            return false;
        };
        if keys.flight().is_none()
            && keys.pending_len() == 0
            && connection.session.forward & mask == 0
        {
            return false;
        }
        if keys.pending_len() >= 1024 {
            // The new key follows the queued ones it was waiting behind.
            self.degrade("XIM pending key queue exceeded 1024 events");
            self.app.returned.push(*event);
            return true;
        }
        if keys.flight().is_some() {
            // A forward waits: the key waits behind it.
            if let Some(keys) = connection.session.context.keys_mut() {
                keys.push((*event, down));
            }
            return true;
        }
        self.advance(VecDeque::from([(*event, down)]));
        true
    }
    /// Start synchronous forwards from `pending` while no flight waits: keys
    /// whose event mask is not forwarded go back to Keyboard in order.
    fn advance(&mut self, mut pending: VecDeque<(KeyPressEvent, bool)>) {
        loop {
            let Xim::On(connection) = &mut self.xim else {
                return;
            };
            let Some((event, down)) = pending.pop_front() else {
                return;
            };
            let ids = connection.session.context.ids().filter(|_| {
                self.app.allowed == ImeAllowance::Allowed
                    && self.app.focused == WindowFocus::Focused
            });
            let Some((im, ic)) = ids else {
                return;
            };
            let mask = if down { 1 } else { 2 };
            if connection.session.forward & mask == 0 {
                // Local releases wait behind the forwarded press, then use the
                // same returned-key path without ever being sent back to XIM.
                self.app.returned.push(event);
                continue;
            }
            let client = &mut connection.client;
            let mut forward = || -> Result<(), ClientError> {
                client.forward_event(im, ic, ForwardEventFlag::SYNCHRONOUS, &event)?;
                client.conn().flush()?;
                Ok(())
            };
            match forward() {
                Ok(()) => {
                    connection.session.context.set_keys(Keys::Waiting {
                        flight: Flight::Live {
                            im,
                            ic,
                            event,
                            down,
                            deadline: Instant::now() + FLIGHT_TIMEOUT,
                        },
                        pending,
                    });
                    return;
                }
                Err(error) => {
                    pending.push_front((event, down));
                    self.degrade_with(&error.to_string(), pending);
                    return;
                }
            }
        }
    }
    fn collect_returned_releases(&mut self) {
        self.app
            .cancelled_releases
            .extend(self.app.returned.drain(..).filter(|event| {
                event.response_type & 0x7f == x11rb::protocol::xproto::KEY_RELEASE_EVENT
            }));
    }
    fn cancel_pending(&mut self) {
        // The in-flight event precedes the queued events; preserve release
        // order so their native modifier masks are applied in the same order.
        let mut queued = VecDeque::new();
        if let Xim::On(connection) = &mut self.xim {
            let (flight, pending) = connection.session.context.take_keys().into_parts();
            let flight = match flight {
                Some(Flight::Live {
                    im,
                    ic,
                    event,
                    down,
                    deadline,
                }) => {
                    if !down {
                        self.app.cancelled_releases.push(event);
                    }
                    Some(Flight::Canceled { im, ic, deadline })
                }
                other => other,
            };
            connection.session.context.set_keys(match flight {
                Some(flight) => Keys::Waiting {
                    flight,
                    pending: VecDeque::new(),
                },
                None => Keys::Idle,
            });
            queued = pending;
        }
        self.collect_returned_releases();
        self.app.cancelled_releases.extend(
            queued
                .into_iter()
                .filter_map(|(event, down)| (!down).then_some(event)),
        );
        if let Xim::On(connection) = &mut self.xim {
            connection.session.preedit.clear();
        }
        self.app.events.retain(|event| matches!(event, RawIme::End));
    }
    pub(super) fn drain(&mut self) -> (Vec<RawIme>, Vec<KeyPressEvent>, Vec<KeyPressEvent>) {
        // The loop drains once per pass and wakes for the flight deadline, so
        // a reply that never comes is noticed here.
        if let Xim::On(connection) = &self.xim
            && connection
                .session
                .context
                .keys()
                .and_then(Keys::flight)
                .is_some_and(|flight| Instant::now() >= flight.deadline())
        {
            self.degrade("XIM server did not answer");
        }
        (
            std::mem::take(&mut self.app.events),
            std::mem::take(&mut self.app.returned),
            std::mem::take(&mut self.app.cancelled_releases),
        )
    }
    pub(super) fn focus(&mut self, focused: bool) {
        let focused = if focused {
            WindowFocus::Focused
        } else {
            WindowFocus::Blurred
        };
        if self.app.focused == focused {
            return;
        }
        if focused == WindowFocus::Blurred {
            self.cancel_pending();
        }
        self.app.focused = focused;
        self.sync_focus();
    }
    // A broken server is degraded away rather than failing the window.
    pub(super) fn allow(&mut self, allowed: bool) {
        let allowed = if allowed {
            ImeAllowance::Allowed
        } else {
            ImeAllowance::Disallowed
        };
        if allowed == ImeAllowance::Disallowed {
            self.cancel_pending();
        }
        self.app.allowed = allowed;
        self.sync_focus();
    }
    fn sync_focus(&mut self) {
        let Xim::On(connection) = &mut self.xim else {
            return;
        };
        let Some((im, ic)) = connection.session.context.ids() else {
            return;
        };
        let on =
            self.app.allowed == ImeAllowance::Allowed && self.app.focused == WindowFocus::Focused;
        if !on {
            connection.session.preedit.clear();
            self.app.events.push(RawIme::End);
        }
        let client = &mut connection.client;
        let mut focus = || -> Result<(), ClientError> {
            if on {
                client.set_focus(im, ic)?;
            } else {
                client.unset_focus(im, ic)?;
            }
            client.conn().flush()?;
            Ok(())
        };
        if let Err(error) = focus() {
            self.degrade(&error.to_string());
        }
    }
    // A broken server is degraded away, as in `allow`.
    pub(super) fn cursor(&mut self, area: kivact_types::Rect, scale: f64) {
        let spot = xim::Point {
            x: (area.x * scale).clamp(i16::MIN as f64, i16::MAX as f64) as i16,
            y: ((area.y + area.height) * scale).clamp(i16::MIN as f64, i16::MAX as f64) as i16,
        };
        self.app.spot = spot.clone();
        let Xim::On(connection) = &mut self.xim else {
            return;
        };
        let Some((im, ic)) = connection.session.context.ids() else {
            return;
        };
        let client = &mut connection.client;
        let mut place = || -> Result<(), ClientError> {
            let attrs = client
                .build_ic_attributes()
                .nested_list(AttributeName::PreeditAttributes, |b| {
                    b.push(AttributeName::SpotLocation, spot.clone());
                })
                .build();
            client.set_ic_values(im, ic, attrs)?;
            client.conn().flush()?;
            Ok(())
        };
        if let Err(error) = place() {
            self.degrade(&error.to_string());
        }
    }
}
fn same_key(a: &KeyPressEvent, b: &KeyPressEvent) -> bool {
    a.detail == b.detail && a.time == b.time && a.response_type & 0x7f == b.response_type & 0x7f
}
/// Hand one X event to the client; the observed SyncReply (if any) and whether the client took it.
fn feed(
    connection: &mut XimConnection,
    app: &mut App,
    event: &Event,
    observe: bool,
) -> Result<(Option<(u16, u16)>, bool), ClientError> {
    let reply = if observe {
        sync_reply(&connection.client, connection.protocol, event)?
    } else {
        None
    };
    let mut callbacks = Callbacks {
        app,
        session: &mut connection.session,
    };
    let filtered = connection.client.filter_event(event, &mut callbacks)?;
    Ok((reply, filtered))
}
/// The connection with its protocol atoms, or `None` when no server is announced.
fn connect(native: &Native) -> Result<Option<XimConnection>, ShellError> {
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
        return Ok(None);
    }
    let client = match ClientConnection::init(connection, native.screen as usize, None) {
        Ok(client) => client,
        Err(ClientError::NoXimServer) => return Ok(None),
        Err(error) => return Err(super::error::platform(error)),
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
    Ok(Some(XimConnection {
        client,
        protocol,
        xconnect,
        phase: Connection::Handshake {
            // Zero is not an X window; no protocol message has been observed yet.
            protocol_window: 0,
        },
        session: Session {
            context: Context::Connecting,
            forward: u32::from(x11rb::protocol::xproto::EventMask::KEY_PRESS),
            preedit: String::new(),
            caret: 0,
        },
    }))
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
impl ClientHandler<ClientConnection> for Callbacks<'_> {
    fn handle_connect(&mut self, client: &mut ClientConnection) -> Result<(), ClientError> {
        client.open(&self.app.locale)
    }
    fn handle_open(&mut self, client: &mut ClientConnection, im: u16) -> Result<(), ClientError> {
        self.session.context = Context::Open;
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
            .push(AttributeName::ClientWindow, self.app.window)
            .push(AttributeName::FocusWindow, self.app.window)
            .nested_list(AttributeName::PreeditAttributes, |b| {
                b.push(AttributeName::SpotLocation, self.app.spot.clone());
            })
            .build();
        client.create_ic(im, attrs)
    }
    fn handle_create_ic(
        &mut self,
        client: &mut ClientConnection,
        im: u16,
        ic: u16,
    ) -> Result<(), ClientError> {
        self.session.context = Context::Ready {
            im,
            ic,
            keys: Keys::Idle,
        };
        if self.app.allowed == ImeAllowance::Allowed && self.app.focused == WindowFocus::Focused {
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
        self.session.forward = forward;
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
        self.app.returned.push(event);
        Ok(())
    }
    fn handle_commit(
        &mut self,
        _: &mut ClientConnection,
        _: u16,
        _: u16,
        text: &str,
    ) -> Result<(), ClientError> {
        self.session.preedit.clear();
        self.app.events.push(RawIme::Commit(text.into()));
        Ok(())
    }
    fn handle_preedit_start(
        &mut self,
        client: &mut ClientConnection,
        im: u16,
        ic: u16,
    ) -> Result<(), ClientError> {
        self.session.preedit.clear();
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
        let session = &mut *self.session;
        let start = byte(&session.preedit, first);
        let end = byte(&session.preedit, first.saturating_add(length.max(0)));
        session.preedit.replace_range(start..end.max(start), text);
        session.caret = caret.clamp(0, session.preedit.chars().count() as i32);
        let cursor = byte(&session.preedit, caret);
        self.app.events.push(RawIme::Preedit {
            text: session.preedit.clone(),
            caret: Some((cursor, cursor)),
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
        let session = &mut *self.session;
        let length = session.preedit.chars().count() as i32;
        session.caret = match direction {
            xim::CaretDirection::ForwardChar => session.caret.saturating_add(1),
            xim::CaretDirection::BackwardChar => session.caret.saturating_sub(1),
            xim::CaretDirection::LineStart => 0,
            xim::CaretDirection::LineEnd => length,
            xim::CaretDirection::AbsolutePosition => *position,
            _ => session.caret,
        }
        .clamp(0, length);
        *position = session.caret;
        let cursor = byte(&session.preedit, session.caret);
        self.app.events.push(RawIme::Preedit {
            text: session.preedit.clone(),
            caret: Some((cursor, cursor)),
        });
        Ok(())
    }
    fn handle_preedit_done(
        &mut self,
        _: &mut ClientConnection,
        _: u16,
        _: u16,
    ) -> Result<(), ClientError> {
        self.session.preedit.clear();
        self.app.events.push(RawIme::Preedit {
            text: String::new(),
            caret: None,
        });
        Ok(())
    }
    // We never send Disconnect, so this is the server ending the connection.
    // The handler cannot degrade the Ime that owns it; `filter` does.
    fn handle_disconnect(&mut self) {
        let context = std::mem::replace(&mut self.session.context, Context::Connecting);
        self.session.context = match context {
            Context::Ready { keys, .. } | Context::Disconnected { keys } => {
                Context::Disconnected { keys }
            }
            // No context existed: no key could be queued without its ids.
            Context::Connecting | Context::Open => Context::Disconnected { keys: Keys::Idle },
        };
    }
}

pub(super) struct Port<'a> {
    pub(super) ime: &'a mut Ime,
    pub(super) scale: f64,
}
impl ImePort for Port<'_> {
    fn allow(&mut self, allowed: bool) {
        self.ime.allow(allowed);
    }
    fn cursor_area(&mut self, area: kivact_types::Rect) {
        self.ime.cursor(area, self.scale);
    }
    fn surrounding(&mut self, _: Option<&arvane_shell::ShellImeText>) {}
}
