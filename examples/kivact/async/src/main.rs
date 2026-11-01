//! Role: Load a record by id with a resource and render loading, error and ready.
#![warn(redundant_imports)]

use std::time::Duration;

use kivact::prelude::*;

fn app() -> impl IntoView {
    let id = signal(1u32);
    let user = resource(id, |id| async move {
        kivact::task::sleep(Duration::from_millis(300)).await;
        if id == 0 {
            Err(UiError::msg("no user 0"))
        } else {
            Ok(format!("User #{id}"))
        }
    });
    column((
        row((
            button("Previous")
                .disabled(move || id.get() == 0)
                .on_press(move || id.update(|n| *n -= 1)),
            button("Next").on_press(move || id.update(|n| *n += 1)),
        ))
        .style(style().gap(8)),
        dynamic(
            move || user.state(),
            |state| match state {
                AsyncState::Idle | AsyncState::Pending => text("Loading…"),
                AsyncState::Failed(error) => text(format!("Error: {error}")),
                AsyncState::Ready(name) => text(name),
                // Variants added later fall back to the loading text.
                _ => text("Loading…"),
            },
        ),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Kivact · Async").run(app)
}

#[cfg(test)]
mod tests {
    //! Role: Drive the async resource example in a test window.

    use super::*;

    const LOAD: Duration = Duration::from_millis(300);

    #[test]
    fn next_and_previous_load_and_user_zero_fails() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Loading…");
        screen.advance(LOAD);
        screen.get_by_text("User #1");

        screen.get_by_role(Role::Button, "Next").click();
        screen.get_by_text("Loading…");
        screen.advance(LOAD);
        screen.get_by_text("User #2");

        screen.get_by_role(Role::Button, "Previous").click();
        screen.get_by_role(Role::Button, "Previous").click();
        screen.advance(LOAD);
        screen.get_by_text("Error: no user 0");
        assert!(screen.get_by_role(Role::Button, "Previous").is_disabled());

        screen.get_by_role(Role::Button, "Next").click();
        screen.advance(LOAD);
        screen.get_by_text("User #1");
    }
}
