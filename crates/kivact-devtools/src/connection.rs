//! Role: Finding a target, connecting to it, and the connection state screens.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use kivact::prelude::*;
use kivact_devtools_client as client;
use kivact_devtools_protocol as p;

use crate::source::Source;
use crate::state::{Connection, Target};

/// Connect to the discovery file at `path`, after listing its windows and configuration.
pub(crate) fn connect(connection: Signal<Connection>, path: PathBuf) {
    connection.set(Connection::Connecting(path.clone()));
    spawn(async move {
        let opened = blocking({
            let path = path.clone();
            move || open_target(&path)
        })
        .await;
        match opened {
            Ok((connection_handle, app, windows, config)) => {
                connection.set(Connection::Connected(Target {
                    source: Source::Live(Arc::new(connection_handle)),
                    app,
                    windows,
                    config: std::rc::Rc::new(config),
                    path: Some(path),
                }));
            }
            Err(reason) => connection.set(Connection::Lost {
                path,
                app_name: String::new(),
                reason,
            }),
        }
    });
}

/// Open a connection and read the app, its windows and its configuration.
fn open_target(
    path: &Path,
) -> Result<
    (
        client::Connection,
        p::AppInfo,
        Vec<p::WindowInfo>,
        p::DevtoolsConfig,
    ),
    String,
> {
    let file = client::read_target(path).map_err(|error| error.to_string())?;
    if file.protocol != p::PROTOCOL_VERSION {
        return Err(format!(
            "version differs (target v{}, DevTools v{})",
            file.protocol,
            p::PROTOCOL_VERSION
        ));
    }
    let (connection, initialized) = client::Connection::open(
        &file,
        p::ClientInfo {
            name: "kivact-devtools".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            kind: p::ClientKind::Devtools,
        },
    )
    .map_err(|error| error.to_string())?;
    let windows = connection
        .call::<p::ListWindows>(p::Empty {})
        .map_err(|error| error.to_string())?;
    let config = connection
        .call::<p::GetConfig>(p::Empty {})
        .map_err(|error| error.to_string())?;
    Ok((connection, initialized.app, windows.windows, config))
}

/// List the running targets and connect to the preferred one, or let the user choose.
pub(crate) fn discover(connection: Signal<Connection>, prefer: Option<(PathBuf, String)>) {
    connection.set(Connection::Searching);
    spawn(async move {
        let list = blocking(client::list_targets).await;
        match choose(&list, prefer.as_ref()) {
            Some(path) => connect(connection, path),
            None => connection.set(Connection::Choosing(list)),
        }
    });
}

/// The target to connect to: the preferred file, else its only app, else the only target.
fn choose(
    list: &[(PathBuf, p::DiscoveryFile)],
    prefer: Option<&(PathBuf, String)>,
) -> Option<PathBuf> {
    match prefer {
        Some((path, app_name)) => {
            if let Some((found, _)) = list.iter().find(|(candidate, _)| candidate == path) {
                return Some(found.clone());
            }
            let mut named = list
                .iter()
                .filter(|(_, file)| file.app_name == *app_name)
                .map(|(candidate, _)| candidate.clone());
            let first = named.next()?;
            match named.next() {
                Some(_) => None,
                None => Some(first),
            }
        }
        None => match list {
            [only] => Some(only.0.clone()),
            _ => None,
        },
    }
}

/// The screen shown while the app lists the running targets.
pub(crate) fn searching_view() -> impl IntoView {
    column((text("Looking for running apps…"), kivact::ui::spinner()))
}

/// The screen that lets the user pick one of `targets`.
pub(crate) fn choosing_view(
    connection: Signal<Connection>,
    targets: Vec<(PathBuf, p::DiscoveryFile)>,
) -> impl IntoView {
    if targets.is_empty() {
        column((
            text("No running Kivact app with DevTools was found."),
            button("Refresh").on_press(move || discover(connection, None)),
        ))
        .into_view()
    } else {
        column(for_each(
            targets,
            |(path, _)| path.clone(),
            move |row| {
                let (path, file) = row.get();
                button(format!(
                    "{} · pid {} · {}",
                    file.app_name, file.pid, file.workspace_root
                ))
                .on_press(move || connect(connection, path.clone()))
            },
        ))
        .into_view()
    }
}

/// The screen shown while a connection is being opened.
pub(crate) fn connecting_view(path: PathBuf) -> impl IntoView {
    column((
        kivact::ui::spinner(),
        text(path.to_string_lossy().into_owned()),
    ))
}

/// The screen shown after the connection ended.
pub(crate) fn lost_view(
    connection: Signal<Connection>,
    path: PathBuf,
    app_name: String,
    reason: String,
) -> impl IntoView {
    column((
        text(reason),
        row((
            button("Reconnect")
                .on_press(move || discover(connection, Some((path.clone(), app_name.clone())))),
            button("Choose another target").on_press(move || discover(connection, None)),
        )),
    ))
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the target choice.

    use super::*;

    #[test]
    fn choose_prefers_the_same_file() {
        let list = vec![target("a.json", "A"), target("b.json", "B")];
        let prefer = (PathBuf::from("b.json"), "A".to_owned());
        assert_eq!(choose(&list, Some(&prefer)), Some(PathBuf::from("b.json")));
    }

    #[test]
    fn choose_falls_back_to_the_only_app_of_the_name() {
        let list = vec![target("a.json", "A"), target("b.json", "B")];
        let prefer = (PathBuf::from("gone.json"), "B".to_owned());
        assert_eq!(choose(&list, Some(&prefer)), Some(PathBuf::from("b.json")));
    }

    #[test]
    fn choose_takes_the_only_target() {
        let list = vec![target("a.json", "A")];
        assert_eq!(choose(&list, None), Some(PathBuf::from("a.json")));
    }

    #[test]
    fn choose_gives_up_with_two_targets() {
        let list = vec![target("a.json", "A"), target("b.json", "B")];
        assert_eq!(choose(&list, None), None);
    }

    fn target(path: &str, app_name: &str) -> (PathBuf, p::DiscoveryFile) {
        let file = serde_json::from_value(serde_json::json!({
            "protocol": p::PROTOCOL_VERSION,
            "pid": 1,
            "app_name": app_name,
            "kivact_version": "0.1.0",
            "port": 4000,
            "token": "token",
            "started_unix_ms": 1,
            "workspace_root": "/workspace"
        }))
        .expect("invariant: the test discovery file is valid");
        (PathBuf::from(path), file)
    }
}
