//! Role: The data display page: badges, avatars, progress and a request summary.

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
use super::data::*;
use catalog_shell::{Example, Page};
use super::common::*;
/// The page of the data display components.
pub fn page() -> Page {
    Page::new("display", "Data Display", "Components", include_str!("display.rs"))
        .description("Badges, avatars and progress, which show a state at a glance.")
        .example(Example::new(
            "display",
            "Badges, avatars and progress",
            "Badges take a hue; the progress bar advances on its own.",
            display_demo,
        ))
}

// xtask-allow(state): independent: next_chip counts the chips added by the badge specimen's add button and picks the next suggestion; live is the progress value advanced only by the 900 ms loop
fn display_demo() -> impl IntoView {
    // example: display
    let notify = notify();
    let badges = [
        ui::Hue::Blue,
        ui::Hue::Mint,
        ui::Hue::Yellow,
        ui::Hue::Red,
        ui::Hue::Pink,
        ui::Hue::Gray,
    ]
    .into_iter()
    .zip(["進行中", "完了", "保留", "期限切れ", "NEW", "下書き"])
    .map(|(hue, s)| {
        let badge = kivact::ui::badge(s).variant(ui::Hue::from_badge(hue));
        // Full-width labels retain their 12 px em width, 8 px side padding
        // and 1 px borders when the source's inline row wraps.
        if s.is_ascii() {
            badge
        } else {
            badge.style(style().min_width(s.chars().count() as f64 * 12.0 + 18.0))
        }
    })
    .collect::<Vec<_>>();
    let chip_tags = signal(vec!["爆発".to_string(), "貫通".into(), "神秘".into()]);
    let next_chip = signal(0usize);
    let live = signal(35.0f64);
    spawn::<()>(async move {
        loop {
            kivact::task::sleep(Duration::from_millis(900)).await;
            live.update(|n| {
                *n = if *n >= 100.0 {
                    8.0
                } else {
                    (*n + 7.0).min(100.0)
                }
            });
        }
    });
    section(
        "05",
        "Data Display",
        "バッジ・アバター・プログレス",
        cards_grid((
            third(badge_examples(badges, chip_tags, next_chip)),
            third(avatar_examples()),
            third(progress_examples(live)),
            request_summary(notify).style(style().grid_column(GridPlacement::span(12))),
        )),
    )
    // end example
}

fn progress_ring(percent: f64, label: &'static str, hue: ui::Hue) -> Node {
    let percent: kivact::state::Value<f64> = percent.into_value();
    let label = kivact::state::IntoText::into_text_value(label);
    column((
        kivact::ui::progress(move || Some(percent.get() / 100.0))
            .name(label)
            .variant(kivact::ui::ProgressVariant::Ring)
            .variant(ui::ProgressHue(hue.into_value())),
        text(label).style(style().font_size(12).color(color::TEXT_MUTED)),
    ))
    .style(style().align_items(Align::Center).gap(6))
}

fn badge_examples(
    badges: impl kivact::view::IntoView,
    chip_tags: Signal<Vec<String>>,
    next_chip: Signal<usize>,
) -> kivact::ui::Card {
    card(
        "バッジ・チップ",
        column((
            wrap(badges),
            wrap((
                kivact::ui::badge("オンライン")
                    .variant(ui::Hue::Mint)
                    .variant(ui::BadgeVariant::Dot),
                kivact::ui::badge("離席中")
                    .variant(ui::Hue::Yellow)
                    .variant(ui::BadgeVariant::Dot),
                kivact::ui::badge("オフライン")
                    .variant(ui::Hue::Gray)
                    .variant(ui::BadgeVariant::Dot),
                kivact::ui::badge("Lv.90").variant(ui::BadgeVariant::Solid),
                kivact::ui::badge("EX").variant(ui::BadgeVariant::Slant),
            )),
            ui::chip_list(chip_tags)
                .on_add(move || {
                    let options = [
                        "振動",
                        "分解",
                        "重装甲",
                        "軽装備",
                        "特殊装甲",
                        "市街地",
                        "屋内",
                        "野外",
                    ];
                    let index = next_chip.get();
                    chip_tags.update(|items| items.push(options[index % options.len()].into()));
                    next_chip.set(index + 1);
                })
                .on_remove(move |index| {
                    chip_tags.update(|items| {
                        items.remove(index);
                    })
                })
                .test_id("pop-chip-list"),
        ))
        .style(style().gap(20)),
    )
}

fn avatar_examples() -> kivact::ui::Card {
    card(
        "アバター・ヘイロー",
        column((
            wrap((
                decorated_avatar(
                    kivact::state::IntoText::into_text_value("MN"),
                    AvatarLook::default().size(72).halo(true),
                ),
                decorated_avatar(
                    kivact::state::IntoText::into_text_value("HK"),
                    AvatarLook::default().size(56).hue(330.0).halo(true),
                ),
                decorated_avatar(
                    kivact::state::IntoText::into_text_value("YZ"),
                    AvatarLook::default().hue(45.0).halo(true),
                ),
                decorated_avatar(
                    kivact::state::IntoText::into_text_value("AR"),
                    AvatarLook::default().size(32).hue(160.0),
                ),
                decorated_avatar(
                    kivact::state::IntoText::into_text_value("K"),
                    AvatarLook::default().size(24).hue(265.0),
                ),
            ))
            .style(style().align_items(Align::End)),
            avatar_profile(),
            row((
                decorated_avatar(
                    kivact::state::IntoText::into_text_value("HK"),
                    AvatarLook::default().hue(330.0).halo(true).online(true),
                ),
                column((
                    text("春日 ハルカ").style(style().font_weight(600)),
                    muted("セレスト女学院 · 救護騎士団").style(style().font_size(12)),
                    text("★★★").style(style().color(rgb(0xf2bf0a))),
                ))
                .style(style().flex_grow(1)),
                kivact::ui::button("プロフィール")
                    .size(kivact::ui::ControlSize::Small)
                    .variant(kivact_ui_pop::ButtonTone::Soft),
            ))
            .style(
                style()
                    .padding(14)
                    .gap(14)
                    .align_items(Align::Center)
                    .background(color::SURFACE_ALT)
                    .border(1, color::BORDER)
                    .border_radius(10),
            ),
        ))
        .style(style().gap(20)),
    )
}

fn progress_examples(live: Signal<f64>) -> kivact::ui::Card {
    card(
        "プログレス",
        column((
            column((
                row((muted("EXP").style(style().flex_grow(1)), text("7,820 / 10,000"))),
                kivact::ui::progress(Some(0.78)).name("EXP"),
            ))
            .style(style().gap(8)),
            column((
                row((
                    muted("同期中…").style(style().flex_grow(1)),
                    text(move || {
                        if live.get() >= 100.0 {
                            "完了".to_string()
                        } else {
                            format!("{:.0}%", live.get())
                        }
                    }),
                )),
                kivact::ui::progress(move || Some(live.get() / 100.0))
                    .name("同期中")
                    .variant(ui::Striped),
            ))
            .style(style().gap(8)),
            column((
                row((muted("絆ランク").style(style().flex_grow(1)), text("Rank 14"))),
                kivact::ui::progress(Some(0.6))
                    .name("絆ランク")
                    .variant(ui::Hue::Pink)
                    .variant(ui::Segments(10)),
            ))
            .style(style().gap(8)),
            columns(
                (
                    progress_ring(72.0, "出席率", ui::Hue::Blue),
                    progress_ring(45.0, "予算消化", ui::Hue::Yellow),
                    progress_ring(91.0, "達成率", ui::Hue::Mint),
                ),
                3,
            ),
        ))
        .style(style().gap(16)),
    )
}

fn request_summary(notify: Callback<(ui::Hue, String, String)>) -> kivact::ui::Card {
    kivact::ui::card((
        row((
            text("リスト・キーバリュー").style(style().font_size(15).font_weight(600).flex_grow(1)),
            kivact::ui::button("すべて表示")
                .size(kivact::ui::ControlSize::Small)
                .variant(kivact::ui::ButtonVariant::Ghost)
                .trailing(kivact::view::icon("chevron-right")),
        ))
        .style(style().align_items(Align::Center).margin_bottom(16)),
        columns((document_rows(), request_metadata(notify)), 2),
    ))
}

fn avatar_profile() -> Node {
    row((
        decorated_avatar(
            kivact::state::IntoText::into_text_value("MN"),
            AvatarLook::default().size(32),
        ),
        decorated_avatar(
            kivact::state::IntoText::into_text_value("HK"),
            AvatarLook::default().size(32).hue(330.0),
        )
        .style(style().margin_left(-8)),
        decorated_avatar(
            kivact::state::IntoText::into_text_value("YZ"),
            AvatarLook::default().size(32).hue(45.0),
        )
        .style(style().margin_left(-8)),
        decorated_avatar(
            kivact::state::IntoText::into_text_value("AR"),
            AvatarLook::default().size(32).hue(160.0),
        )
        .style(style().margin_left(-8)),
        node()
            .child(text("+8").style(style().font_size(11).color(color::TEXT_MUTED)))
            .style(
                style()
                    .size(32, 32)
                    .border_radius(999)
                    .margin_left(-8)
                    .background(color::SURFACE_ALT)
                    .direction(Direction::Row)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center),
            ),
        muted("12名が参加中").style(style().font_size(12).margin_left(8)),
    ))
    .style(style().align_items(Align::Center))
}

fn document_rows() -> kivact::ui::ItemList {
    kivact::ui::item_list((
        kivact::ui::item_row("第3章 作戦報告書")
            .description("更新：10/02 22:14 · ミナト")
            .media(kivact::view::icon("file-text"))
            .variant(ui::Hue::Blue)
            .actions({
                row((
                    kivact::ui::badge("承認済")
                        .variant(ui::Hue::from_badge(ui::Hue::Mint.into_value())),
                    kivact::ui::button("報告書メニュー")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("dots-vertical")),
                ))
                .style(style().gap(8))
            }),
        kivact::ui::item_row("弾薬補給 予算申請")
            .description("更新：10/02 18:40 · ユズ")
            .media(kivact::view::icon("coin"))
            .variant(ui::Hue::Gray)
            .actions({
                row((
                    kivact::ui::badge("レビュー待ち")
                        .variant(ui::Hue::from_badge(ui::Hue::Yellow.into_value())),
                    kivact::ui::button("申請メニュー")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("dots-vertical")),
                ))
                .style(style().gap(8))
            }),
        kivact::ui::item_row("救護所 シフト表")
            .description("更新：10/01 09:03 · ハルカ")
            .media(kivact::view::icon("heart"))
            .variant(ui::Hue::Gray)
            .actions({
                row((
                    kivact::ui::badge("下書き").variant(ui::Hue::Gray),
                    kivact::ui::button("シフトメニュー")
                        .variant(kivact::ui::ButtonVariant::Ghost)
                        .hide_label(true)
                        .icon(kivact::view::icon("dots-vertical")),
                ))
                .style(style().gap(8))
            }),
    ))
}

fn request_metadata(notify: Callback<(ui::Hue, String, String)>) -> kivact::ui::DescriptionList {
    kivact::ui::description_list((
        kivact::ui::description(
            "依頼ID",
            row((
                text("REQ-2026-1003").style(
                    style()
                        .background(color::SURFACE_ALT)
                        .font_family("monospace")
                        .padding_x(4)
                        .border_radius(4),
                ),
                kivact::ui::button("依頼IDをコピー")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::view::icon("copy"))
                    .on_press(move || {
                        spawn(kivact::platform::copy_text("REQ-2026-1003"));
                        notify.call((
                            ui::Hue::Blue,
                            "コピーしました".into(),
                            "REQ-2026-1003".into(),
                        ));
                    }),
            )),
        ),
        kivact::ui::description(
            "担当",
            row((
                decorated_avatar(
                    kivact::state::IntoText::into_text_value("M"),
                    AvatarLook::default().size(24),
                ),
                text("空崎 ミナト"),
            ))
            .style(style().gap(8).align_items(Align::Center)),
        ),
        kivact::ui::description(
            "ステータス",
            kivact::ui::badge("進行中").variant(ui::BadgeVariant::Dot),
        ),
        kivact::ui::description("期限", text("2026/10/10 (土) 18:00")),
        kivact::ui::description(
            "エリア",
            row((
                kivact::view::icon("map-pin"),
                text("中央区 外郭地区 第7ブロック"),
            ))
            .style(style().gap(6)),
        ),
    ))
}
