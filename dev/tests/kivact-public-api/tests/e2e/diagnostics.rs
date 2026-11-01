//! Role: Failure reports (snapshots), the structured trace and why chains.

use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use std::panic::AssertUnwindSafe;

use kivact::prelude::*;
use kivact::style::{Align, Justify};
use kivact::test::render;

fn app() -> impl IntoView {
    let count = signal(0);
    let doubled = memo(move || count.get() * 2);
    column((
        text(move || format!("count: {}", count.get())).test_id("count"),
        text(move || format!("doubled: {}", doubled.get())).test_id("doubled"),
        row((
            {
                let label = ("+1").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
                    .style(move || {
                        let t = &current_theme();
                        style()
                            .direction(kivact::style::Direction::Row)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .padding_x(t.space().lg)
                            .padding_y(t.space().sm)
                            .min_height(t.control_height())
                    })
            }
            .on_press(move || count.update(|c| *c += 1))
            .test_id("inc"),
            {
                let label = ("+1").into_value();
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(text(label))
                    .style(move || {
                        let t = &current_theme();
                        style()
                            .direction(kivact::style::Direction::Row)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .padding_x(t.space().lg)
                            .padding_y(t.space().sm)
                            .min_height(t.control_height())
                    })
            }
            .test_id("other"),
        ))
        .test_id("buttons"),
    ))
}

/// Stable snapshots: `/` separators, `file:L:C` positions, no durations.
fn normalize(s: &str) -> String {
    let s = s.replace('\\', "/");
    let mut out = String::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        // ":<digits>:<digits>" → ":L:C"
        if b[i] == b':' {
            let mut j = i + 1;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            if j > i + 1 && j < b.len() && b[j] == b':' {
                let mut k = j + 1;
                while k < b.len() && b[k].is_ascii_digit() {
                    k += 1;
                }
                if k > j + 1 {
                    out.push_str(":L:C");
                    i = k;
                    continue;
                }
            }
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

/// The failure message of `f`, without the paths of the failure artifacts
/// (the last line).
fn failure(f: impl FnOnce()) -> String {
    let message = match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(()) => panic!("expected the query to fail"),
        Err(p) => p
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_default(),
    };
    match message.find(
        "
failure artifacts: ",
    ) {
        Some(at) => message[..at].to_string(),
        None => message,
    }
}

// Nodes are named by where their builder was called (`@file:line:col`);
// a button's label shares its button's location and the root has
// none. The accessibility tree is the one the host published (laid out
// with the bundled test font).
const MISSING: &str = r#"kivact test: no element matches text "count: 5"
scope: whole screen
candidates: 0
focus: node 8 Some(Button) "+1" @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C region None
recent actions: ["click node 8"]
recent updates:
  turn 1: 2 op(s), 4 re-run(s), causes ["HoverChanged Some(9)", "FocusChanged Some(8) region None visible false", "Activate n8 region None"]
    write core::option::Option<kivact_wire::ids::FocusTarget> (created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C) at crates/kivact-core/src/view/focus.rs:L:C
    write i32 (created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C) at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
errors: []
accessibility tree (published by the host):
Generic "" [0 0 800x600]
  Generic "" [0 0 800x66]
    Text "count: 1" #count [0 0 800x19]
    Text "doubled: 2" #doubled [0 19 800x19]
    Generic "" [0 38 800x28]
      Button "+1" focusable #inc [0 38 39x28]
        Text "+1" [12 43 15x19]
      Button "+1" focusable #other [39 38 38x28]
        Text "+1" [51 43 14x19]
focus: 8
semantic tree:
None ""
  None "" @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
    Some(Text) "count: 1" #count @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
    Some(Text) "doubled: 2" #doubled @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
    None "" #buttons @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
      Some(Button) "+1" #inc @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C [focused]
        Some(Text) "+1" @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
      Some(Button) "+1" #other @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
        Some(Text) "+1" @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
"#;

#[test]
fn failure_report_snapshot_for_a_missing_element() {
    let screen = render(app);
    screen.get_by_test_id("inc").click();
    let msg = normalize(&failure(|| {
        screen.get_by_text("count: 5");
    }));
    assert_eq!(msg, MISSING, "actual report:\n{msg}");
}

const AMBIGUOUS: &str = r#"kivact test: 2 elements match role Button named "+1" (ambiguous)
scope: node 6 None "" @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
candidates: 2
  node 8 Some(Button) "+1" @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
  node 10 Some(Button) "+1" @dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
focus: none
recent actions: []
recent updates:
errors: []
"#;

#[test]
fn ambiguous_query_reports_candidates_and_scope() {
    let screen = render(app);
    let buttons = screen.get_by_test_id("buttons");
    let msg = normalize(&failure(|| {
        screen.within(buttons).get_by_role(Role::Button, "+1");
    }));
    assert!(msg.starts_with(AMBIGUOUS), "actual report:\n{msg}");
    // A narrower scope resolves it.
    let inc = screen.get_by_test_id("inc");
    assert_eq!(
        screen.within(inc).get_by_role(Role::Button, "+1").name(),
        "+1"
    );
    assert!(screen.within(buttons).query_by_text("nope").is_none());
    assert_eq!(screen.within(buttons).all_by_text("+1").len(), 2);
}

const WHY: &str = "node 5 Text \"doubled: 4\"
  Content set by the effect created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
    reads memo i32 created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
      reads signal i32 created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C, last written at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C (turn 2)";

#[test]
fn why_follows_the_prop_to_the_write_through_a_memo() {
    let screen = render(app);
    screen.get_by_test_id("inc").click();
    screen.get_by_test_id("inc").click();
    let why = screen.get_by_test_id("doubled").why();
    assert_eq!(normalize(&why.to_string()), WHY, "actual:\n{why}");
    let content = &why.props[0];
    assert_eq!(content.prop, "Content");
    assert_eq!(content.reads[0].kind, "memo");
    let signal = &content.reads[0].reads[0];
    assert_eq!((signal.kind, signal.ty), ("signal", "i32"));
    assert!(
        signal
            .last_write
            .is_some_and(|(at, _)| at.file() == file!())
    );
    // A button's style is a render effect reading the theme signal.
    let other = screen.get_by_test_id("other").why();
    let paint = other.props.iter().find(|p| p.prop == "Paint");
    assert!(
        paint.is_some_and(|p| p.reads.iter().any(|d| d.ty.ends_with("Theme"))),
        "{other}"
    );
}

#[test]
fn trace_records_causes_writes_reruns_ops_and_duration() {
    let screen = render(app);
    screen.get_by_test_id("inc").click();
    let trace = screen.trace();
    let t = trace
        .iter()
        .find(|t| t.causes.iter().any(|c| c.starts_with("Activate")))
        .unwrap_or_else(|| panic!("no activation turn in {trace:#?}"));
    // The click moves the focus (the core's focus source) and writes the
    // counter.
    assert_eq!(t.writes.len(), 2, "{:#?}", t.writes);
    assert!(
        t.writes[0]
            .ty
            .ends_with("Option<kivact_wire::ids::FocusTarget>")
    );
    assert_eq!(t.writes[1].ty, "i32");
    assert_eq!(t.writes[1].at.file(), file!());
    // Two text render effects and the memo re-ran; two SetProps were sent.
    let kinds: Vec<&str> = t.reruns.iter().map(|r| r.kind).collect();
    assert!(kinds.contains(&"memo") && kinds.iter().filter(|k| **k == "render").count() >= 2);
    assert_eq!(t.ops, 2);
    assert!(t.duration_ns > 0);
    // Same-value write: nothing recorded but the focus move, no ops.
    let before = screen.trace().len();
    screen.get_by_test_id("other").click();
    let after = screen.trace();
    assert!(
        after[before..]
            .iter()
            .all(|t| t.ops == 0 && t.writes.iter().all(|w| w.at.file().ends_with("focus.rs")))
    );
}

#[test]
fn app_side_why_through_a_node_ref() {
    let slot: std::rc::Rc<std::cell::Cell<Option<NodeRef>>> = Default::default();
    let s2 = slot.clone();
    let screen = render(move || {
        let count = signal(1);
        let r = node_ref();
        s2.set(Some(r));
        text(move || format!("n = {}", count.get())).node_ref(r)
    });
    let Some(r) = slot.get() else {
        panic!("setup did not run")
    };
    let why = kivact::diagnostics::why(&r).unwrap_or_else(|| panic!("node is mounted"));
    assert_eq!(why.props.len(), 1);
    assert_eq!(why.props[0].reads[0].last_write, None);
    drop(screen);
}

/// `.at(location)` replaces the call site recorded for the node it is
/// called on (as a component forwarding its caller's location does); a
/// component's inner nodes keep their own, other builders theirs.
#[test]
fn at_replaces_the_call_site_of_the_node_it_is_called_on() {
    let here = std::panic::Location::caller();
    let label_line = std::rc::Rc::new(std::cell::Cell::new(0));
    let recorded = label_line.clone();

    let screen = render(move || {
        column((
            {
                let label = ("go").into_value();
                recorded.set(line!() + 1);
                let label_node = text(label);
                node()
                    .role(Role::Button)
                    .focusable(true)
                    .name(label)
                    .child(label_node)
                    .style(move || {
                        let t = &current_theme();
                        style()
                            .direction(kivact::style::Direction::Row)
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .padding_x(t.space().lg)
                            .padding_y(t.space().sm)
                            .min_height(t.control_height())
                    })
            }
            .test_id("go")
            .at(here),
            text("t").test_id("t").at(here),
        ))
    });
    let snap = screen.snapshot();
    let outer = &snap.children[0];
    let (go, t) = (&outer.children[0], &outer.children[1]);
    assert_eq!(go.at, Some(here));
    assert_eq!(t.at, Some(here));
    assert_eq!(
        go.children[0].at.map(|a| a.line()),
        Some(label_line.get()),
        "the child retains its own text builder call site"
    );
    assert_ne!(outer.at, Some(here), "other builders keep their call site");
}
