//! Role: Tests of the agent against real client connections and its discovery file.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use kivact_devtools_agent::{Agent, AgentConfig, Incoming};
use kivact_devtools_protocol::client::{ClientError, Connection, read_target};
use kivact_devtools_protocol::{
    AppInfo, BuildKind, Call, ClientInfo, ClientKind, DiscoveryFile, Empty, ErrorEntry, GetAppInfo,
    GetConfig, InitializeParams, Notification, Reply, RpcError, RpcResponse, Subscribe,
    SubscribeParams, Topic, WindowList, code, request_line,
};

fn sample_app() -> AppInfo {
    AppInfo {
        name: "agent test".to_owned(),
        pid: u64::from(std::process::id()),
        kivact_version: "0.1.0".to_owned(),
        build: BuildKind::Debug,
        exe: String::new(),
        cwd: String::new(),
        workspace_root: String::new(),
    }
}

fn client_info(kind: ClientKind) -> ClientInfo {
    ClientInfo {
        name: "agent test".to_owned(),
        version: "0.1.0".to_owned(),
        kind,
    }
}

fn start_agent() -> Agent {
    Agent::start(AgentConfig { app: sample_app() }, Box::new(|| {}))
        .expect("invariant: the agent starts")
}

/// An agent whose wake sends on the returned channel, so a test waits for arrivals.
fn start_agent_woken() -> (Agent, mpsc::Receiver<()>) {
    let (sender, receiver) = mpsc::channel();
    let agent = Agent::start(
        AgentConfig { app: sample_app() },
        Box::new(move || {
            let _ = sender.send(());
        }),
    )
    .expect("invariant: the agent starts");
    (agent, receiver)
}

fn discovery(agent: &Agent) -> DiscoveryFile {
    read_target(agent.discovery_path()).expect("invariant: the agent wrote its discovery file")
}

fn wait_closed(agent: &mut Agent, wakes: &mpsc::Receiver<()>) -> usize {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        for incoming in agent.take_incoming() {
            if let Incoming::Closed { remaining } = incoming {
                return remaining;
            }
        }
        let left = deadline.saturating_duration_since(Instant::now());
        assert!(wakes.recv_timeout(left).is_ok(), "no Closed arrived");
    }
}

fn connect(file: &DiscoveryFile) -> TcpStream {
    let port = u16::try_from(file.port).expect("invariant: the agent's port fits");
    TcpStream::connect(("127.0.0.1", port)).expect("invariant: the agent accepts connections")
}

fn send_line(stream: &mut TcpStream, line: &str) {
    stream
        .write_all(line.as_bytes())
        .expect("invariant: the test stream writes");
    stream
        .write_all(b"\n")
        .expect("invariant: the test stream writes");
    stream.flush().expect("invariant: the test stream flushes");
}

fn read_response(stream: &mut TcpStream) -> RpcResponse {
    let mut reader = BufReader::new(
        stream
            .try_clone()
            .expect("invariant: the test stream can be cloned"),
    );
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("invariant: the agent answers the request");
    serde_json::from_str(&line).expect("invariant: the answer is JSON-RPC")
}

#[test]
fn get_app_info_is_answered_on_the_ui_thread() {
    let (mut agent, wakes) = start_agent_woken();
    let file = discovery(&agent);
    let (sender, receiver) = mpsc::channel();
    let client = std::thread::spawn(move || {
        let connection = Connection::open(&file, client_info(ClientKind::Other))
            .expect("invariant: the client connects")
            .0;
        let info = connection
            .call::<GetAppInfo>(Empty {})
            .expect("invariant: get_app_info is answered");
        let _ = sender.send(info);
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut answered = false;
    while !answered {
        let left = deadline.saturating_duration_since(Instant::now());
        assert!(wakes.recv_timeout(left).is_ok(), "the call did not arrive");
        for incoming in agent.take_incoming() {
            match incoming {
                Incoming::Call(Call::GetAppInfo(_), responder) => {
                    responder.send(Ok(Reply::GetAppInfo(agent.app().clone())));
                    answered = true;
                }
                Incoming::Call(_, responder) => {
                    responder.send(Err(RpcError::new(code::INTERNAL_ERROR, "unexpected call")));
                }
                Incoming::Closed { .. } => {}
            }
        }
    }
    let info = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("the call was answered");
    client
        .join()
        .expect("invariant: the client thread finishes");
    assert!(!info.workspace_root.is_empty());
    assert_eq!(info, agent.app().clone());
}

#[test]
fn get_config_is_answered_with_defaults() {
    let agent = start_agent();
    let file = discovery(&agent);
    let connection = Connection::open(&file, client_info(ClientKind::Other))
        .expect("invariant: the client connects")
        .0;
    let config = connection
        .call::<GetConfig>(Empty {})
        .expect("invariant: get_config is answered");
    let keys = config.keys.expect("invariant: get_config fills the keys");
    assert_eq!(
        keys.open.expect("invariant: keys.open is filled"),
        vec!["F12".to_owned(), "Ctrl+Shift+I".to_owned()]
    );
    assert_eq!(
        keys.pick.expect("invariant: keys.pick is filled"),
        vec!["Ctrl+Shift+C".to_owned()]
    );
}

#[test]
fn a_wrong_token_is_rejected() {
    let agent = start_agent();
    let mut file = discovery(&agent);
    file.token = "wrong".to_owned();
    let error = match Connection::open(&file, client_info(ClientKind::Other)) {
        Err(error) => error,
        Ok(_) => panic!("the wrong token was accepted"),
    };
    match error {
        ClientError::Rpc(error) => assert_eq!(error.code, code::BAD_TOKEN),
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn protocol_mismatches_and_early_calls_are_rejected() {
    let agent = start_agent();
    let file = discovery(&agent);
    let mut stream = connect(&file);
    let params = serde_json::to_value(InitializeParams {
        protocol: 99,
        token: file.token.clone(),
        client: client_info(ClientKind::Other),
    })
    .expect("invariant: the initialize params serialize");
    send_line(&mut stream, &request_line(1, "initialize", params));
    let response = read_response(&mut stream);
    assert_eq!(
        response
            .error
            .expect("invariant: a version mismatch is an error")
            .code,
        code::VERSION_MISMATCH
    );

    let mut stream = connect(&file);
    send_line(
        &mut stream,
        &request_line(2, "get_app_info", serde_json::json!({})),
    );
    let response = read_response(&mut stream);
    assert_eq!(
        response
            .error
            .expect("invariant: an early call is an error")
            .code,
        code::NOT_INITIALIZED
    );
}

#[test]
fn subscribe_limits_notifications_to_its_topics() {
    let agent = start_agent();
    let file = discovery(&agent);
    let connection = Connection::open(&file, client_info(ClientKind::Other))
        .expect("invariant: the client connects")
        .0;
    connection
        .call::<Subscribe>(SubscribeParams {
            topics: vec![Topic::Windows],
        })
        .expect("invariant: subscribe is answered");
    assert!(agent.wants(Topic::Windows));
    assert!(!agent.wants(Topic::Errors));
    agent.notify(Notification::WindowsChanged(WindowList {
        windows: Vec::new(),
    }));
    match connection
        .next_notification(Duration::from_secs(10))
        .expect("invariant: the connection stays open")
    {
        Some(Notification::WindowsChanged(list)) => assert!(list.windows.is_empty()),
        other => panic!("unexpected notification: {other:?}"),
    }
    agent.notify(Notification::ErrorReported(ErrorEntry {
        seq: 1,
        kind: "test".to_owned(),
        message: "boom".to_owned(),
        source: None,
        turn: None,
        handled: false,
    }));
    assert!(
        connection
            .next_notification(Duration::from_millis(200))
            .expect("invariant: the connection stays open")
            .is_none()
    );
}

#[test]
fn closing_a_connection_reports_the_remaining_clients() {
    let (mut agent, wakes) = start_agent_woken();
    let file = discovery(&agent);
    let first = Connection::open(&file, client_info(ClientKind::Other))
        .expect("invariant: the first client connects")
        .0;
    let second = Connection::open(&file, client_info(ClientKind::Other))
        .expect("invariant: the second client connects")
        .0;
    drop(first);
    assert_eq!(wait_closed(&mut agent, &wakes), 1);
    drop(second);
    assert_eq!(wait_closed(&mut agent, &wakes), 0);
}

#[test]
fn open_devtools_notifies_a_connected_devtools_client() {
    let mut agent = start_agent();
    let file = discovery(&agent);
    let connection = Connection::open(&file, client_info(ClientKind::Devtools))
        .expect("invariant: the DevTools client connects")
        .0;
    connection
        .call::<Subscribe>(SubscribeParams {
            topics: vec![Topic::Devtools],
        })
        .expect("invariant: subscribe is answered");
    agent
        .open_devtools()
        .expect("a DevTools client is connected");
    match connection
        .next_notification(Duration::from_secs(10))
        .expect("invariant: the connection stays open")
    {
        Some(Notification::FocusRequested(Empty {})) => {}
        other => panic!("unexpected notification: {other:?}"),
    }
}

#[test]
fn drop_removes_the_discovery_file_and_does_not_hang() {
    let agent = start_agent();
    let path = agent.discovery_path().to_path_buf();
    let file = read_target(&path).expect("invariant: the agent wrote its discovery file");
    let _connection = Connection::open(&file, client_info(ClientKind::Other))
        .expect("invariant: the client connects")
        .0;
    drop(agent);
    assert!(!path.exists());
}
