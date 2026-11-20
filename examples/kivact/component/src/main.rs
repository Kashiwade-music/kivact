//! Compose a reusable card with props, children and a close callback.

use kivact::prelude::*;

/// A card with a title, a collapsible body and a close button that tells the
/// parent through `on_close`.
///
/// The body is a `ViewFn`: closing and opening the card creates it again, so
/// the two cards of the example never share their contents.
#[component]
fn card(
    title: Value<String>,
    on_close: Option<Callback<()>>,
    content: kivact::view::ViewFn,
) -> impl Element {
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

/// A card that goes away when its close button is pressed: it keeps whether
/// it is shown itself, so every card closes on its own.
#[component]
fn closable_card(title: Value<String>, content: kivact::view::ViewFn) -> impl IntoView {
    let shown = signal(true);
    show(shown, move || {
        card(title, content.clone()).on_close(move || shown.set(false))
    })
}

fn app() -> impl IntoView {
    column((
        closable_card("First", || text("The first card holds one line.")),
        closable_card("Second", || {
            column((
                text("The second card holds two lines."),
                text("Its children differ from the first."),
            ))
        }),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Kivact · Component").run(app)
}

#[cfg(test)]
mod tests {
    //! Drive the card component example in a test window.

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
