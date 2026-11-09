//! Role: The Input panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// The recorded input events and the IME state of one load.
#[derive(Clone, Debug, PartialEq)]
struct InputData {
    log: p::InputLog,
    ime: p::ImeState,
}

/// The recorded input events and the IME state.
pub(crate) fn view(session: Session) -> impl IntoView {
    let data = resource(
        move || session.window(),
        move |window| load(session, window),
    );
    let entries = memo(move || {
        data.latest()
            .map(|data| data.log.entries)
            .unwrap_or_default()
    });
    column((
        row((
            text("Input log"),
            spacer(),
            button("Refresh").on_press(move || data.refetch()),
        ))
        .style(style().align_items(Align::Center)),
        header(),
        virtual_list(entries, |entry| entry.seq, input_row)
            .row_height(RowHeight::Fixed(22.0))
            .style(style().grow(1.0).min_height(0.0)),
        text("IME state"),
        dynamic(
            move || data.state(),
            move |state| match state {
                AsyncState::Idle | AsyncState::Pending => {
                    text("The IME state is loading...").into_view()
                }
                AsyncState::Failed(error) => text(error.message().to_owned()).into_view(),
                AsyncState::Ready(data) => ime_view(data.ime).into_view(),
            },
        ),
    ))
    .style(style().grow(1.0).min_height(0.0))
}

/// Load the input log and the IME state of the inspected window.
async fn load(session: Session, window: p::WindowId) -> Result<InputData, UiError> {
    let source = session.source();
    let log = source
        .call::<p::ListInputs>(p::ListInputsParams {
            since: None,
            limit: Some(500),
        })
        .await?;
    let ime = source
        .call::<p::GetImeState>(p::WindowParams { window })
        .await?;
    Ok(InputData { log, ime })
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
fn ime_view(state: p::ImeState) -> impl IntoView {
    column((
        text(format!("Purpose: {}", state.purpose)),
        text(format!("Composing: {}", state.composing)),
        text(format!("Target node: {}", node_text(state.node))),
        state
            .preedit
            .map(|preedit| text(format!("Preedit: {preedit}"))),
        state.cursor_area.map(|area| {
            text(format!(
                "Cursor: {:.1} x {:.1} at {:.1}, {:.1}",
                area.width, area.height, area.x, area.y
            ))
        }),
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
