#![allow(dead_code, unused_variables)]
// Usage example: child component, arguments, form
use kivact::prelude::*;
use kivact::{Binding, Callback};
use kivact::view::{PointerEvent, callback};
use kivact::style::Color;
use kivact::draw::Draw;
use kivact::draw::Rect;
use kivact::view::Handled;
use kivact::view::PointerId;
use kivact::draw::Size;
include!("../support/prelude_stubs.rs");

// 部品の入力は普通の引数。title は親の Signal・memo・定数のどれからも作れる。
// on_save は子から親へ現在値を渡す
pub fn editor(title: Value<String>, on_save: Callback<String>) -> impl IntoView {
    let draft = signal(title.get_untracked());
    let dirty = memo(move || draft.with(|d| *d != title.get()));
    column((
        kivact::control::node().name("題名").part(kivact::interaction::use_text_edit(draft).control()), // Signal<String> をそのまま双方向に接続
        { let label = ("保存").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }
            .disabled(move || !dirty.get())
            .on_press(move || on_save.call(draft.get())),
    ))
}

fn parent() -> impl IntoView {
    // 親
    let title = signal(String::from("最初"));
    let saved = callback(move |next| title.set(next));
    editor(title.into_value(), saved)
}

fn field_binding() -> impl IntoView {
    let form = signal(Form::default());
    let url: Binding<String> = form.field(|f| &f.url, |f| &mut f.url);
    kivact::control::node().name("URL").part(kivact::interaction::use_text_edit(url).control())
}

fn main() {
    let _ = parent;
    let _ = field_binding;
}
