//! Role: Stateful source conversation, including delayed typing and replies.

use super::common::section;
use kivact::draw::linear_gradient;
use kivact::geometry::Rect;
use kivact::prelude::{
    ColorToken, Node, NodeBuilder, column, dynamic, for_each, node, rgb, row, show, signal, text,
};
use kivact::state::callback;
use kivact::style::use_theme;
use kivact::style::{Align, Color, Justify, pct, style};
use kivact::task::spawn;
use kivact::view::IntoView;
use kivact::view::node_ref;
use kivact_ui_pop as ui;
use std::time::Duration;

#[derive(Clone, PartialEq)]
struct ChatEntry {
    id: u64,
    text: String,
    mine: bool,
    typing: bool,
}

pub(super) fn messaging() -> Node {
    let active = signal(0usize);
    let draft = signal(String::new());
    let messages = signal(Vec::<ChatEntry>::new());
    let serial = signal(0u64);
    let unread = signal([2usize, 1, 0, 0]);
    let replies = signal(true);
    let chat_scroll = node_ref();
    let names = ["空崎 ミナト", "春日 ハルカ", "花岡 ユズ", "白石 アリサ"];
    let send = callback(move |s: String| {
        if !s.trim().is_empty() {
            replies.set(false);
            let id = serial.get();
            serial.set(id + 3);
            messages.update(|entries| {
                entries.push(ChatEntry {
                    id,
                    text: s,
                    mine: true,
                    typing: false,
                })
            });
            draft.set(String::new());
            spawn(async move {
                kivact::task::sleep(Duration::from_millis(400)).await;
                messages.update(|entries| {
                    entries.push(ChatEntry {
                        id: id + 1,
                        text: String::new(),
                        mine: false,
                        typing: true,
                    })
                });
                kivact::task::sleep(Duration::from_millis(1300)).await;
                let replies = [
                    "わかりました！すぐに確認しますね。",
                    "先生、ありがとうございます♪",
                    "えへへ……がんばります！",
                    "了解です。他にお手伝いできることはありますか？",
                ];
                messages.update(|entries| {
                    entries.retain(|entry| entry.id != id + 1);
                    entries.push(ChatEntry {
                        id: id + 2,
                        text: replies[(id / 3) as usize % replies.len()].into(),
                        mine: false,
                        typing: false,
                    });
                });
            });
        }
    });
    let threads = [
        ("MN", "空崎 ミナト", "報告書、置いておきました！", 2),
        ("HK", "春日 ハルカ", "救護所のシフト、確認お願いします", 1),
        ("YZ", "花岡 ユズ", "ゲーム開発部に遊びに来ませんか", 0),
        ("AR", "白石 アリサ", "スタンプを送信しました", 0),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (initials, name, preview, _))| {
        ui::thread(initials, name)
            .preview(preview)
            .unread(move || unread.with(|counts| counts[i]))
            .with_node(|node| node.selected(move || active.get() == i))
            .portrait({
                {
                    let initials = kivact::state::IntoText::into_text_value(initials);
                    let look = ui::AvatarLook::default()
                        .size(32)
                        .hue([205.0, 330.0, 45.0, 160.0][i]);
                    kivact::ui::avatar(initials)
                        .fallback_content(move || kivact::view::text(initials))
                        .variant(look)
                        .status_content(look.decorations())
                }
            })
            .test_id(format!("pop-thread-{i}"))
            .on_press(move || {
                active.set(i);
                unread.update(|counts| counts[i] = 0);
            })
    })
    .collect::<Vec<_>>();
    let list = column((
        row((
            text("トーク").style(
                style()
                    .grow(1)
                    .font_size(16)
                    .weight(600)
                    .color(Color::WHITE),
            ),
            kivact::ui::button("新規トーク")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::view::icon("pencil"))
                .style(style().color(Color::WHITE)),
        ))
        .style(
            style()
                .height(56)
                .padding_x(18)
                .align_items(Align::Center)
                .background(linear_gradient(90).colors([rgb(0xf2559a), rgb(0xff7cb0)]))
                .color(Color::WHITE),
        ),
        column(threads).style(style().padding(8)),
    ))
    .style(
        style()
            .width(280)
            .shrink(0)
            .background(ColorToken::SurfaceAlt)
            .border_right(1, ColorToken::Border),
    );
    let body = kivact::ui::chat_thread(
        column((
            text("今日").style(
                style()
                    .font_size(12)
                    .align_self(Align::Center)
                    .padding_x(10)
                    .padding_y(2)
                    .background(ColorToken::SurfaceAlt)
                    .radius(999)
                    .color(ColorToken::TextMuted),
            ),
            {
                let mine: kivact::state::Value<bool> = kivact::state::IntoValue::into_value(false);
                let initials = kivact::state::IntoText::into_text_value("M");
                let time = kivact::state::IntoText::into_text_value("09:12");
                let paragraphs: kivact::state::Value<Vec<String>> =
                    kivact::state::IntoValue::into_value(vec![
                        "先生、おはようございます。".into(),
                        "第3章の報告書、デスクに置いておきました！".into(),
                    ]);
                kivact::view::column(kivact::view::dynamic(paragraphs, move |paragraphs| {
                    let count = paragraphs.len();
                    paragraphs
                        .into_iter()
                        .enumerate()
                        .map(move |(index, paragraph)| {
                            let group = if count == 1 {
                                kivact::ui::ChatBubbleGroup::Single
                            } else if index == 0 {
                                kivact::ui::ChatBubbleGroup::First
                            } else if index + 1 == count {
                                kivact::ui::ChatBubbleGroup::Last
                            } else {
                                kivact::ui::ChatBubbleGroup::Middle
                            };
                            let mut message = kivact::ui::chat_message(initials)
                                .sender(move || {
                                    if mine.get() {
                                        kivact::ui::ChatSender::User
                                    } else {
                                        kivact::ui::ChatSender::Assistant
                                    }
                                })
                                .content(kivact::view::text(paragraph))
                                .group(group)
                                .variant(ui::PopBubble);
                            if index == 0 {
                                message = message.avatar({
                                    let look = ui::AvatarLook::default().size(24);
                                    kivact::ui::avatar(initials)
                                        .fallback_content(move || kivact::view::text(initials))
                                        .variant(look)
                                        .status_content(look.decorations())
                                });
                            }
                            if index + 1 == count {
                                message = message.time(time);
                            }
                            message
                        })
                        .collect::<Vec<_>>()
                }))
                .style(style().gap(4))
            },
            {
                let mine: kivact::state::Value<bool> = kivact::state::IntoValue::into_value(true);
                let initials = kivact::state::IntoText::into_text_value("");
                kivact::ui::chat_message(initials)
                    .sender(move || {
                        if mine.get() {
                            kivact::ui::ChatSender::User
                        } else {
                            kivact::ui::ChatSender::Assistant
                        }
                    })
                    .content(kivact::view::text("ありがとう、助かるよ。あとで読むね。"))
                    .time("既読 09:14")
                    .variant(ui::PopBubble)
                    .avatar({
                        let look = ui::AvatarLook::default().size(24);
                        kivact::ui::avatar(initials)
                            .fallback_content(move || kivact::view::text(initials))
                            .variant(look)
                            .status_content(look.decorations())
                    })
            },
            {
                let mine: kivact::state::Value<bool> = kivact::state::IntoValue::into_value(false);
                let initials = kivact::state::IntoText::into_text_value("M");
                kivact::ui::chat_message(initials)
                    .sender(move || {
                        if mine.get() {
                            kivact::ui::ChatSender::User
                        } else {
                            kivact::ui::ChatSender::Assistant
                        }
                    })
                    .content(kivact::view::text(
                        "それと、今日のお昼……少しお時間ありますか？",
                    ))
                    .time("09:15")
                    .variant(ui::PopBubble)
                    .avatar({
                        let look = ui::AvatarLook::default().size(24);
                        kivact::ui::avatar(initials)
                            .fallback_content(move || kivact::view::text(initials))
                            .variant(look)
                            .status_content(look.decorations())
                    })
            },
            show(replies, move || {
                column((
                    text("返信を選択")
                        .style(style().font_size(12).weight(500).padding_left(4))
                        .style(move || {
                            let t = &use_theme().get();
                            style().color(t.ext::<ui::Tokens>().hue(ui::Hue::Pink).1)
                        }),
                    kivact::ui::button("もちろん、一緒に食べよう")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .style(style().justify_content(Justify::Start))
                        .on_press(move || {
                            send.call("もちろん、一緒に食べよう".into());
                            replies.set(false);
                        }),
                    kivact::ui::button("ごめん、会議が入ってる")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .style(style().justify_content(Justify::Start))
                        .on_press(move || {
                            send.call("ごめん、会議が入ってる".into());
                            replies.set(false);
                        }),
                ))
                .test_id("pop-reply-card")
                .style(
                    style()
                        .gap(6)
                        .padding(10)
                        .width(300)
                        .max_width(pct(90))
                        .align_self(Align::End)
                        .radius(8),
                )
                .style(move || {
                    let t = &use_theme().get();
                    let p = t.ext::<ui::Tokens>();
                    style()
                        .background(p.hue(ui::Hue::Pink).0)
                        .border(1, p.pink.mix_in(Color::TRANSPARENT, 0.7, t))
                })
            }),
            for_each(
                messages,
                |entry| entry.id,
                move |entry| {
                    node().child(dynamic(entry, |entry| {
                        if entry.typing {
                            {
                                let initials = kivact::state::IntoText::into_text_value("M");
                                kivact::ui::typing_indicator()
                                    .avatar({
                                        let look = ui::AvatarLook::default().size(24);
                                        kivact::ui::avatar(initials)
                                            .fallback_content(move || kivact::view::text(initials))
                                            .variant(look)
                                            .status_content(look.decorations())
                                    })
                                    .into_view()
                            }
                        } else {
                            {
                                let mine: kivact::state::Value<bool> =
                                    kivact::state::IntoValue::into_value(entry.mine);
                                let initials = kivact::state::IntoText::into_text_value("M");
                                kivact::ui::chat_message(initials)
                                    .sender(move || {
                                        if mine.get() {
                                            kivact::ui::ChatSender::User
                                        } else {
                                            kivact::ui::ChatSender::Assistant
                                        }
                                    })
                                    .content(kivact::view::text(entry.text))
                                    .time(if entry.mine { "既読 09:16" } else { "09:16" })
                                    .variant(ui::PopBubble)
                                    .avatar({
                                        let look = ui::AvatarLook::default().size(24);
                                        kivact::ui::avatar(initials)
                                            .fallback_content(move || kivact::view::text(initials))
                                            .variant(look)
                                            .status_content(look.decorations())
                                    })
                                    .into_view()
                            }
                        }
                    }))
                },
            ),
        ))
        .style(style().gap(14).padding_x(20).padding_y(18)),
    )
    .viewport_ref(chat_scroll)
    .style(style().grow(1).min_height(0))
    .draw(|d, s| {
        for x in (0..s.width as usize).step_by(18) {
            for y in (0..s.height as usize).step_by(18) {
                d.rounded_rect(
                    Rect::new(x as f64, y as f64, 2.0, 2.0),
                    1.0,
                    ColorToken::Border,
                );
            }
        }
    });
    let main = column((
        row((
            {
                let initials = kivact::state::IntoText::into_text_value("MN");
                let look = ui::AvatarLook::default().size(32).halo(true).online(true);
                kivact::ui::avatar(initials)
                    .fallback_content(move || kivact::view::text(initials))
                    .variant(look)
                    .status_content(look.decorations())
            },
            column((
                text(move || names[active.get()].to_string()).style(style().weight(600)),
                text("調査部 · オンライン").style(style().font_size(12).color(rgb(0x0f8a6a))),
            ))
            .style(style().grow(1)),
            kivact::ui::button("トークを検索")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::view::icon("search")),
            kivact::ui::button("トーク操作")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::view::icon("dots-vertical")),
        ))
        .style(
            style()
                .height(56)
                .padding_x(18)
                .gap(12)
                .align_items(Align::Center)
                .border_bottom(1, ColorToken::Border),
        ),
        body,
        kivact::ui::chat_composer(draft)
            .placeholder("メッセージを入力…")
            .on_submit(move |message| send.call(message))
            .header_actions(
                kivact::ui::button("添付")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::view::icon("paperclip")),
            )
            .footer_actions(
                kivact::ui::button("スタンプ")
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .hide_label(true)
                    .icon(kivact::view::icon("mood-smile"))
                    .on_press(move || send.call("★".into())),
            )
            .send_icon(kivact::view::icon("send")),
    ))
    .style(style().grow(1).min_width(0));
    section(
        "10",
        "Messaging",
        "トーク — 生徒とのやりとり",
        kivact::ui::card(row((list, main)).style(style().height(560)))
            .style(style().padding(0).clip(true)),
    )
}
