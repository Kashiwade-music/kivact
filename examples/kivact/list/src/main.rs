//! Role: Keep a keyed todo list with checkboxes, removal and a remaining count.
#![warn(redundant_imports)]

use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    title: String,
    done: bool,
}

fn app() -> impl IntoView {
    let todos = signal(Vec::<Todo>::new());
    let draft = signal(String::new());
    let next_id = var(1);
    let left = memo(move || todos.with(|list| list.iter().filter(|todo| !todo.done).count()));

    let add = move || {
        let title = draft.get().trim().to_owned();
        if !title.is_empty() {
            let id = next_id.get();
            next_id.set(id + 1);
            todos.update(|list| {
                list.push(Todo {
                    id,
                    title,
                    done: false,
                })
            });
            draft.set(String::new());
        }
    };

    column((
        row((
            text_input("New todo", draft).style(style().width(240)),
            button("Add").on_press(add),
        ))
        .style(style().gap(8)),
        column(for_each(
            todos,
            |todo| todo.id,
            move |todo| {
                let id = todo.get().id;
                let done = todo.project(|t| &t.done, |t| &mut t.done);
                row((
                    checkbox(move || todo.get().title, done),
                    button("Remove")
                        .on_press(move || todos.update(|list| list.retain(|t| t.id != id))),
                ))
                .style(style().gap(8))
            },
        ))
        .style(style().gap(8)),
        text(move || format!("{} left", left.get())),
    ))
    .style(style().padding(24).gap(12))
}

fn main() -> kivact::Result<()> {
    App::new("Kivact · List").run(app)
}

#[cfg(test)]
mod tests {
    //! Role: Drive the todo list example in a test window.

    use super::*;

    fn add(screen: &kivact::test::Screen, title: &str) {
        screen
            .get_by_role(Role::TextBox, "New todo")
            .type_text(title);
        screen.get_by_role(Role::Button, "Add").click();
    }

    #[test]
    fn adding_checking_and_removing_update_the_remaining_count() {
        let screen = kivact::test::render(app);
        screen.get_by_text("0 left");

        add(&screen, "Buy milk");
        add(&screen, "Write code");
        screen.get_by_text("2 left");
        assert_eq!(
            screen
                .get_by_role(Role::TextBox, "New todo")
                .value()
                .as_deref(),
            Some("")
        );

        screen.get_by_role(Role::CheckBox, "Buy milk").click();
        assert_eq!(
            screen.get_by_role(Role::CheckBox, "Buy milk").checked(),
            Some(CheckedState::Checked)
        );
        screen.get_by_text("1 left");

        screen.all_by_role(Role::Button, "Remove")[0].click();
        assert!(screen.query_by_role(Role::CheckBox, "Buy milk").is_none());
        screen.get_by_role(Role::CheckBox, "Write code");
        screen.get_by_text("1 left");

        screen.get_by_role(Role::Button, "Remove").click();
        screen.get_by_text("0 left");
    }
}
