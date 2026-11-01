//! Role: Mini Postman through `kivact::test`.
//! Real HTTP (curl against a local
//! listener) where timing does not matter; a virtual-clock transport where
//! it does (latest-wins, cancel).
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact::task::sleep;
use kivact::test::{Element, Screen, render};
use kivact_example_mini_postman::{
    HistoryEntry, PendingResponse, Request, Response, Transport, curl, load_history, postman,
    save_history,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn temp_path(name: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("kivact-postman-{name}-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn open(path: &Path, transport: Transport) -> Screen {
    let path = path.to_path_buf();
    let screen = render(move || postman(path, transport));
    screen.find_by_text("送信待ち");
    screen
}

/// Replace the value of a text input (caret to the end, erase, type).
fn fill(input: Element<'_>, text: &str) {
    let old = input.value().unwrap_or_default().chars().count();
    input.press_key(Key::End);
    for _ in 0..old {
        input.press_key(Key::Backspace);
    }
    if !text.is_empty() {
        input.type_text(text);
    }
}

fn status(screen: &Screen) -> String {
    screen.get_by_test_id("postman-status").text()
}

/// Wait for the real transport, not the presence of the always-mounted status.
fn wait_for_response(screen: &Screen) {
    // curl has a 12-second limit; allow extra time for process startup and scheduling.
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        screen.run_until_idle();
        let current = status(screen);
        if current != "送信中" {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "HTTP response remained pending beyond the real-time deadline: {current}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// Virtual-clock transport: URLs containing "slow" take 1 s, others 10 ms.
fn fake(request: Request) -> PendingResponse {
    Box::pin(async move {
        let delay = if request.url.contains("slow") {
            1000
        } else {
            10
        };
        sleep(Duration::from_millis(delay)).await;
        if request.url.contains("fail") {
            return Err("connection refused".to_string());
        }
        Ok(Response {
            status: 200,
            headers: "HTTP/1.1 200 OK".into(),
            body: format!("{} {}", request.method, request.url),
            elapsed_ms: u128::from(delay),
        })
    })
}

/// One-shot local HTTP server; returns the URL base and the received request.
fn serve_once(reply: &'static [u8]) -> (String, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a local test port");
    let address = listener
        .local_addr()
        .expect("bound listener has an address");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("the app connects to the test server");
        stream
            .set_read_timeout(Some(Duration::from_millis(500)))
            .expect("set a read timeout on the test connection");
        let mut received = Vec::new();
        let mut buffer = [0; 4096];
        while let Ok(size) = stream.read(&mut buffer) {
            if size == 0 {
                break;
            }
            received.extend_from_slice(&buffer[..size]);
            let text = String::from_utf8_lossy(&received);
            if let Some((head, body)) = text.split_once("\r\n\r\n") {
                let length = head
                    .lines()
                    .find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                    })
                    .unwrap_or(0);
                if body.len() >= length {
                    break;
                }
            }
        }
        stream.write_all(reply).expect("send the canned reply");
        String::from_utf8_lossy(&received).into_owned()
    });
    (format!("http://{address}"), server)
}

#[test]
fn invalid_url_is_reported_before_sending() {
    let path = temp_path("invalid");
    let screen = open(&path, fake);
    fill(screen.get_by_role(Role::TextBox, "URL"), "invalid-url");
    screen.get_by_role(Role::Button, "送信").click();
    assert!(
        status(&screen).contains("URLは http://"),
        "{}",
        status(&screen)
    );
    fill(screen.get_by_test_id("url"), "http://127.0.0.1/");
    fill(screen.get_by_test_id("headers"), "Bad Header: x");
    screen.get_by_role(Role::Button, "送信").click();
    assert!(
        status(&screen).starts_with("ヘッダー名が不正です"),
        "{}",
        status(&screen)
    );
    assert!(load_history(&path).unwrap().is_empty());
}

#[test]
fn real_get_shows_the_response_and_persists_history() {
    let path = temp_path("get");
    let (base, server) =
        serve_once(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nX-Reply: yes\r\n\r\nhello");
    let screen = open(&path, curl);
    fill(screen.get_by_test_id("url"), &format!("{base}/hello"));
    screen.get_by_test_id("send").click();
    wait_for_response(&screen);
    let ok = screen.get_by_test_id("postman-status");
    assert!(ok.text().starts_with("HTTP 200"), "{}", ok.text());
    let request = server.join().unwrap();
    assert!(request.starts_with("GET /hello"), "{request}");
    assert!(request.contains("Accept: application/json"));
    let response = screen.get_by_test_id("response").text();
    assert!(
        response.contains("X-Reply: yes") && response.ends_with("hello"),
        "{response}"
    );
    screen.run_until_idle();
    let saved = load_history(&path).unwrap();
    assert_eq!(saved.len(), 1);
    assert!(saved[0].result.starts_with("HTTP 200"));
    // Copy the response through the platform clipboard
    screen.get_by_role(Role::Button, "応答をコピー").click();
    screen.find_by_text("応答をコピーしました");
    assert_eq!(screen.clipboard().as_deref(), Some(response.as_str()));
    drop(screen);

    // A new App restores the history row
    let reopened = open(&path, fake);
    let row = reopened.find_by_test_id("history-1");
    assert!(
        row.name()
            .starts_with(&format!("GET {base}/hello — HTTP 200")),
        "{}",
        row.name()
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn real_post_sends_method_headers_and_body() {
    let path = temp_path("post");
    let (base, server) = serve_once(b"HTTP/1.1 201 Created\r\nContent-Length: 2\r\n\r\nok");
    let screen = open(&path, curl);
    screen
        .get_by_role(Role::ComboBox, "メソッド")
        .select_option("POST");
    fill(screen.get_by_test_id("url"), &format!("{base}/items"));
    fill(screen.get_by_test_id("headers"), "X-Tutorial: yes");
    fill(screen.get_by_test_id("request-body"), "sample-body");
    screen.get_by_test_id("send").click();
    wait_for_response(&screen);
    let ok = screen.get_by_test_id("postman-status");
    assert!(ok.text().starts_with("HTTP 201"), "{}", ok.text());
    let request = server.join().unwrap();
    assert!(request.starts_with("POST /items"), "{request}");
    assert!(request.contains("X-Tutorial: yes"));
    assert!(request.ends_with("sample-body"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn only_the_latest_response_enters_history() {
    let path = temp_path("latest");
    let screen = open(&path, fake);
    fill(screen.get_by_test_id("url"), "http://example.test/slow");
    screen.get_by_test_id("send").click();
    assert_eq!(status(&screen), "送信中");
    assert!(!screen.get_by_role(Role::Button, "キャンセル").is_disabled());
    screen.advance(Duration::from_millis(100));
    fill(screen.get_by_test_id("url"), "http://example.test/fast");
    screen.get_by_test_id("send").click();
    screen.advance(Duration::from_secs(2));
    assert_eq!(status(&screen), "HTTP 200 · 10 ms");
    assert_eq!(
        screen.get_by_test_id("response").text(),
        "HTTP/1.1 200 OK\n\nGET http://example.test/fast"
    );
    assert!(screen.get_by_role(Role::Button, "キャンセル").is_disabled());
    screen.run_until_idle();
    let saved = load_history(&path).unwrap();
    assert_eq!(saved.len(), 1, "the superseded request is not recorded");
    assert!(saved[0].request.url.ends_with("/fast"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn retry_cancel_and_failure_are_visible() {
    let path = temp_path("cancel");
    let screen = open(&path, fake);
    screen.get_by_role(Role::Button, "再試行").click();
    assert_eq!(status(&screen), "再試行する履歴がありません");
    fill(screen.get_by_test_id("url"), "http://example.test/slow");
    screen.get_by_test_id("send").click();
    screen.advance(Duration::from_millis(200));
    screen.get_by_role(Role::Button, "キャンセル").click();
    assert_eq!(status(&screen), "キャンセルしました");
    screen.advance(Duration::from_secs(2));
    assert_eq!(
        status(&screen),
        "キャンセルしました",
        "a cancelled run never publishes"
    );
    assert!(screen.query_by_test_id("history-1").is_none());
    // Retry re-runs the last input
    screen.get_by_role(Role::Button, "再試行").click();
    assert_eq!(status(&screen), "送信中");
    screen.advance(Duration::from_secs(2));
    assert_eq!(status(&screen), "HTTP 200 · 1000 ms");
    screen.find_by_test_id("history-1");
    // A transport failure is shown and recorded
    fill(screen.get_by_test_id("url"), "http://example.test/fail");
    screen.get_by_test_id("send").click();
    screen.advance(Duration::from_millis(50));
    assert_eq!(status(&screen), "失敗: connection refused");
    assert_eq!(
        screen.get_by_test_id("response").text(),
        "connection refused"
    );
    screen.find_by_test_id("history-2");
    let _ = std::fs::remove_file(path);
}

#[test]
fn saved_history_row_restores_every_form_field() {
    let path = temp_path("restore");
    let saved = HistoryEntry {
        id: 1,
        request: Request {
            method: "POST".into(),
            url: "http://127.0.0.1:34123/post".into(),
            headers: "X-Tutorial: yes".into(),
            body: "gui-body".into(),
        },
        result: "HTTP 201".into(),
    };
    save_history(&path, &[saved]).unwrap();
    let screen = open(&path, fake);
    screen.find_by_test_id("history-1").click();
    assert_eq!(
        screen
            .get_by_role(Role::ComboBox, "メソッド")
            .value()
            .as_deref(),
        Some("POST")
    );
    for (id, expected) in [
        ("url", "http://127.0.0.1:34123/post"),
        ("headers", "X-Tutorial: yes"),
        ("request-body", "gui-body"),
    ] {
        assert_eq!(
            screen.get_by_test_id(id).value().as_deref(),
            Some(expected),
            "{id}"
        );
    }
    // With no run in this session, retry sends the newest history entry
    screen.get_by_role(Role::Button, "再試行").click();
    screen.advance(Duration::from_millis(50));
    assert_eq!(status(&screen), "HTTP 200 · 10 ms");
    screen.run_until_idle();
    assert_eq!(load_history(&path).unwrap().len(), 2);
    let _ = std::fs::remove_file(path);
}

#[test]
fn focus_button_moves_focus_to_the_url_field() {
    let path = temp_path("focus");
    let screen = open(&path, fake);
    screen.get_by_role(Role::Button, "URLへ移動").click();
    screen.find_by_text("URL欄へ移動しました");
    let focused = screen.focused().expect("something has focus");
    assert_eq!(focused.name(), "URL");
    // Keyboard only: Tab order continues from the URL field
    screen.press_tab(false);
    assert_eq!(
        screen.focused().map(|f| f.name()).as_deref(),
        Some("URLへ移動")
    );
}

#[test]
fn history_survives_many_quick_requests_in_order() {
    let path = temp_path("order");
    let screen = open(&path, fake);
    for index in 0..35 {
        fill(
            screen.get_by_test_id("url"),
            &format!("http://example.test/{index}"),
        );
        screen.get_by_test_id("send").click();
        screen.advance(Duration::from_millis(20));
    }
    screen.run_until_idle();
    let saved = load_history(&path).unwrap();
    assert_eq!(saved.len(), 30, "the history keeps the newest 30");
    assert!(saved[0].request.url.ends_with("/34"));
    assert_eq!(saved[0].id, 35);
    let _ = std::fs::remove_file(path);
}
