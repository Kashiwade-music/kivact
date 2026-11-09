//! Role: The Input panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// The recorded input events and the IME state.
pub(crate) fn view(session: Session) -> impl IntoView {
    let log = signal(None::<p::InputLog>);
    let ime = signal(None::<p::ImeState>);
    on_mount(move || load(session, log, ime));

    let entries = memo(move || log.get().map(|log| log.entries).unwrap_or_default());
    column((
        text("Input log"),
        header(),
        virtual_list(entries, |entry| entry.seq, input_row)
            .row_height(RowHeight::Fixed(22.0))
            .style(style().grow(1.0).min_height(0.0)),
        text("IME state"),
        show_some(ime, ime_view).fallback(|| text("The IME state is loading...")),
    ))
    .style(style().grow(1.0).min_height(0.0))
}

/// Load the input log and the IME state of the inspected window.
fn load(session: Session, log: Signal<Option<p::InputLog>>, ime: Signal<Option<p::ImeState>>) {
    let window = session.window();
    session.send::<p::ListInputs>(
        p::ListInputsParams {
            since: None,
            limit: Some(500),
        },
        move |result| log.set(Some(result)),
    );
    session.send::<p::GetImeState>(p::WindowParams { window }, move |result| {
        ime.set(Some(result));
    });
}

/// The column titles of the log.
fn header() -> impl IntoView {
    row((
        text("Kind").style(style().width(90.0)),
        text("Detail").style(style().width(140.0)),
        text("Target").style(style().width(60.0)),
        text("Route").style(style().width(110.0)),
        text("Handlers").style(style().width(220.0)),
        text("Source").style(style().width(70.0)),
    ))
}

/// One row of the input log.
fn input_row(entry: Value<p::InputLogEntry>) -> impl IntoView {
    row((
        text(move || entry.get().kind).style(style().width(90.0)),
        text(move || entry.get().detail).style(style().width(140.0)),
        text(move || node_text(entry.get().target)).style(style().width(60.0)),
        text(move || route_text(&entry.get())).style(style().width(110.0)),
        text(move || handlers_text(&entry.get())).style(style().width(220.0)),
        text(move || source_text(entry.get().source)).style(style().width(70.0)),
    ))
}

/// The IME state of the window.
fn ime_view(state: Value<p::ImeState>) -> impl IntoView {
    column((
        text(move || format!("Purpose: {}", state.get().purpose)),
        text(move || format!("Composing: {}", state.get().composing)),
        text(move || format!("Target node: {}", node_text(state.get().node))),
        show_some(
            move || state.get().preedit,
            |preedit| text(move || format!("Preedit: {}", preedit.get())),
        ),
        show_some(
            move || state.get().cursor_area,
            |area| {
                text(move || {
                    let area = area.get();
                    format!(
                        "Cursor: {:.1} x {:.1} at {:.1}, {:.1}",
                        area.width, area.height, area.x, area.y
                    )
                })
            },
        ),
    ))
}

/// A node id, or a dash when there is none.
fn node_text(node: Option<p::NodeId>) -> String {
    node.map(|node| node.to_string())
        .unwrap_or_else(|| "-".to_owned())
}

/// The route of an event, from the target to the root.
fn route_text(entry: &p::InputLogEntry) -> String {
    entry
        .route
        .iter()
        .map(|node| node.to_string())
        .collect::<Vec<_>>()
        .join(" > ")
}

/// The handlers that ran and their outcomes.
fn handlers_text(entry: &p::InputLogEntry) -> String {
    entry
        .handlers
        .iter()
        .map(|handler| format!("{}: {}", handler.event, outcome_text(handler.outcome)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The wire name of an input source.
fn source_text(source: p::InputSource) -> String {
    serde_json::to_value(source)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "platform".to_owned())
}

/// The wire name of a handler outcome.
fn outcome_text(outcome: p::HandlerOutcome) -> String {
    serde_json::to_value(outcome)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "not_handled".to_owned())
}
