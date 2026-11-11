//! Role: The actions of one connected DevTools session.

use std::rc::Rc;

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::editor;
use crate::source::Source;
use crate::state::{
    Connection, Inspected, MAIN_WINDOW, MAX_ERRORS, MAX_FRAMES, MAX_TABLES, MAX_TURNS, Panel,
    Session, Target, overlays_off, push_capped,
};

impl Session {
    /// Make the handles of a connected session; call inside setup only.
    pub(crate) fn new(target: Target, connection: Signal<Connection>) -> Session {
        let source = target.source.clone();
        let windows = signal(target.windows.clone());
        // xtask-allow(state-fields): identity: the session's target, written once before any reader exists
        let target = var(target);
        let inspected = signal(Inspected::Loading {
            window: MAIN_WINDOW,
            select: None,
        });
        let turns = signal(Vec::new());
        let frames = signal(Vec::new());
        let errors = signal(Vec::new());
        let tables = signal(Vec::new());
        let panel = signal(Panel::Elements);
        let popped = signal(Vec::new());
        // xtask-allow(state-fields): independent: becomes Session.status, empty at setup; the target var beside it is fixed for the session whether or not a failure is shown
        let status = signal(None);
        let detail = {
            let source = source.clone();
            resource(
                move || inspected.with(Inspected::selected),
                move |node: Option<p::NodeId>| {
                    let source = source.clone();
                    async move {
                        match node {
                            None => Ok(None),
                            Some(node) => source
                                .call::<p::GetNode>(p::GetNodeParams { node })
                                .await
                                .map(Some),
                        }
                    }
                },
            )
        };
        let recording = read::<p::GetRecording>(&source);
        let overrides = read::<p::ListStyleOverrides>(&source);
        let emulation = read::<p::GetEmulation>(&source);
        let overlays = read::<p::GetOverlays>(&source);
        let review = read::<p::GetReview>(&source);
        let session = Session {
            target,
            connection,
            windows,
            inspected,
            turns,
            frames,
            errors,
            tables,
            panel,
            popped,
            status,
            detail,
            recording,
            overrides,
            emulation,
            overlays,
            review,
        };
        on_mount(move || {
            session.subscribe();
            session.load_window(MAIN_WINDOW, None);
            session.listen();
        });
        session
    }

    /// Subscribe to every topic.
    fn subscribe(self) {
        self.send::<p::Subscribe>(
            p::SubscribeParams {
                topics: vec![
                    p::Topic::Tree,
                    p::Topic::Turns,
                    p::Topic::Frames,
                    p::Topic::Pick,
                    p::Topic::Errors,
                    p::Topic::Devtools,
                    p::Topic::Scripts,
                    p::Topic::Windows,
                ],
            },
            |_| (),
        );
    }

    /// Deliver notifications until the connection closes; only this moves the app to `Lost`.
    fn listen(self) {
        let (path, app_name) = self
            .target
            .with(|t| (t.path.clone().unwrap_or_default(), t.app.name.clone()));
        let connection = self.connection;
        self.source().listen(
            move |notification| self.apply(notification),
            move |reason| {
                connection.set(Connection::Lost {
                    path,
                    app_name,
                    reason,
                });
            },
        );
    }

    /// Apply one notification.
    pub(crate) fn apply(self, notification: p::Notification) {
        match notification {
            p::Notification::TreeChanged(change) => {
                self.inspected
                    .update(|inspected| inspected.apply_diff(&change));
            }
            p::Notification::TurnRecorded(turn) => {
                self.turns
                    .update(|turns| push_capped(turns, turn, MAX_TURNS));
            }
            p::Notification::FrameRecorded(frame) => {
                self.frames
                    .update(|frames| push_capped(frames, frame, MAX_FRAMES));
            }
            p::Notification::Pick(event) => self.on_pick(&event),
            p::Notification::ErrorReported(error) => {
                self.errors
                    .update(|errors| push_capped(errors, error, MAX_ERRORS));
            }
            p::Notification::FocusRequested(_) => {
                spawn(async {
                    let _ = kivact::platform::focus_window().await;
                });
            }
            p::Notification::ScriptTable(table) => {
                self.tables
                    .update(|tables| push_capped(tables, table, MAX_TABLES));
            }
            p::Notification::WindowsChanged(windows) => self.on_windows(windows.windows),
        }
    }

    /// Follow a pick event.
    fn on_pick(self, event: &p::PickEvent) {
        match event.phase {
            p::PickPhase::Cancel => {}
            p::PickPhase::Hover => self.panel.set(Panel::Elements),
            p::PickPhase::Select => {
                self.panel.set(Panel::Elements);
                let Some(node) = event.node else {
                    return;
                };
                if self.inspected.update(|i| i.select_in(event.window, node)) {
                    self.load_window(event.window, Some(node));
                }
            }
        }
    }

    /// Follow a window list; fall back to the main window when the inspected one closed.
    fn on_windows(self, windows: Vec<p::WindowInfo>) {
        let main = self.inspected.update(|i| i.keep_window(&windows));
        self.windows.set(windows);
        if main {
            self.load_window(MAIN_WINDOW, None);
        }
    }

    /// Load the tree of `window` and select `select` when it arrives.
    pub(crate) fn load_window(self, window: p::WindowId, select: Option<p::NodeId>) {
        self.inspected.set(Inspected::Loading { window, select });
        let inspected = self.inspected;
        self.send::<p::GetTree>(
            p::GetTreeParams {
                window,
                root: None,
                depth: None,
            },
            move |tree| inspected.update(|i| i.finish_load(tree)),
        );
    }

    /// Call `M`; on success run `done`, on failure write the status bar.
    pub(crate) fn send<M>(self, params: M::Params, done: impl FnOnce(M::Result) + 'static)
    where
        M: p::Method + 'static,
        M::Params: Send + 'static,
        M::Result: Send + 'static,
    {
        let source = self.source();
        spawn(async move {
            match source.call::<M>(params).await {
                Ok(result) => done(result),
                Err(error) => self
                    .status
                    .set(Some(format!("{}: {}", M::NAME, error.message()))),
            }
        });
    }

    /// Highlight a node (`None` clears it), optionally with the layout boxes.
    pub(crate) fn highlight(self, node: Option<p::NodeId>, layout: bool) {
        self.send::<p::Highlight>(p::HighlightParams { node, layout }, |_| ());
    }

    /// Change one overlay toggle without clearing the others.
    pub(crate) fn set_overlay(self, edit: impl FnOnce(&mut p::OverlaySettings)) {
        let mut settings = self.overlays.latest().unwrap_or_else(overlays_off);
        edit(&mut settings);
        self.send::<p::SetOverlays>(settings, move |_| self.overlays.refetch());
    }

    /// Whether the current overlays are loaded (toggles wait for them).
    pub(crate) fn overlays_ready(self) -> bool {
        self.overlays.latest().is_some()
    }

    /// Show a failure in the status bar.
    pub(crate) fn report(self, text: String) {
        self.status.set(Some(text));
    }

    /// Open a source location in the user's editor.
    pub(crate) fn open_source(self, location: p::SourceLocation) {
        let config = self.target.with(|t| (*t.config).clone());
        let root = self.workspace_root();
        spawn(async move {
            if let Err(text) =
                blocking(move || editor::open_in_editor(&config, &root, &location)).await
            {
                self.report(text);
            }
        });
    }

    /// Where the session gets its data.
    pub(crate) fn source(self) -> Source {
        self.target.with(|t| t.source.clone())
    }

    /// The configuration read at connect time.
    pub(crate) fn config(self) -> Rc<p::DevtoolsConfig> {
        self.target.with(|t| t.config.clone())
    }

    /// A resource for a method that takes no parameters (setup only).
    pub(crate) fn read<M>(self) -> Resource<M::Result>
    where
        M: p::Method<Params = p::Empty> + 'static,
        M::Result: Clone + Send + 'static,
    {
        read::<M>(&self.source())
    }

    /// The workspace root of the target.
    pub(crate) fn workspace_root(self) -> String {
        self.target.with(|t| t.app.workspace_root.clone())
    }

    /// The window the app inspects.
    pub(crate) fn window(self) -> p::WindowId {
        self.inspected.with(Inspected::window)
    }
}

/// A resource for a method that takes no parameters.
fn read<M>(source: &Source) -> Resource<M::Result>
where
    M: p::Method<Params = p::Empty> + 'static,
    M::Result: Clone + Send + 'static,
{
    let source = source.clone();
    resource(
        || (),
        move |()| {
            let source = source.clone();
            async move { source.call::<M>(p::Empty {}).await }
        },
    )
}
