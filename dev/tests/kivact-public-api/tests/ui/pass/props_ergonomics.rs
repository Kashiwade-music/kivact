#![allow(dead_code, reason = "compile-only public component examples")]

use kivact::prelude::*;

#[derive(Clone, PartialEq)]
struct CardProps {
    title: String,
}

kivact::view::constant_value!(CardProps);

fn card(props: impl IntoValue<CardProps>) -> impl IntoView {
    let props = props.into_value();
    let title = memo(move || props.with(|p| p.title.clone()));
    node().name(title).child(text(title))
}

fn borrowed_label(label: &str) -> Node {
    node().name(label).child(text(label))
}

fn app() -> impl IntoView {
    let props = signal(CardProps {
        title: "reactive".to_owned(),
    });
    let revision = signal(0);
    let label = String::from("borrowed until this function returns");
    column((
        card((CardProps {
            title: "static".to_owned(),
        }).into_value()),
        card(props),
        card(props.into_value()),
        card(move || CardProps {
            title: format!("revision {}", revision.get()),
        }),
        borrowed_label(&label),
    ))
}

fn main() {
    let _ = || App::new("Props").run(app);
}
