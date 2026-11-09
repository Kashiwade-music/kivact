//! Role: Tests of the Console panel.

use std::rc::Rc;

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::support;

#[test]
fn the_kind_filter_narrows_the_errors() {
    let mut fixture = support::fixture();
    fixture
        .notifications
        .push(p::Notification::ErrorReported(error(1, "panic", "Boom")));
    fixture
        .notifications
        .push(p::Notification::ErrorReported(error(
            2, "warning", "Careful",
        )));
    let fixture = Rc::new(fixture);

    let screen = support::open(&fixture, "Console");
    screen.get_by_text("Boom");
    screen.get_by_text("Careful");

    screen.get_by_role(Role::ComboBox, "Kind").click();
    screen.get_by_role(Role::Option, "panic").click();
    screen.get_by_text("Boom");
    assert!(screen.query_by_text("Careful").is_none());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

fn error(seq: u64, kind: &str, message: &str) -> p::ErrorEntry {
    serde_json::from_value(serde_json::json!({
        "seq": seq,
        "kind": kind,
        "message": message,
        "handled": false
    }))
    .expect("invariant: the test error is valid")
}
