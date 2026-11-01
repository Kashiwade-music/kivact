#![allow(dead_code, unused_variables)]
// Usage example: list, selection, context
use kivact::prelude::*;
use kivact::view::{A11yAction};
use kivact::primitive::NodeExt;
use kivact::view::PointerEvent;
use kivact::style::Color;
use kivact::draw::Draw;
use kivact::draw::Rect;
use kivact::view::Handled;
use kivact::Callback;
use kivact::view::PointerId;
use kivact::draw::Size;
include!("../support/prelude_stubs.rs");

#[derive(Clone, PartialEq)]
pub struct Task {
    pub id: u64,
    pub title: String,
    pub done: bool,
}

#[derive(Clone, Copy)]
pub struct Board {
    pub tasks: Signal<Vec<Task>>,
    pub selected: Signal<Option<u64>>,
}

fn set_done(all: &mut Vec<Task>, id: u64, done: bool) {
    if let Some(t) = all.iter_mut().find(|t| t.id == id) {
        t.done = done;
    }
}

fn details(id: Value<u64>) -> impl IntoView {
    text(move || format!("#{}", id.get()))
}

pub fn board(initial: Vec<Task>) -> impl IntoView {
    let board = Board { tasks: signal(initial), selected: signal(None) };
    let query = signal(String::new());
    let visible = memo(move || {
        let q = query.get();
        board.tasks.with(|all| all.iter().filter(|t| t.title.contains(&q)).cloned().collect::<Vec<_>>())
    });
    // 子孫は context::<Board>() で取得する（兄弟には届かない）
    provide(board, move || {
        column((
            kivact::control::node().name("検索").part(kivact::interaction::use_text_edit(query).control()),
            for_each(visible, |t| t.id, task_row),
            show_some(board.selected, details).fallback(|| text("未選択")),
        ))
    })
}

fn task_row(id: u64, task: Value<Task>) -> impl IntoView {
    // 引数名を `row` にすると row() builder を隠すので避ける
    let board = context::<Board>(); // A missing provider panics at this call.
    let is_selected = memo(move || board.selected.get() == Some(id));
    let visits = var(0u32); // 追跡しない局所変数（useRef 相当）
    row((
        node().role(Role::CheckBox).focusable(true)
            .checked(move || task.get().done)
            .on_press(move || { let done = !task.get_untracked().done; board.tasks.update(|all| set_done(all, id, done)); }),
        text(move || task.get().title),
        { let label = ("選択").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.on_press(move || {
            visits.update(|v| *v += 1);
            board.selected.set(Some(id));
        }),
    ))
    .test_id(format!("row-{id}")) // 同じ部品が並ぶときは within で絞ってもよい
    .selected(is_selected)
}

fn main() {
    let _ = board;
}
