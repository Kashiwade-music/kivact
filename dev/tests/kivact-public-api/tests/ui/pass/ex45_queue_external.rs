#![allow(dead_code, unused_variables)]
// Usage example: ordered saving (queued action) and external objects
use kivact::motion::{FrameInfo, frame_loop};
use kivact::prelude::*;
use kivact::{effect_on};
use kivact::motion::{FrameRate};
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
}

fn saving(path: std::path::PathBuf) -> impl IntoView {
    let tasks = signal(Vec::<Task>::new());
    let saver = action(move |board: Vec<Task>| {
        let path = path.clone(); // path: PathBuf（handle ではない通常の値）
        async move { Ok(blocking(move || storage::save(&path, &board)).await?) }
    })
    .policy(ActionPolicy::QueueLatest);
    effect_on(tasks, move |board| saver.run(board.clone())); // 変化するたびに保存を予約する（初回は飛ばす）
    text(move || match saver.state() {
        AsyncState::Idle | AsyncState::Ready(()) => "保存しました".into(),
        AsyncState::Pending => "保存中".into(),
        AsyncState::Failed(error) => format!("保存できません: {error}"),
    })
}

fn player_view() -> impl IntoView {
    // Clone も PartialEq もない外部 object も、Signal に入れてよい
    let player = signal(AudioPlayer::new());
    let play = { let label = ("再生").into_value(); node().role(Role::Button).focusable(true).name(label).child(text(label)) }.on_press(move || player.update(|p| p.toggle())); // update は常に通知する
    let frames = frame_loop(move |_f: FrameInfo| player.notify()); // 内部で変化したことを毎 frame 通知
    effect(move || if player.with(|p| p.is_playing()) { frames.start(FrameRate::Display) } else { frames.stop() });
    column((play, text(move || player.with(|p| format_time(p.position())))))
}

fn main() {
    let _ = saving;
    let _ = player_view;
}
