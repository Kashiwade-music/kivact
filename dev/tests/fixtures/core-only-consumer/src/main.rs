use kivact::prelude::*;

fn counter() -> impl IntoView {
    let count = signal(0);
    column((
        text(move || format!("Count: {}", count.get())),
        node()
            .role(Role::Button)
            .focusable(true)
            .name("Increment")
            .child(text("Increment"))
            .on_press(move || count.update(|value| *value += 1)),
    ))
}

fn main() -> kivact::Result<()> {
    App::new("Independent core consumer").run(counter)
}
