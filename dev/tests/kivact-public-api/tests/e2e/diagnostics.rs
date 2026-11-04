//! Role: Failure reports (snapshots), the structured trace and why chains.

use kivact::style::current_theme;

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

const WHY: &str = "node 5 Text \"doubled: 4\"
  Content set by the effect created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
    reads memo i32 created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C
      reads signal i32 created at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C, last written at dev/tests/kivact-public-api/tests/e2e/diagnostics.rs:L:C (turn 4)";

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
            .ends_with("Option<kivact_types::ids::FocusTarget>")
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

/// A running App reads the errors no boundary handled: a failing handler's
/// error is in `diagnostics::errors`, the same ring `Screen::errors` reads.
#[test]
fn the_app_reads_the_errors_of_its_default_sink() {
    let screen = render(|| {
        let seen = signal(0_usize);
        column((
            button("Fail").on_press(|| -> kivact::Result<()> {
                Err(UiError::new(
                    kivact::diagnostics::ErrorKind::Message,
                    "export failed",
                    None,
                ))
            }),
            button("Count").on_press(move || seen.set(kivact::diagnostics::errors().len())),
            text(move || format!("errors: {}", seen.get())),
        ))
    });
    screen.get_by_role(Role::Button, "Count").click();
    screen.get_by_text("errors: 0");
    screen.get_by_role(Role::Button, "Fail").click();
    screen.get_by_role(Role::Button, "Count").click();
    screen.get_by_text("errors: 1");
    let errors = kivact::diagnostics::errors();
    assert!(
        errors.is_empty(),
        "outside the App's code there is no current App"
    );
    assert_eq!(screen.errors().len(), 1);
}

/// A setup-only function called from a handler is a misuse: it panics at the
/// call and ends the test, with the kind in its message.
#[test]
#[should_panic(expected = "SetupOnly")]
fn a_setup_function_in_a_handler_panics() {
    let screen = render(|| {
        button("Misuse").on_press(|| {
            let _late = signal(0);
        })
    });
    screen.get_by_role(Role::Button, "Misuse").click();
}
