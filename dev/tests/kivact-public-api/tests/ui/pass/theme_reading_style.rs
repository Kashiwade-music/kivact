#![allow(dead_code, unused_variables)]
// Writing an app: a `.style(..)` closure reads the theme with
// `current_theme()` on every element builder, and a plain
// `fn(&Theme) -> Style` is called with it.
use kivact::prelude::*;
use kivact::style::{Style};
use kivact::style::current_theme;

fn primary(t: &Theme) -> Style {
    style().padding_x(t.space().lg).radius(t.radius().md)
}

fn styled() -> impl IntoView {
    let wide = signal(false);
    column((
        node()
            .style(move || style().radius(current_theme().radius().sm))
            .style(style().margin(4)),
        text("label").style(move || style().font_size(current_theme().text().size_large)),
        node()
            .role(Role::Button)
            .focusable(true)
            .name("ok")
            .child(text("ok"))
            .style(move || primary(&current_theme())),
        kivact::control::node().name("name").part(kivact::interaction::use_text_edit(kivact::signal(String::new())).control()).style(move || {
            let t = &current_theme();
            let w = if wide.get() { 2.0 } else { 1.0 } * t.control_height();
            style().width(w)
        }),
    ))
    .style(move || style().gap(current_theme().space().md))
}

fn main() {
    let _ = || App::new("Theme style").run(styled);
}
