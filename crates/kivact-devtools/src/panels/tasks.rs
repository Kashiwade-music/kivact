//! Role: The Async panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// The tasks, resources, actions and timers.
pub(crate) fn view(session: Session) -> impl IntoView {
    let data = signal(None::<p::TaskList>);
    on_mount(move || refresh(session, data));

    let rows = memo(move || data.get().map(|list| list.tasks).unwrap_or_default());
    column((
        text(move || match data.get() {
            Some(list) => format!("Outstanding blocking: {}", list.outstanding_blocking),
            None => "Outstanding blocking: loading...".to_owned(),
        }),
        data_table(
            rows,
            |task: &p::TaskInfo| task.id,
            vec![
                DataColumn::new("Kind", |task: &p::TaskInfo| kind_text(task.kind)),
                DataColumn::new("Id", |task: &p::TaskInfo| task.id.to_string()),
                DataColumn::new("Phase", |task: &p::TaskInfo| phase_text(task.phase)),
                DataColumn::new("Owner", |task: &p::TaskInfo| owner_text(task.owner)),
                DataColumn::new("Deadline", |task: &p::TaskInfo| {
                    deadline_text(task.deadline_ns)
                }),
                DataColumn::new("Source", |task: &p::TaskInfo| {
                    source_text(task.source.as_ref())
                }),
            ],
        )
        .row_label(|task| format!("{} #{}", kind_text(task.kind), task.id))
        .name("Tasks and resources")
        .style(style().grow(1.0).min_height(0.0)),
    ))
    .style(style().grow(1.0).min_height(0.0))
}

/// Read the tasks, resources, actions and timers.
fn refresh(session: Session, data: Signal<Option<p::TaskList>>) {
    session.send::<p::ListTasks>(p::Empty {}, move |list| data.set(Some(list)));
}

/// The wire name of a task kind.
fn kind_text(kind: p::TaskKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "task".to_owned())
}

/// The wire name of an async phase.
fn phase_text(phase: p::AsyncPhase) -> String {
    serde_json::to_value(phase)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "idle".to_owned())
}

/// An owner id, or a dash when there is none.
fn owner_text(owner: Option<p::OwnerId>) -> String {
    owner
        .map(|owner| owner.to_string())
        .unwrap_or_else(|| "-".to_owned())
}

/// A timer deadline in milliseconds, or a dash when there is none.
fn deadline_text(deadline_ns: Option<u64>) -> String {
    deadline_ns
        .map(|ns| format!("{:.3} ms", ns as f64 / 1_000_000.0))
        .unwrap_or_else(|| "-".to_owned())
}

/// A source location as `file:line`, or a dash when there is none.
fn source_text(source: Option<&p::SourceLocation>) -> String {
    source
        .map(|source| format!("{}:{}", source.file, source.line))
        .unwrap_or_else(|| "-".to_owned())
}
