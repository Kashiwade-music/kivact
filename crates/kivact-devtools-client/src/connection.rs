//! Role: One connection to a DevTools agent: typed calls and notifications.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use kivact_devtools_protocol::{
    ClientInfo, DiscoveryFile, Initialize, InitializeParams, InitializeResult, Message, Method,
    Notification, PROTOCOL_VERSION, RpcResponse, parse_message, request_line,
};

use crate::error::{ClientError, io_error};

const CALL_TIMEOUT: Duration = Duration::from_secs(30);

type Answer = Result<serde_json::Value, ClientError>;

/// The calls waiting for an answer, or why the connection ended.
enum Calls {
    Open(HashMap<u64, mpsc::Sender<Answer>>),
    Closed(ClientError),
}

/// A connection to one agent. `Send + Sync`: the DevTools app shares it as `Arc<Connection>`.
pub struct Connection {
    writer: Mutex<TcpStream>,
    calls: Arc<Mutex<Calls>>,
    next_id: AtomicU64,
    notifications: Mutex<mpsc::Receiver<Notification>>,
}

impl Connection {
    /// Connect to the agent of `file`, send `initialize` and check the protocol version.
    pub fn open(
        file: &DiscoveryFile,
        client: ClientInfo,
    ) -> Result<(Connection, InitializeResult), ClientError> {
        let port = u16::try_from(file.port)
            .map_err(|_| ClientError::Protocol("port out of range".to_owned()))?;
        let stream = TcpStream::connect(("127.0.0.1", port)).map_err(io_error)?;
        stream.set_nodelay(true).map_err(io_error)?;
        stream
            .set_write_timeout(Some(CALL_TIMEOUT))
            .map_err(io_error)?;
        let reader = stream.try_clone().map_err(io_error)?;
        let calls = Arc::new(Mutex::new(Calls::Open(HashMap::new())));
        let (sender, receiver) = mpsc::channel::<Notification>();
        let thread_calls = Arc::clone(&calls);
        std::thread::Builder::new()
            .name("kivact-devtools-client".to_owned())
            .spawn(move || read_loop(reader, thread_calls, sender))
            .map_err(io_error)?;
        let connection = Connection {
            writer: Mutex::new(stream),
            calls,
            next_id: AtomicU64::new(1),
            notifications: Mutex::new(receiver),
        };
        let result = connection.call::<Initialize>(InitializeParams {
            protocol: PROTOCOL_VERSION,
            token: file.token.clone(),
            client,
        })?;
        if result.protocol != PROTOCOL_VERSION {
            return Err(ClientError::Protocol(format!(
                "protocol version mismatch: target {}, client {}",
                result.protocol, PROTOCOL_VERSION
            )));
        }
        Ok((connection, result))
    }

    /// Send one typed call and read its typed answer.
    pub fn call<M: Method>(&self, params: M::Params) -> Result<M::Result, ClientError> {
        let value = serde_json::to_value(params)
            .map_err(|error| ClientError::Protocol(error.to_string()))?;
        let answer = self.call_raw(M::NAME, value)?;
        serde_json::from_value(answer).map_err(|error| ClientError::Protocol(error.to_string()))
    }

    /// Send one call and read its raw answer.
    pub fn call_raw(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ClientError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::channel();
        {
            let mut calls = lock(&self.calls);
            match &mut *calls {
                Calls::Closed(error) => return Err(error.clone()),
                Calls::Open(map) => {
                    map.insert(id, sender);
                }
            }
        }
        if let Err(error) = self.send(&request_line(id, method, params)) {
            if let Calls::Open(map) = &mut *lock(&self.calls) {
                map.remove(&id);
            }
            return Err(error);
        }
        match receiver.recv_timeout(CALL_TIMEOUT) {
            Ok(answer) => answer,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if let Calls::Open(map) = &mut *lock(&self.calls) {
                    map.remove(&id);
                }
                Err(ClientError::Io("timed out".to_owned()))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(ClientError::Closed),
        }
    }

    /// The next notification, `Ok(None)` when nothing arrives within `timeout`.
    /// `Err` means the connection ended.
    pub fn next_notification(
        &self,
        timeout: Duration,
    ) -> Result<Option<Notification>, ClientError> {
        let receiver = self
            .notifications
            .lock()
            .expect("invariant: the lock is never poisoned");
        match receiver.recv_timeout(timeout) {
            Ok(notification) => Ok(Some(notification)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let reason = match &*lock(&self.calls) {
                    Calls::Closed(error) => error.clone(),
                    Calls::Open(_) => ClientError::Closed,
                };
                Err(reason)
            }
        }
    }

    fn send(&self, line: &str) -> Result<(), ClientError> {
        let mut writer = self
            .writer
            .lock()
            .expect("invariant: the lock is never poisoned");
        writer.write_all(line.as_bytes()).map_err(io_error)?;
        writer.write_all(b"\n").map_err(io_error)?;
        writer.flush().map_err(io_error)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let writer = self
            .writer
            .lock()
            .expect("invariant: the lock is never poisoned");
        let _ = writer.shutdown(Shutdown::Both);
    }
}

fn lock(calls: &Mutex<Calls>) -> MutexGuard<'_, Calls> {
    calls
        .lock()
        .expect("invariant: the calls lock is never poisoned")
}

fn read_loop(
    stream: TcpStream,
    calls: Arc<Mutex<Calls>>,
    notifications: mpsc::Sender<Notification>,
) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => deliver(&line, &calls, &notifications),
            Err(_) => break,
        }
    }
    close(&calls, ClientError::Closed);
}

fn deliver(line: &str, calls: &Mutex<Calls>, notifications: &mpsc::Sender<Notification>) {
    match parse_message(line.trim_end()) {
        Ok(Message::Notification(notification)) => {
            let _ = notifications.send(notification);
        }
        Ok(Message::Response(response)) => answer(calls, response),
        Err(_) => {}
    }
}

fn answer(calls: &Mutex<Calls>, response: RpcResponse) {
    let Some(id) = response.id.as_u64() else {
        return;
    };
    let sender = match &mut *lock(calls) {
        Calls::Open(map) => map.remove(&id),
        Calls::Closed(_) => None,
    };
    let Some(sender) = sender else {
        return;
    };
    let answer = match (response.error, response.result) {
        (Some(error), _) => Err(ClientError::Rpc(error)),
        (None, Some(value)) => Ok(value),
        (None, None) => Ok(serde_json::Value::Null),
    };
    let _ = sender.send(answer);
}

fn close(calls: &Mutex<Calls>, reason: ClientError) {
    let waiting = std::mem::replace(&mut *lock(calls), Calls::Closed(reason.clone()));
    if let Calls::Open(map) = waiting {
        for sender in map.values() {
            let _ = sender.send(Err(reason.clone()));
        }
    }
}
