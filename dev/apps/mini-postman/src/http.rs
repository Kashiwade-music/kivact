//! Role: Bounded HTTP transport for the tutorial, using the system curl executable.
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::pin::Pin;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use kivact::task::{CancelToken, blocking};

/// A pending response. Dropping it cancels the request.
pub type PendingResponse = Pin<Box<dyn Future<Output = Result<Response, String>>>>;

/// How the app sends a request: [`curl`] in the app, a virtual-clock fake in
/// tests (a plain `fn` pointer, so it is `Copy`).
pub type Transport = fn(Request) -> PendingResponse;

/// Run [`execute`] on the App's blocking pool. Dropping the future (a newer
/// request, Cancel, or closing the window) flips the token and stops curl.
pub fn curl(request: Request) -> PendingResponse {
    Box::pin(blocking(move |cancel: CancelToken| {
        execute(request, &|| cancel.is_cancelled())
    }))
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: String,
    pub body: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Response {
    pub status: u16,
    pub headers: String,
    pub body: String,
    pub elapsed_ms: u128,
}

pub fn parse_headers(value: &str) -> Result<Vec<(String, String)>, String> {
    value
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let (name, value) = line
                .split_once(':')
                .ok_or_else(|| format!("ヘッダーに : がありません: {line}"))?;
            let name = name.trim();
            let value = value.trim();
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            {
                return Err(format!("ヘッダー名が不正です: {name}"));
            }
            if value.contains(['\r', '\n']) {
                return Err("ヘッダー値に改行を含められません".into());
            }
            Ok((name.into(), value.into()))
        })
        .collect()
}

pub fn validate(request: &Request) -> Result<Vec<(String, String)>, String> {
    if !matches!(request.method.as_str(), "GET" | "POST") {
        return Err("GETまたはPOSTを選んでください".into());
    }
    if !(request.url.starts_with("http://") || request.url.starts_with("https://")) {
        return Err("URLは http:// または https:// から始めてください".into());
    }
    if request.url.contains(['\r', '\n']) {
        return Err("URLに改行を含められません".into());
    }
    if request.body.len() > 1_048_576 {
        return Err("本文は1MiB以内にしてください".into());
    }
    parse_headers(&request.headers)
}

fn read_pipe(
    mut pipe: impl Read + Send + 'static,
) -> std::thread::JoinHandle<std::io::Result<Vec<u8>>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        pipe.read_to_end(&mut bytes)?;
        Ok(bytes)
    })
}

pub fn execute(request: Request, cancelled: &dyn Fn() -> bool) -> Result<Response, String> {
    let headers = validate(&request)?;
    if cancelled() {
        return Err("キャンセルしました".into());
    }
    let mut command = Command::new("curl");
    command.args([
        "--silent",
        "--show-error",
        "--include",
        "--max-time",
        "12",
        "--connect-timeout",
        "3",
        "--max-filesize",
        "1048576",
        "--request",
        &request.method,
        "--write-out",
        "\n__KIVACT_STATUS__=%{http_code}",
        "--url",
        &request.url,
    ]);
    for (name, value) in headers {
        command.arg("--header").arg(format!("{name}: {value}"));
    }
    if request.method == "POST" {
        command.args(["--data-binary", "@-"]);
    }
    let started = Instant::now();
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("curlを起動できません: {error}"))?;
    let stdout = read_pipe(child.stdout.take().ok_or("curl stdoutがありません")?);
    let stderr = read_pipe(child.stderr.take().ok_or("curl stderrがありません")?);
    if request.method == "POST" {
        child
            .stdin
            .take()
            .ok_or("curl stdinがありません")?
            .write_all(request.body.as_bytes())
            .map_err(|error| format!("本文を送れません: {error}"))?;
    } else {
        drop(child.stdin.take());
    }
    let status = loop {
        if cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout.join();
            let _ = stderr.join();
            return Err("キャンセルしました".into());
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(Duration::from_millis(15)),
            Err(error) => return Err(error.to_string()),
        }
    };
    let output = stdout
        .join()
        .map_err(|_| "curl出力スレッドが停止しました")?
        .map_err(|error| error.to_string())?;
    let errors = stderr
        .join()
        .map_err(|_| "curlエラースレッドが停止しました")?
        .map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(String::from_utf8_lossy(&errors).trim().to_owned());
    }
    let text = String::from_utf8_lossy(&output);
    let (payload, code) = text
        .rsplit_once("\n__KIVACT_STATUS__=")
        .ok_or("HTTP statusを取得できません")?;
    let code = code.trim().parse().map_err(|_| "HTTP statusが不正です")?;
    let (headers, body) = split_http_message(payload)?;
    Ok(Response {
        status: code,
        headers: headers.into(),
        body: body.into(),
        elapsed_ms: started.elapsed().as_millis(),
    })
}

fn split_http_message(mut text: &str) -> Result<(&str, &str), String> {
    loop {
        let (header, body) = text
            .split_once("\r\n\r\n")
            .or_else(|| text.split_once("\n\n"))
            .ok_or("HTTP headerを取得できません")?;
        let first_line = header.lines().next().unwrap_or_default();
        let informational = first_line
            .split_whitespace()
            .nth(1)
            .and_then(|code| code.parse::<u16>().ok())
            .is_some_and(|code| (100..200).contains(&code));
        let proxy_tunnel = first_line.contains("Connection established");
        if (informational || proxy_tunnel) && body.starts_with("HTTP/") {
            text = body;
        } else {
            return Ok((header, body));
        }
    }
}
