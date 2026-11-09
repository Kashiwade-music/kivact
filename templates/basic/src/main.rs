//! Role: Counter app template on the standard components, with a test.
//!
//! `cargo run -p kivact-template-basic` opens a window;
//! `cargo test -p kivact-template-basic` drives the same UI in a test window
//! of the Arvane host (`kivact::test`).
#![warn(redundant_imports)]

use kivact::prelude::*;

/// A component is a function returning a view; `#[component]` also makes a
/// builder for its props. The setup runs once; dynamic parts are closures
/// over Copy handles.
#[component]
fn counter(initial: i32) -> impl IntoView {
    let count = signal(initial);
    let doubled = memo(move || count.get() * 2);
    column((
        text(move || format!("count: {} / doubled: {}", count.get(), doubled.get()))
            .test_id("count"),
        row((
            button("-").on_press(move || count.update(|c| *c -= 1)),
            button("+").on_press(move || count.update(|c| *c += 1)),
        ))
        .style(style().gap(8)),
    ))
    .style(style().gap(8).padding(16))
}

fn main() -> kivact::Result<()> {
    App::new("Kivact App").run(|| counter(0))
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the counter template's buttons.

    use super::*;

    #[test]
    fn buttons_change_the_count() {
        let screen = kivact::test::render(|| counter(1));
        screen.get_by_role(Role::Button, "+").click();
        screen.get_by_role(Role::Button, "+").press_key(Key::Enter);
        assert_eq!(
            screen.get_by_test_id("count").text(),
            "count: 3 / doubled: 6"
        );
        screen.get_by_role(Role::Button, "-").click();
        screen.get_by_text("count: 2 / doubled: 4");
    }
}
