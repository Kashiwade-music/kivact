//! Role: The buttons page: the button variants, sizes, groups and the standard buttons.

use kivact::prelude::*;
use kivact::style::{Align, Color, GridPlacement, Justify, Track, color, pct, rgb, rgba, style};
use kivact::view::Node;
use kivact_ui_pop as ui;
use kivact::draw::{LineCap, LineJoin, Path, Stroke};
use kivact::geometry::Point;
use kivact::i18n::{CalendarDate, Weekday};
use kivact::interaction::{ListItem, MenuEntry, ScalarRange};
use kivact::motion::{Animated, Easing, Repeat, Transition};
use kivact::state::{Callback, IntoHandler, IntoText, IntoValue, Machine, MachineHandle, Signal, Step, Value, callback, machine};
use kivact::style::{Direction, Filter, Shadow, Shape, TextDecorationLine, linear_gradient};
use kivact::ui::RadioGroup;
use kivact::view::{Orientation, Placement, rich_text, span as text_span};
use kivact_ui_pop::Hue;
use std::time::Duration;
use catalog_shell::{Example, Page};
use super::common::*;
/// The page of the buttons.
pub fn page() -> Page {
    Page::new("buttons", "Buttons", "Components", include_str!("buttons.rs"))
        .description("Buttons come in variants, sizes and tones; the standard button takes the same theme.")
        .example(Example::new(
            "buttons",
            "Button variants",
            "The primary action is blue; the special moment is yellow.",
            buttons_demo,
        ))
}

/// The sync button: idle, or busy until the delayed finish event arrives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SyncPhase {
    #[default]
    Idle,
    Running,
}

enum SyncEvent {
    Start,
    Finish,
}

/// What the sync machine asks the page to do after a transition.
enum SyncEffect {
    Finished,
}

struct SyncMachine;

impl Machine for SyncMachine {
    type State = SyncPhase;
    type Event = SyncEvent;
    type Effect = SyncEffect;
    fn transition(
        &self,
        state: &SyncPhase,
        event: SyncEvent,
    ) -> Step<SyncPhase, SyncEvent, SyncEffect> {
        match (state, event) {
            (SyncPhase::Idle, SyncEvent::Start) => {
                Step::to(SyncPhase::Running).after(Duration::from_millis(1600), SyncEvent::Finish)
            }
            (SyncPhase::Running, SyncEvent::Finish) => {
                Step::to(SyncPhase::Idle).with(SyncEffect::Finished)
            }
            _ => Step::stay(),
        }
    }
}

fn buttons_demo() -> impl IntoView {
    // example: buttons
    let notify = notify();
    use kivact::ui::ButtonVariant;
    use kivact_ui_pop::ButtonTone;
    let variants = [
        ("プライマリ", ButtonVariant::Primary, None),
        (
            "アクセント",
            ButtonVariant::Primary,
            Some(ButtonTone::Accent),
        ),
        (
            "セカンダリ",
            ButtonVariant::Secondary,
            Some(ButtonTone::Soft),
        ),
        ("アウトライン", ButtonVariant::Secondary, None),
        ("ゴースト", ButtonVariant::Ghost, None),
        ("削除する", ButtonVariant::Danger, None),
    ]
    .into_iter()
    .map(|(label, v, hue)| {
        let button = kivact::ui::button(label).variant(v);
        let button = match hue {
            Some(hue) => button.variant(hue),
            None => button,
        };
        button.on_press(move || notify.call((ui::Hue::Blue, "アクション".into(), label.into())))
    })
    .chain(std::iter::once(kivact::ui::button("無効").disabled(true)))
    .collect::<Vec<_>>();
    section(
        "02",
        "Buttons",
        "ボタン — 主要アクションは青、特別な瞬間は黄色",
        cards_grid((
            half(card("バリエーション", wrap(variants))),
            half(card(
                "スラント（ゲームUI調）",
                wrap((
                    kivact::ui::button("編成する").variant(kivact::ui::Slant),
                    kivact::ui::button("出撃")
                        .variant(kivact_ui_pop::ButtonTone::Accent)
                        .variant(kivact::ui::Slant)
                        .icon(kivact::view::icon("sword")),
                    kivact::ui::button("キャンセル")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .variant(kivact::ui::Slant),
                    kivact::ui::button("詳細")
                        .variant(kivact_ui_pop::ButtonTone::Soft)
                        .size(kivact::ui::ControlSize::Small)
                        .variant(kivact::ui::Slant),
                )),
            )),
            half(button_sizes()),
            half(button_groups(notify)),
            half(card(
                "標準部品（kivact::ui）— テーマの recipe でこのキットの見た目になる",
                column((
                    wrap((
                        kivact::ui::button("標準ボタン").test_id("standard-button"),
                        kivact::ui::button("標準ゴースト")
                            .variant(kivact::ui::ButtonVariant::Ghost),
                        kivact::ui::button("標準無効").disabled(true),
                    )),
                    wrap((
                        kivact::ui::checkbox("標準チェック", signal(true)),
                        kivact::ui::switch("標準スイッチ", signal(true)),
                    )),
                    kivact::ui::text_input("標準入力", signal("標準の入力欄".to_string()))
                        .style(style().width(240)),
                ))
                .style(style().gap(16)),
            )),
        )),
    )
    // end example
}

fn button_sizes() -> kivact::ui::Card {
    card(
        "サイズとアイコン",
        column((
            wrap((
                kivact::ui::button("Small")
                    .size(kivact::ui::ControlSize::Small)
                    .icon(kivact::view::icon("plus")),
                kivact::ui::button("Medium").icon(kivact::view::icon("plus")),
                kivact::ui::button("Large")
                    .size(kivact::ui::ControlSize::Large)
                    .icon(kivact::view::icon("plus")),
            )),
            wrap((
                kivact::ui::button("エクスポート")
                    .variant(kivact::ui::ButtonVariant::Secondary)
                    .trailing(kivact::view::icon("download")),
                kivact::ui::tooltip("追加")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("追加")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("plus"))
                            .style(style().background(color::ACCENT).color(Color::WHITE)),
                    ),
                kivact::ui::tooltip("編集")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("編集")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("pencil"))
                            .style(style().background(color::SURFACE_ALT)),
                    ),
                kivact::ui::tooltip("コピー")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("コピー")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("copy"))
                            .style(style().border(1, color::BORDER)),
                    ),
                kivact::ui::tooltip("その他")
                    .placement(kivact::view::Placement::Below)
                    .align(kivact::view::OverlayAlign::Center)
                    .offset(6.0)
                    .attach(
                        kivact::ui::button("その他")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("dots")),
                    ),
                node()
                    .role(Role::Button)
                    .name("新規作成")
                    .focusable(true)
                    .style(
                        style()
                            .size(44, 44)
                            .rotate(45)
                            .border_radius(10)
                            .background(rgb(0xffdc3c))
                            .align_items(Align::Center)
                            .justify_content(Justify::Center)
                            .focus(style().ring(3, color::FOCUS)),
                    )
                    .child(kivact::view::icon("sparkles").style(style().rotate(-45))),
            )),
        ))
        .style(style().gap(16)),
    )
}

fn button_groups(notify: Callback<(ui::Hue, String, String)>) -> kivact::ui::Card {
    let sync = machine(
        SyncPhase::Idle,
        || SyncMachine,
        move |_, effect| match effect {
            SyncEffect::Finished => notify.call((
                ui::Hue::Mint,
                "同期完了".into(),
                "最新のデータを取得しました。".into(),
            )),
        },
    );
    card(
        "グループ・ローディング",
        column((
            wrap((
                view_mode_buttons(),
                kivact::ui::button(move || {
                    if sync.get() == SyncPhase::Running {
                        "同期中…"
                    } else {
                        "同期する"
                    }
                    .to_string()
                })
                .icon(kivact::view::icon("refresh"))
                .busy(move || sync.get() == SyncPhase::Running)
                .on_press(move || sync.send(SyncEvent::Start)),
            )),
            wrap((
                row((
                    kivact::ui::button("保存して公開")
                        .style(style().border_radius(0))
                        .on_press(move || {
                            notify.call((
                                ui::Hue::Mint,
                                "保存しました".into(),
                                "変更が反映されました。".into(),
                            ))
                        }),
                    kivact::ui::menu(
                        "保存方法",
                        vec![
                            MenuEntry::item("draft", "下書き保存"),
                            MenuEntry::item("schedule", "予約公開…"),
                        ],
                    )
                    .on_activate(|_: &'static str| {})
                    .attach(
                        kivact::ui::button("保存方法")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .hide_label(true)
                            .icon(kivact::view::icon("chevron-down"))
                            .style(
                                style()
                                    .border_radius(0)
                                    .background(color::ACCENT)
                                    .color(Color::WHITE)
                                    .border_left(1, rgba(0xffffff, 0.2)),
                            ),
                    ),
                ))
                .style(style().border_radius(8).overflow(kivact::style::Overflow::Hidden)),
                kivact::ui::button("もっと見る")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .size(kivact::ui::ControlSize::Small)
                    .trailing(kivact::view::icon("arrow-right")),
            )),
        ))
        .style(style().gap(16)),
    )
}



fn view_mode_buttons() -> Node {
    row((
        kivact::ui::button("グリッド")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .icon(kivact::view::icon("layout-grid"))
            .style(style().border_radius_corners(8, 0, 0, 8)),
        kivact::ui::button("リスト")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .icon(kivact::view::icon("list"))
            .style(style().border_radius(0).margin_left(-1)),
        kivact::ui::button("フィルタ")
            .variant(kivact::ui::ButtonVariant::Ghost)
            .hide_label(true)
            .icon(kivact::view::icon("filter"))
            .style(
                style()
                    .border(1, color::BORDER)
                    .border_radius_corners(0, 8, 8, 0)
                    .margin_left(-1),
            ),
    ))
}
