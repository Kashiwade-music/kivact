//! Role: Compose a reusable card with props, children and a close callback.
#![warn(redundant_imports)]

use kivact::prelude::*;

/// A card with a title, a collapsible body and a close button that tells the
/// parent through `on_close`.
///
/// The body is a `ViewFn`: closing and opening the card creates it again, so
/// the two cards of the example never share their contents.
#[component]
fn card(
    title: Value<String>,
    #[prop(optional)] on_close: Option<Callback<()>>,
    content: ViewFn,
) -> impl IntoView {
    let open = signal(true);
    let close = on_close.map(|on_close| button("Close").on_press(on_close));
    column((
        row((
            text(title),
            button(move || if open.get() { "Collapse" } else { "Expand" })
                .on_press(move || open.update(|open| *open = !*open)),
            close,
        ))
        .style(style().gap(8)),
        show(open, content),
    ))
    .style(style().padding(12).gap(8))
}

fn app() -> impl IntoView {
    // xtask-allow(state-fields): independent: each card closes from its own button
    let first = signal(true);
    // xtask-allow(state-fields): independent: each card closes from its own button
    let second = signal(true);
    column((
        show(first, move || {
            card("First", || text("The first card holds one line."))
                .on_close(move || first.set(false))
        }),
        show(second, move || {
            card("Second", || {
                column((
                    text("The second card holds two lines."),
                    text("Its children differ from the first."),
                ))
            })
            .on_close(move || second.set(false))
        }),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Kivact · Component").run(app)
}

#[cfg(test)]
mod tests {
    //! Role: Drive the card component example in a test window.

    use super::*;

    #[test]
    fn collapsing_hides_the_body_and_closing_hides_the_card() {
        let screen = kivact::test::render(app);
        screen.get_by_text("The first card holds one line.");
        screen.get_by_text("The second card holds two lines.");

        screen.all_by_role(Role::Button, "Collapse")[0].click();
        assert!(
            screen
                .query_by_text("The first card holds one line.")
                .is_none()
        );
        screen.get_by_text("The second card holds two lines.");
        screen.get_by_role(Role::Button, "Expand").click();
        screen.get_by_text("The first card holds one line.");

        screen.all_by_role(Role::Button, "Close")[0].click();
        assert!(screen.query_by_text("First").is_none());
        assert!(
            screen
                .query_by_text("The first card holds one line.")
                .is_none()
        );
        screen.get_by_text("Second");
        screen.get_by_text("Its children differ from the first.");
    }
}
