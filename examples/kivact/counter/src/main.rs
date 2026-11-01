//! Role: Increment one signal from a button and show a memo derived from it.
#![warn(redundant_imports)]

use kivact::prelude::*;

fn app() -> impl IntoView {
    let count = signal(0);
    let doubled = memo(move || count.get() * 2);
    column((
        text(move || format!("Count: {}", count.get())),
        text(move || format!("Doubled: {}", doubled.get())),
        button("Increment").on_press(move || count.update(|n| *n += 1)),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Kivact · Counter").run(app)
}

#[cfg(test)]
mod tests {
    //! Role: Drive the counter example in a test window.

    use super::*;

    #[test]
    fn pressing_increment_updates_the_count_and_the_memo() {
        let screen = kivact::test::render(app);
        screen.get_by_text("Count: 0");
        screen.get_by_text("Doubled: 0");
        screen.get_by_role(Role::Button, "Increment").click();
        screen.get_by_role(Role::Button, "Increment").click();
        screen.get_by_text("Count: 2");
        screen.get_by_text("Doubled: 4");
    }
}
