#![allow(dead_code, unused_variables)]
// Usage example: testing (Testing Library style)
use kivact::prelude::*;
use kivact::view::PointerEvent;
use kivact::style::Color;
use kivact::draw::Draw;
use kivact::draw::Rect;
use kivact::view::Handled;
use kivact::Callback;
use kivact::view::PointerId;
use kivact::draw::Size;
include!("../support/prelude_stubs.rs");

#[test]
fn saves_title() {
    let screen = kivact::test::render(editor_demo);
    screen.get_by_role(Role::TextBox, "題名").type_text("新しい"); // IME 確定と同じ経路
    screen.get_by_role(Role::Button, "保存").click();
    screen.find_by_text("保存しました"); // 仮想時計を進めながら待つ（上限は既定5秒）
    assert_eq!(screen.get_by_test_id("status").name(), "保存しました");
}

fn main() {}
