//! Role: One connection: its handshake, forwarded calls and cleanup.

use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use serde_json::Value;

use kivact_devtools_protocol::{
    Call, Empty, Initialize, InitializeResult, Method, PROTOCOL_VERSION, Reply, RpcError,
    RpcRequest, code, parse_call, parse_request,
};

use crate::incoming::{Incoming, Responder};
use crate::session::{ConnectionId, Peer, Session, Shared, Writer, shutdown};

/// A connection that has not sent `initialize` within this time after it was accepted is closed.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);
/// The longest line before `initialize`: a handshake is small.
const MAX_HANDSHAKE_LINE: usize = 64 * 1024;
const MAX_LINE: usize = 64 * 1024 * 1024;

/// What one line did to its connection.
enum Step {
    Continue,
    Initialized,
    Close,
}

/// Serve one connection. `accepted` is when the listener accepted it: the handshake deadline counts from there.
pub(crate) fn run(
    stream: TcpStream,
    id: ConnectionId,
    shared: Arc<Shared>,
    incoming: mpsc::Sender<Incoming>,
    accepted: Instant,
) {
    if stream.set_write_timeout(Some(WRITE_TIMEOUT)).is_err() {
        return;
    }
    let Ok(clone) = stream.try_clone() else {
        return;
    };
    let writer: Writer = Arc::new(Mutex::new(clone));
    let mut reader = BufReader::new(stream);
    let mut session = Session::AwaitingInitialize;
    loop {
        let awaiting = matches!(session, Session::AwaitingInitialize);
        if awaiting && !arm_handshake_read(reader.get_ref(), accepted) {
            break;
        }
        let line = match read_line(&mut reader, line_cap(&session)) {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(error) => {
                if error.kind() == std::io::ErrorKind::InvalidData {
                    reply(
                        &writer,
                        Value::Null,
                        Err(RpcError::new(code::INVALID_REQUEST, error.to_string())),
                    );
                }
                break;
            }
        };
        if awaiting && accepted.elapsed() >= HANDSHAKE_TIMEOUT {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        match handle_line(&line, &session, id, &writer, &shared, &incoming) {
            Step::Continue => {}
            Step::Initialized => {
                session = Session::Ready;
                let _ = reader.get_ref().set_read_timeout(None);
            }
            Step::Close => break,
        }
    }
    finish(id, &session, &writer, &shared, &incoming);
}

/// The longest line the session accepts: the handshake cap before `initialize`, `MAX_LINE` after it.
fn line_cap(session: &Session) -> usize {
    match session {
        Session::AwaitingInitialize => MAX_HANDSHAKE_LINE,
        Session::Ready => MAX_LINE,
    }
}

/// Limit the next read to the time left of the handshake. `false` when the handshake deadline has passed.
fn arm_handshake_read(stream: &TcpStream, accepted: Instant) -> bool {
    match HANDSHAKE_TIMEOUT
        .checked_sub(accepted.elapsed())
        .filter(|left| !left.is_zero())
    {
        Some(left) => stream.set_read_timeout(Some(left)).is_ok(),
        None => false,
    }
}

/// One line of at most `max` bytes; `Ok(None)` at the end of the stream; a longer line is `InvalidData`.
fn read_line(reader: &mut BufReader<TcpStream>, max: usize) -> std::io::Result<Option<String>> {
    let mut bytes = Vec::new();
    let read = reader
        .by_ref()
        .take(max as u64 + 1)
        .read_until(b'\n', &mut bytes)?;
    if read == 0 {
        return Ok(None);
    }
    if bytes.len() > max {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("line longer than {max} bytes"),
        ));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

fn reply(writer: &Writer, id: Value, result: Result<Reply, RpcError>) {
    Responder::new(id, Arc::clone(writer)).send(result);
}

fn handle_line(
    line: &str,
    session: &Session,
    id: ConnectionId,
    writer: &Writer,
    shared: &Shared,
    incoming: &mpsc::Sender<Incoming>,
) -> Step {
    let request = match parse_request(line) {
        Ok(request) => request,
        Err(error) => {
            reply(writer, Value::Null, Err(error));
            return Step::Continue;
        }
    };
    match session {
        Session::AwaitingInitialize => handshake(&request, id, writer, shared),
        Session::Ready => serve(request, id, writer, shared, incoming),
    }
}

/// Before `initialize`: only `initialize` is answered (token first, then the protocol version).
fn handshake(request: &RpcRequest, id: ConnectionId, writer: &Writer, shared: &Shared) -> Step {
    let params = match parse_call(request) {
        Ok(Call::Initialize(params)) => params,
        Err(error) if request.method == Initialize::NAME => {
            reply(writer, request.id.clone(), Err(error));
            return Step::Continue;
        }
        _ => {
            let error = RpcError::new(code::NOT_INITIALIZED, "call initialize first");
            reply(writer, request.id.clone(), Err(error));
            return Step::Continue;
        }
    };
    if params.token != shared.token {
        reply(
            writer,
            request.id.clone(),
            Err(RpcError::new(code::BAD_TOKEN, "bad token")),
        );
        return Step::Close;
    }
    if params.protocol != PROTOCOL_VERSION {
        let message = format!(
            "protocol version mismatch: target {PROTOCOL_VERSION}, client {}",
            params.protocol
        );
        reply(
            writer,
            request.id.clone(),
            Err(RpcError::new(code::VERSION_MISMATCH, message)),
        );
        return Step::Close;
    }
    shared.peers.insert(
        id,
        Peer {
            client: params.client,
            topics: Vec::new(),
            writer: Arc::clone(writer),
        },
    );
    let result = InitializeResult {
        protocol: PROTOCOL_VERSION,
        app: shared.app.clone(),
    };
    reply(writer, request.id.clone(), Ok(Reply::Initialize(result)));
    Step::Initialized
}

/// After `initialize`: `subscribe` and `get_config` are answered here; every other call goes to
/// the UI thread.
fn serve(
    request: RpcRequest,
    id: ConnectionId,
    writer: &Writer,
    shared: &Shared,
    incoming: &mpsc::Sender<Incoming>,
) -> Step {
    match parse_call(&request) {
        Err(error) => reply(writer, request.id, Err(error)),
        Ok(Call::Initialize(_)) => {
            reply(
                writer,
                request.id,
                Err(RpcError::new(code::INVALID_REQUEST, "already initialized")),
            );
        }
        Ok(Call::Subscribe(params)) => {
            shared.peers.set_topics(id, params.topics);
            reply(writer, request.id, Ok(Reply::Subscribe(Empty {})));
        }
        Ok(Call::GetConfig(_)) => {
            reply(
                writer,
                request.id,
                Ok(Reply::GetConfig(shared.config.with_defaults())),
            );
        }
        Ok(call) => {
            let responder = Responder::new(request.id, Arc::clone(writer));
            if incoming.send(Incoming::Call(call, responder)).is_err() {
                return Step::Close;
            }
            (shared.wake)();
        }
    }
    Step::Continue
}

/// End of the connection (EOF, a failed read or write, or `Close`): drop the peer and tell the core.
fn finish(
    id: ConnectionId,
    session: &Session,
    writer: &Writer,
    shared: &Shared,
    incoming: &mpsc::Sender<Incoming>,
) {
    shutdown(writer);
    if matches!(session, Session::Ready) {
        let remaining = shared.peers.remove(id);
        let _ = incoming.send(Incoming::Closed { remaining });
        (shared.wake)();
    }
}
