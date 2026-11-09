//! Role: Where the DevTools app gets its data: a live agent or fixed answers for tests.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use kivact::prelude::*;
use kivact_devtools_client as client;
use kivact_devtools_protocol as p;

/// A live connection, or fixed answers.
#[derive(Clone)]
pub enum Source {
    /// A connection to an agent.
    Live(Arc<client::Connection>),
    /// Fixed answers for tests.
    Fixture(Rc<Fixture>),
}

impl PartialEq for Source {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Source::Live(a), Source::Live(b)) => Arc::ptr_eq(a, b),
            (Source::Fixture(a), Source::Fixture(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// Fixed answers for tests.
pub struct Fixture {
    /// The app.
    pub app: p::AppInfo,
    /// The open windows.
    pub windows: Vec<p::WindowInfo>,
    /// The element trees.
    pub trees: Vec<p::TreeSnapshot>,
    /// The details of single nodes.
    pub nodes: Vec<p::NodeDetail>,
    /// The recorded turns.
    pub turns: Vec<p::TurnGraph>,
    /// The recorded frames.
    pub frames: Vec<p::FrameInfo>,
    /// Answers by method name. They win over the fields above.
    pub answers: Vec<(String, serde_json::Value)>,
    /// Notifications that `Source::listen` delivers once, in order.
    pub notifications: Vec<p::Notification>,
    /// Every call made (method name and params), for assertions.
    pub calls: RefCell<Vec<(String, serde_json::Value)>>,
}

impl Source {
    /// Call method `M`. `Live` waits on a blocking thread; `Fixture` answers at once.
    pub(crate) async fn call<M>(&self, params: M::Params) -> Result<M::Result, UiError>
    where
        M: p::Method + 'static,
        M::Params: Send + 'static,
        M::Result: Send + 'static,
    {
        match self {
            Source::Live(connection) => {
                let connection = connection.clone();
                blocking(move || connection.call::<M>(params))
                    .await
                    .map_err(UiError::from)
            }
            Source::Fixture(fixture) => fixture.answer::<M>(&params),
        }
    }

    /// Deliver notifications: `Live` reads the connection, `Fixture` replays its list.
    pub(crate) fn listen(
        &self,
        on: impl Fn(p::Notification) + 'static,
        closed: impl FnOnce(String) + 'static,
    ) {
        match self {
            Source::Live(connection) => {
                let connection = connection.clone();
                spawn(async move {
                    loop {
                        let connection = connection.clone();
                        match blocking(move || {
                            connection.next_notification(Duration::from_millis(500))
                        })
                        .await
                        {
                            Ok(Some(notification)) => on(notification),
                            Ok(None) => {}
                            Err(error) => {
                                closed(error.to_string());
                                break;
                            }
                        }
                    }
                });
            }
            Source::Fixture(fixture) => {
                for notification in fixture.notifications.clone() {
                    on(notification);
                }
            }
        }
    }
}

impl Fixture {
    /// Answer `M`: an `answers` entry, else a fixture field, else `{}`, read as `M::Result`.
    fn answer<M>(&self, params: &M::Params) -> Result<M::Result, UiError>
    where
        M: p::Method,
    {
        let sent = serde_json::to_value(params).map_err(UiError::from)?;
        self.calls
            .borrow_mut()
            .push((M::NAME.to_owned(), sent.clone()));
        let value = self
            .answers
            .iter()
            .find(|(name, _)| name.as_str() == M::NAME)
            .map(|(_, value)| value.clone())
            .or_else(|| builtin_answer(self, M::NAME, &sent))
            .unwrap_or_else(|| serde_json::json!({}));
        serde_json::from_value::<M::Result>(value)
            .map_err(|_| UiError::msg(format!("not in the fixture: {}", M::NAME)))
    }
}

/// The answers the fixture's fields give, by method name.
fn builtin_answer(
    fixture: &Fixture,
    method: &str,
    params: &serde_json::Value,
) -> Option<serde_json::Value> {
    match method {
        "get_app_info" => serde_json::to_value(&fixture.app).ok(),
        "list_windows" => serde_json::to_value(p::WindowList {
            windows: fixture.windows.clone(),
        })
        .ok(),
        "get_tree" => {
            let window = params.get("window").and_then(serde_json::Value::as_u64)?;
            let tree = fixture.trees.iter().find(|tree| tree.window == window)?;
            serde_json::to_value(tree).ok()
        }
        "get_node" => {
            let node = params.get("node").and_then(serde_json::Value::as_u64)?;
            let detail = fixture.nodes.iter().find(|detail| detail.node.id == node)?;
            serde_json::to_value(detail).ok()
        }
        "list_turns" => serde_json::to_value(p::TurnList {
            turns: fixture.turns.clone(),
        })
        .ok(),
        "list_frames" => serde_json::to_value(p::FrameList {
            frames: fixture.frames.clone(),
        })
        .ok(),
        _ => None,
    }
}
