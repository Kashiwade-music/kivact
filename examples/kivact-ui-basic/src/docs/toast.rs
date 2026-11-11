//! Role: Catalog page of Toast: pushed notifications, variants, actions and durations.

use crate::model::{Example, Page, PreviewAlign};
use kivact::prelude::*;
use kivact::ui::ControlSize;
use kivact_ui_basic as basic;
use std::time::Duration;

pub(crate) fn page() -> Page {
    Page {
        slug: "toast",
        name: "Toast",
        description: "操作の結果などを一時的に知らせる通知です。標準の toast_region(toaster()).hold_all_on_hover(true) をアプリのルートに一度だけ置き、toaster().push(Toast::new(題名).tone(tone)) で積みます。二行目は body、長さは duration、操作は action のように Toast の method で足し、icon は tone ごとに kit の theme が描きます。複数あるときは新しい通知の後ろに小さく重なり、ポインターを重ねるかフォーカスすると間隔を空けた一覧に開きます。開いている間はすべての通知の時間が止まり、Escape で閉じます。",
        usage: "use kivact::prelude::*;\nuse kivact::ui::{ControlSize};
use kivact::interaction::{Toast, Tone, toaster};\nuse kivact_ui_basic as basic;\nuse std::time::Duration;",
        source: include_str!("toast.rs"),
        align: PreviewAlign::Center,
        examples: vec![
            Example::new(
                "default",
                "Default",
                "題名と body を持つ通知です。5 秒後に消え、右下に積まれます。toast_region はこのカタログのルートに置いてあります。",
                default,
            ),
            Example::new(
                "variants",
                "Variants",
                "tone で Neutral、Success、Info、Warning、Danger を選びます。Warning と Danger は支援技術へ強めに読み上げられます。Loading は icon に spinner を渡し、duration を None にして閉じるまで残します。",
                variants,
            ),
            Example::new(
                "action",
                "With action",
                "action でボタンを足します。押すとコールバックを呼んで通知を閉じます。コールバックは setup で作り、ハンドラから渡します。",
                with_action,
            ),
            Example::new(
                "duration",
                "Duration",
                "duration で表示する長さを変えます。None なら閉じるまで残るので、戻り値の ToastId で閉じられます。",
                duration,
            ),
        ],
    }
}

// example: default
fn default() -> impl IntoView {
    button("Show Toast")
        .variant(ButtonVariant::Outline)
        .on_press(|| {
            toaster().push(
                Toast::new("Event has been created").body("Sunday, December 03, 2023 at 9:00 AM"),
            );
        })
}
// end example

// example: variants
fn variants() -> impl IntoView {
    let show = |label: &'static str, tone: Tone| {
        button(label)
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small)
            .on_press(move || {
                toaster().push(Toast::new(format!("{label} toast")).tone(tone));
            })
    };
    row((
        show("Default", Tone::Neutral),
        show("Success", Tone::Success),
        show("Info", Tone::Info),
        show("Warning", Tone::Warning),
        show("Error", Tone::Danger),
        button("Loading")
            .variant(ButtonVariant::Outline)
            .size(ControlSize::Small)
            .on_press(|| {
                // The spinner is a status of its own; the toast already announces it.
                toaster().push(
                    Toast::new("Loading toast")
                        .icon(|| kivact::ui::spinner().with_node(|node| node.role(Role::Generic)))
                        .duration(None),
                );
            }),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example

// example: action
fn with_action() -> impl IntoView {
    let undone = signal(0);
    column((
        button("Create Event")
            .variant(ButtonVariant::Outline)
            .on_press(move || {
                toaster().push(
                    Toast::new("Event created")
                        .action("Undo", move || undone.update(|count| *count += 1)),
                );
            }),
        basic::muted(move || format!("Undone {} time(s).", undone.get())),
    ))
    .style(style().gap(12).align_items(Align::Center))
}
// end example

// example: duration
fn duration() -> impl IntoView {
    let last = signal(None);
    row((
        button("For 2 seconds")
            .variant(ButtonVariant::Outline)
            .on_press(|| {
                toaster().push(Toast::new("Gone soon").duration(Some(Duration::from_secs(2))));
            }),
        button("Until dismissed")
            .variant(ButtonVariant::Outline)
            .on_press(move || {
                last.set(Some(toaster().push(Toast::new("Stays").duration(None))));
            }),
        button("Dismiss it")
            .variant(ButtonVariant::Ghost)
            .on_press(move || {
                if let Some(id) = last.get() {
                    toaster().dismiss(id);
                }
            }),
    ))
    .style(style().gap(8).align_items(Align::Center))
}
// end example
