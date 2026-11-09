//! Role: Tests of the DevTools client against a fake agent in another thread.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use kivact_devtools_client::{ClientError, Connection};
use kivact_devtools_protocol::{
    AppInfo, BuildKind, ClientInfo, ClientKind, DiscoveryFile, Empty, GetAppInfo, InitializeResult,
    Notification, PROTOCOL_VERSION, RpcRequest, notification_line, parse_request, response,
    response_line,
};

fn app_info() -> AppInfo {
    AppInfo {
        name: "test".to_owned(),
        pid: 12,
        kivact_version: "0.1.0".to_owned(),
        build: BuildKind::Debug,
        exe: "test.exe".to_owned(),
        cwd: "C:/work".to_owned(),
        workspace_root: "C:/work".to_owned(),
    }
}

fn client_info() -> ClientInfo {
    ClientInfo {
        name: "test-client".to_owned(),
        version: "0.1.0".to_owned(),
        kind: ClientKind::Python,
    }
}

fn discovery_file(port: u16) -> DiscoveryFile {
    DiscoveryFile {
        protocol: PROTOCOL_VERSION,
        pid: 12,
        app_name: "test".to_owned(),
        kivact_version: "0.1.0".to_owned(),
        port: u32::from(port),
        token: "secret".to_owned(),
        started_unix_ms: 0,
        workspace_root: "C:/work".to_owned(),
    }
}

fn read_request(reader: &mut BufReader<TcpStream>) -> RpcRequest {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("invariant: the fake agent reads the request line");
    parse_request(line.trim_end()).expect("invariant: the client sends a valid request")
}

fn respond(writer: &mut TcpStream, id: serde_json::Value, result: serde_json::Value) {
    let line = response_line(&response(id, Ok(result)));
    writer
        .write_all(line.as_bytes())
        .expect("invariant: the fake agent writes its response");
    writer
        .write_all(b"\n")
        .expect("invariant: the fake agent writes its response");
    writer
        .flush()
        .expect("invariant: the fake agent flushes its response");
}

fn start_agent(
    protocol: u32,
    after: impl FnOnce(&mut BufReader<TcpStream>, &mut TcpStream) + Send + 'static,
) -> (u16, JoinHandle<()>) {
    let listener =
        TcpListener::bind("127.0.0.1:0").expect("invariant: the fake agent can bind a port");
    let port = listener
        .local_addr()
        .expect("invariant: a bound listener has an address")
        .port();
    let handle = thread::spawn(move || {
        let (stream, _) = listener
            .accept()
            .expect("invariant: the client connects to the fake agent");
        let mut writer = stream
            .try_clone()
            .expect("invariant: the fake agent can clone its stream");
        let mut reader = BufReader::new(stream);
        let request = read_request(&mut reader);
        let result = InitializeResult {
            protocol,
            app: app_info(),
        };
        let value =
            serde_json::to_value(result).expect("invariant: the initialize result serializes");
        respond(&mut writer, request.id.clone(), value);
        after(&mut reader, &mut writer);
    });
    (port, handle)
}

#[test]
fn open_and_call() {
    let (port, handle) = start_agent(PROTOCOL_VERSION, |reader, writer| {
        let request = read_request(reader);
        assert_eq!(request.method, "get_app_info");
        respond(
            writer,
            request.id.clone(),
            serde_json::to_value(app_info()).unwrap(),
        );
    });
    let (connection, result) = Connection::open(&discovery_file(port), client_info()).unwrap();
    assert_eq!(result.protocol, PROTOCOL_VERSION);
    let info = connection.call::<GetAppInfo>(Empty {}).unwrap();
    assert_eq!(info.name, "test");
    handle.join().unwrap();
}

#[test]
fn a_closed_agent_closes_the_connection() {
    let (port, handle) = start_agent(PROTOCOL_VERSION, |_reader, _writer| {});
    let (connection, _) = Connection::open(&discovery_file(port), client_info()).unwrap();
    handle.join().unwrap();
    assert_eq!(
        connection.next_notification(Duration::from_secs(5)),
        Err(ClientError::Closed)
    );
    let started = Instant::now();
    assert_eq!(
        connection.call::<GetAppInfo>(Empty {}),
        Err(ClientError::Closed)
    );
    assert!(started.elapsed() < Duration::from_secs(1));
}

#[test]
fn a_different_protocol_version_is_rejected() {
    let (port, handle) = start_agent(99, |_reader, _writer| {});
    let error = match Connection::open(&discovery_file(port), client_info()) {
        Ok(_) => panic!("expected a protocol error"),
        Err(error) => error,
    };
    assert!(matches!(error, ClientError::Protocol(_)));
    handle.join().unwrap();
}

#[test]
fn notifications_are_delivered() {
    let (release, wait) = mpsc::channel::<()>();
    let (port, handle) = start_agent(PROTOCOL_VERSION, move |_reader, writer| {
        let notification = Notification::FocusRequested(Empty {});
        let line = notification_line(&notification);
        writer.write_all(line.as_bytes()).unwrap();
        writer.write_all(b"\n").unwrap();
        writer.flush().unwrap();
        let _ = wait.recv();
    });
    let (connection, _) = Connection::open(&discovery_file(port), client_info()).unwrap();
    assert_eq!(
        connection
            .next_notification(Duration::from_secs(5))
            .unwrap(),
        Some(Notification::FocusRequested(Empty {}))
    );
    assert_eq!(
        connection
            .next_notification(Duration::from_millis(50))
            .unwrap(),
        None
    );
    let _ = release.send(());
    handle.join().unwrap();
}

#[test]
fn connection_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Connection>();
}
