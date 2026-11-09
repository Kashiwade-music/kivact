//! Role: Tests of the Scripts panel.

use std::rc::Rc;

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::support;

#[test]
fn tables_shown_by_scripts_are_listed() {
    let mut fixture = support::fixture();
    fixture
        .notifications
        .push(p::Notification::ScriptTable(p::ShowTableParams {
            title: "Buttons".to_owned(),
            columns: vec!["Name".to_owned(), "Kind".to_owned()],
            rows: vec![vec!["Increment".to_owned(), "button".to_owned()]],
        }));
    let fixture = Rc::new(fixture);
    let screen = support::open(&fixture, "Scripts");
    screen.get_by_text("Buttons");
    screen.get_by_text("Increment");
    screen.get_by_text("Interpreter");
    assert!(screen.get_by_role(Role::Button, "Run").is_disabled());
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}
