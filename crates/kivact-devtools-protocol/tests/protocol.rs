//! Role: Tests of the wire types, the JSON-RPC helpers and the pure protocol functions.

use std::ffi::OsString;
use std::path::PathBuf;

use kivact_devtools_protocol::{
    AppInfo, BuildKind, Call, Comment, DevtoolsConfig, Empty, Emulation, GetByTestId,
    GetTreeParams, KeyConfig, METHOD_NAMES, Message, Method, Notification, PseudoLocale, Rect,
    Reply, ReviewDocument, ReviewItem, ReviewTarget, Role, RpcError, RpcRequest, chrome_trace,
    code, discovery_dir_from, discovery_file_name, locator, notification_line, parse_call,
    parse_message, parse_request, request_line, response, response_line, rfc3339_utc, to_markdown,
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

fn vars(pairs: Vec<(&'static str, &'static str)>) -> impl Fn(&str) -> Option<OsString> {
    move |name| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| OsString::from(value))
    }
}

#[test]
fn get_tree_call_is_parsed() {
    let line = request_line(1, "get_tree", serde_json::json!({ "window": 1 }));
    let request = parse_request(&line).unwrap();
    assert_eq!(request.method, "get_tree");
    let call = parse_call(&request).unwrap();
    assert_eq!(
        call,
        Call::GetTree(GetTreeParams {
            window: 1,
            root: None,
            depth: None
        })
    );
}

#[test]
fn errors_have_their_codes() {
    let request = RpcRequest {
        jsonrpc: "2.0".to_owned(),
        id: serde_json::json!(1),
        method: "nope".to_owned(),
        params: None,
    };
    assert_eq!(
        parse_call(&request).unwrap_err().code,
        code::METHOD_NOT_FOUND
    );
    let request = RpcRequest {
        jsonrpc: "2.0".to_owned(),
        id: serde_json::json!(1),
        method: "get_tree".to_owned(),
        params: Some(serde_json::json!({ "window": "x" })),
    };
    assert_eq!(parse_call(&request).unwrap_err().code, code::INVALID_PARAMS);
    assert_eq!(parse_request("{").unwrap_err().code, code::PARSE_ERROR);
}

#[test]
fn empty_params_may_be_omitted_or_null() {
    let request = parse_request(r#"{"jsonrpc":"2.0","id":1,"method":"get_app_info"}"#).unwrap();
    assert_eq!(parse_call(&request).unwrap(), Call::GetAppInfo(Empty {}));
    let request =
        parse_request(r#"{"jsonrpc":"2.0","id":1,"method":"get_app_info","params":null}"#).unwrap();
    assert_eq!(parse_call(&request).unwrap(), Call::GetAppInfo(Empty {}));
}

#[test]
fn responses_and_notifications_round_trip() {
    let value = Reply::GetAppInfo(app_info()).into_value();
    assert!(value.is_object());
    let line = response_line(&response(serde_json::json!(7), Ok(value)));
    match parse_message(&line).unwrap() {
        Message::Response(response) => assert_eq!(response.id, serde_json::json!(7)),
        Message::Notification(_) => panic!("expected a response"),
    }
    let notification = Notification::FocusRequested(Empty {});
    let line = notification_line(&notification);
    assert_eq!(
        parse_message(&line).unwrap(),
        Message::Notification(notification)
    );
    assert_eq!(
        Notification::FocusRequested(Empty {}).method(),
        "focus_requested"
    );
}

#[test]
fn methods_are_complete() {
    assert_eq!(METHOD_NAMES.len(), 60);
    assert_eq!(METHOD_NAMES[0], "initialize");
    assert_eq!(<GetByTestId as Method>::NAME, "get_by_test_id");
}

#[test]
fn time_text_is_rfc3339() {
    assert_eq!(rfc3339_utc(0), "1970-01-01T00:00:00Z");
    assert_eq!(rfc3339_utc(1_762_579_920_000), "2025-11-08T05:32:00Z");
}

#[test]
fn discovery_directory_and_file_name() {
    let temp = PathBuf::from("temp");
    let local = discovery_dir_from(vars(vec![("LOCALAPPDATA", "C:/local")]), temp.clone());
    assert_eq!(local, PathBuf::from("C:/local/kivact/devtools"));
    let xdg = discovery_dir_from(vars(vec![("XDG_RUNTIME_DIR", "R")]), temp.clone());
    assert_eq!(xdg, PathBuf::from("R/kivact/devtools"));
    let home = discovery_dir_from(vars(vec![("HOME", "H")]), temp.clone());
    assert_eq!(home, PathBuf::from("H/.cache/kivact/devtools"));
    let fallback = discovery_dir_from(vars(Vec::new()), temp.clone());
    assert_eq!(fallback, temp.join("kivact-devtools"));
    let empty_local = discovery_dir_from(
        vars(vec![("LOCALAPPDATA", ""), ("XDG_RUNTIME_DIR", "R")]),
        temp.clone(),
    );
    assert_eq!(empty_local, PathBuf::from("R/kivact/devtools"));
    assert_eq!(discovery_file_name(12, 3456), "12-3456.json");
}

#[test]
fn config_defaults_fill_in() {
    let default = DevtoolsConfig::default();
    assert_eq!(default.open_keys(), vec!["F12", "Ctrl+Shift+I"]);
    let config = DevtoolsConfig {
        keys: Some(KeyConfig {
            open: Some(Vec::new()),
            pick: None,
        }),
        ..DevtoolsConfig::default()
    };
    assert!(config.open_keys().is_empty());
    assert_eq!(config.pick_keys(), vec!["Ctrl+Shift+C"]);
    let parsed: DevtoolsConfig = serde_json::from_str("{}").unwrap();
    assert_eq!(parsed, DevtoolsConfig::default());
    let parsed: DevtoolsConfig = serde_json::from_str(r#"{"$schema":"x","editor":[]}"#).unwrap();
    assert_eq!(parsed.schema.as_deref(), Some("x"));
    assert!(!parsed.with_defaults().editor.is_empty());
}

#[test]
fn codes_are_stable() {
    assert_eq!(code::NODE_NOT_FOUND, -32010);
    assert_eq!(code::UNSUPPORTED, -32015);
    assert_eq!(RpcError::new(code::UNSUPPORTED, "x").code, -32015);
}

fn review_document() -> ReviewDocument {
    let node = ReviewItem {
        id: "C1".to_owned(),
        window_title: "Main".to_owned(),
        window: 1,
        target: ReviewTarget::Node {
            node: 5,
            role: Some(Role::Button),
            name: Some("Save".to_owned()),
            test_id: Some("save".to_owned()),
            source: None,
            call_chain: Vec::new(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 80.0,
                height: 24.0,
            },
            states: Vec::new(),
        },
        style_changes: Vec::new(),
        image: None,
        comments: vec![Comment {
            id: "C1.1".to_owned(),
            text: "Looks good".to_owned(),
            created: "2026-11-08T05:32:00Z".to_owned(),
        }],
    };
    let window = ReviewItem {
        id: "C2".to_owned(),
        window_title: "Settings".to_owned(),
        window: 2,
        target: ReviewTarget::Window {},
        style_changes: Vec::new(),
        image: None,
        comments: Vec::new(),
    };
    ReviewDocument {
        schema: "kivact-devtools-review/1".to_owned(),
        app: app_info(),
        session: "20261108-053200-12-3456".to_owned(),
        created: "2026-11-08T05:32:00Z".to_owned(),
        updated: "2026-11-08T05:32:00Z".to_owned(),
        environment: Emulation {
            theme: None,
            reduced_motion: None,
            locale: None,
            pseudo_locale: PseudoLocale::Off,
            scale: None,
            touch: None,
        },
        items: vec![node, window],
    }
}

#[test]
fn review_markdown_and_locator() {
    let document = review_document();
    let markdown = to_markdown(&document);
    assert!(markdown.contains("## C1 Button \"Save\""));
    assert!(markdown.contains("## C2 window \"Settings\""));
    assert!(markdown.contains("- Comments:"));
    assert_eq!(
        locator(&document.items[0]).as_deref(),
        Some("screen.get_by_test_id(\"save\")")
    );
    assert_eq!(locator(&document.items[1]), None);

    let other = ReviewItem {
        id: "C3".to_owned(),
        window_title: "Settings".to_owned(),
        window: 2,
        target: ReviewTarget::Node {
            node: 6,
            role: Some(Role::Button),
            name: Some("OK".to_owned()),
            test_id: None,
            source: None,
            call_chain: Vec::new(),
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 40.0,
                height: 24.0,
            },
            states: Vec::new(),
        },
        style_changes: Vec::new(),
        image: None,
        comments: Vec::new(),
    };
    assert_eq!(
        locator(&other).as_deref(),
        Some("screen.window(\"Settings\").get_by_role(Role::Button, \"OK\")")
    );
}

#[test]
fn empty_trace_is_empty() {
    assert_eq!(chrome_trace(&[], &[]), "{\"traceEvents\":[]}");
}
