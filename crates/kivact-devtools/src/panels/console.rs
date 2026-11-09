//! Role: The Console panel.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;
use crate::widgets::source_link::source_link;

/// The errors and notes the app reported, filtered by kind.
pub(crate) fn view(session: Session) -> impl IntoView {
    let kind = signal(Some("all".to_owned()));
    let rows = memo(move || {
        let selected = kind.get().unwrap_or_else(|| "all".to_owned());
        session.errors.with(|errors| {
            errors
                .iter()
                .filter(|error| selected == "all" || error.kind == selected)
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    let kinds = memo(move || {
        let mut kinds = Vec::<String>::new();
        session.errors.with(|errors| {
            for error in errors {
                if !kinds.contains(&error.kind) {
                    kinds.push(error.kind.clone());
                }
            }
        });
        let mut items = vec![ListItem::new("all".to_owned(), "All kinds")];
        items.extend(
            kinds
                .into_iter()
                .map(|kind| ListItem::new(kind.clone(), kind)),
        );
        items
    });

    column((
        row((
            text("Console"),
            select("Kind", kinds, kind).style(style().width(180.0)),
        )),
        header(),
        virtual_list(
            rows,
            |error| error.seq,
            move |error| error_row(error, session),
        )
        .row_height(RowHeight::Fixed(22.0))
        .style(style().grow(1.0).min_height(0.0)),
    ))
    .style(style().grow(1.0).min_height(0.0))
}

/// The column titles of the error table.
fn header() -> impl IntoView {
    row((
        text("Kind").style(style().width(90.0)),
        text("Message").style(style().width(260.0)),
        text("Source").style(style().width(160.0)),
        text("Turn").style(style().width(60.0)),
        text("Handled").style(style().width(70.0)),
    ))
}

/// One reported error.
fn error_row(entry: Value<p::ErrorEntry>, session: Session) -> impl IntoView {
    let source = match entry.get_untracked().source {
        Some(location) => source_link(location, session).into_view(),
        None => text("").into_view(),
    };
    row((
        text(move || entry.get().kind).style(style().width(90.0)),
        text(move || entry.get().message).style(style().width(260.0)),
        node().style(style().width(160.0)).child(source),
        text(move || turn_text(entry.get().turn)).style(style().width(60.0)),
        text(move || yes_no(entry.get().handled)).style(style().width(70.0)),
    ))
}

/// A turn number, or a dash when there is none.
fn turn_text(turn: Option<u64>) -> String {
    turn.map(|turn| turn.to_string())
        .unwrap_or_else(|| "-".to_owned())
}

/// `yes` or `no`.
fn yes_no(value: bool) -> String {
    if value {
        "yes".to_owned()
    } else {
        "no".to_owned()
    }
}
