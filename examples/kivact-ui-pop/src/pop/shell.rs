//! Role: The catalog shell: top bar, section sidebar, command palette and overview.

use super::common::{columns, tone_of, wrap};
use super::components::buttons;
use super::components::forms;
use super::components::tokens;
use super::data::display;
use super::data::table;
use super::hero::hero;
use super::keyed_catalog::selection;
use super::messaging::messaging;
use super::patterns::cards;
use super::patterns::feedback;
use super::patterns::navigation;
use super::patterns::overlays;
use super::patterns::schedule;
use kivact::draw::{LineCap, LineJoin, Stroke, linear_gradient};
use kivact::geometry::Point;
use kivact::i18n::CalendarDate;
use kivact::interaction::{Command, ListItem, Toast, toaster};
use kivact::prelude::{
    ColorToken, IntoView, Key, Node, NodeBuilder, NodeRef, Role, Signal, column, effect, node, rgb,
    row, scroll, signal, text,
};
use kivact::state::callback;
use kivact::style::{Align, Color, Justify, pct, rgba, style};
use kivact::task::spawn;
use kivact::ui::command_palette;
use kivact::view::ScrollBehavior;
use kivact::view::{Handled, KeyEvent, node_ref};
use kivact_ui_pop as ui;
use std::rc::Rc;
use std::time::Duration;

const SIDEBAR_WIDTH: f64 = 256.0;

const TOPBAR_HEIGHT: f64 = 56.0;

/// What the command palette can do; the key of each of its commands.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PaletteCommand {
    Section(usize),
    Theme,
    Create,
    Start,
    Profile,
    Reward,
}

const SECTIONS: [(&str, &str, &str); 13] = [
    ("overview", "ダッシュボード", "home"),
    ("tokens", "デザイントークン", "palette"),
    ("buttons", "ボタン", "bolt"),
    ("forms", "フォーム", "pencil"),
    ("selection", "セレクション", "adjustments-horizontal"),
    ("display", "データ表示", "sparkles"),
    ("table", "テーブル", "table"),
    ("feedback", "フィードバック", "bell"),
    ("overlays", "オーバーレイ", "layout-grid"),
    ("navigation", "ナビゲーション", "map-pin"),
    ("messaging", "メッセージ", "message-circle"),
    ("schedule", "スケジュール", "calendar"),
    ("cards", "ミッションカード", "target"),
];

/// Mount the complete catalog through public component APIs.
pub fn catalog() -> impl IntoView {
    catalog_with_mode(kivact::style::ThemeMode::Light)
}

/// The same catalogue with an explicit initial palette for comparisons.
fn catalog_with_mode(initial: kivact::style::ThemeMode) -> impl IntoView {
    let dark = signal(initial == kivact::style::ThemeMode::Dark);
    let theme = move || {
        let t = ui::theme(if dark.get() {
            kivact::style::ThemeMode::Dark
        } else {
            kivact::style::ThemeMode::Light
        });
        let text = kivact::style::theme::Typography {
            family: Some("Inter, 'Noto Sans JP', sans-serif".into()),
            ..t.text().clone()
        };
        t.with_text(text)
    };
    toaster().max_visible(4);
    let selected = signal(0usize);
    let activity_points = signal(128u16);
    spawn::<()>(async move {
        loop {
            kivact::task::sleep(Duration::from_secs(6)).await;
            activity_points.update(|points| *points = points.saturating_add(1).min(160));
        }
    });
    let scroll_ref = node_ref();
    let content_ref = node_ref();
    let refs: Rc<Vec<NodeRef>> = Rc::new((0..SECTIONS.len()).map(|_| node_ref()).collect());
    let confirm = signal(false);
    let form_dialog = signal(false);
    let sheet = signal(false);
    let drawer_title = signal("空崎 ミナト".to_string());
    let command = signal(false);
    let remember_confirmation = signal(false);
    let request_title = signal(String::new());
    let request_priority = signal(Some("normal"));
    let request_detail = signal(String::new());
    let request_date = signal(CalendarDate::new(2026, 10, 10));
    let pin_student = signal(true);
    let auto_skill = signal(false);
    let notify = callback(move |(hue, title, body): (ui::Hue, String, String)| {
        let mut toast = Toast::new(title)
            .tone(tone_of(hue))
            .body(body)
            .duration(Some(Duration::from_millis(4200)));
        if toast.title.contains("報酬") {
            toast = toast.icon(|| kivact::view::icon("gift"));
        }
        if toast.title.contains("作戦開始") {
            toast = toast.icon(|| kivact::view::icon("sword"));
        }
        if toast.title == "コピーしました" {
            toast = toast
                .icon(|| kivact::view::icon("copy"))
                .duration(Some(Duration::from_millis(2200)));
        }
        toaster().push(toast);
    });
    let action = move || confirm.set(true);
    let sections = vec![
        overview(action, command),
        tokens(notify),
        buttons(notify),
        forms(),
        selection(),
        display(notify),
        table(sheet, drawer_title),
        feedback(notify),
        overlays(confirm, form_dialog, sheet, command),
        navigation(notify),
        messaging(),
        schedule(),
        cards(notify),
    ];
    let section_views = sections
        .into_iter()
        .enumerate()
        .map(|(i, view)| {
            view.with_node(|node| node.node_ref(refs[i]))
                .test_id(format!("pop-section-{}", SECTIONS[i].0))
                .style(style().margin_bottom(48))
        })
        .collect::<Vec<_>>();
    let spy_refs = refs.clone();
    effect(move || {
        let offset = scroll_ref.scroll_offset().get();
        let mut current = 0;
        for (i, r) in spy_refs.iter().enumerate() {
            if r.rect().get().is_some_and(|rect| rect.y <= 300.0) {
                current = i;
            }
        }
        if offset.y > 0.0
            && content_ref
                .rect()
                .get()
                .zip(scroll_ref.rect().get())
                .is_some_and(|(content, viewport)| {
                    content.y + content.height <= viewport.y + viewport.height + 2.0
                })
        {
            current = SECTIONS.len() - 1;
        }
        selected.set(current);
    });
    let nav = SECTIONS
        .iter()
        .enumerate()
        .flat_map(|(i, (_, title, glyph))| {
            let mut nodes = Vec::new();
            if i == 0 || i == 2 || i == 10 {
                nodes.push(
                    text(if i == 0 {
                        "Overview"
                    } else if i == 2 {
                        "Components"
                    } else {
                        "Patterns"
                    })
                    .style(
                        style()
                            .padding_top(16)
                            .padding_bottom(6)
                            .padding_x(8)
                            .font_size(12)
                            .weight(500)
                            .color(ColorToken::TextMuted),
                    ),
                );
            }
            let target = refs[i];
            let trailing = (
                (i == 6)
                    .then(|| text("24").style(style().font_size(12).color(ColorToken::TextMuted))),
                (i == 10).then(|| {
                    ui::badge("3")
                        .hue(ui::Hue::Pink)
                        .variant(ui::BadgeVariant::Count)
                }),
            );
            nodes.push(
                row((
                    kivact::view::icon(*glyph).style(move || {
                        style().color(if selected.get() == i {
                            ColorToken::Accent
                        } else {
                            ColorToken::TextMuted
                        })
                    }),
                    text(*title).style(style().grow(1)),
                    trailing,
                ))
                .role(Role::Button)
                .name(*title)
                .test_id(format!("pop-nav-{}", SECTIONS[i].0))
                .focusable(true)
                .with_node(|node| node.selected(move || selected.get() == i))
                .style(
                    style()
                        .height(32)
                        .padding_x(8)
                        .gap(8)
                        .align_items(Align::Center)
                        .radius(8)
                        .hover(style().background(ColorToken::SurfaceAlt))
                        .selected(style().background(ColorToken::SurfaceAlt).weight(500))
                        .focus(style().ring(3, ColorToken::Focus)),
                )
                .on_press(move || {
                    if let Some(r) = target.rect().get() {
                        let offset = scroll_ref.scroll_offset().get();
                        scroll_ref.scroll_to(
                            Point::new(0.0, (offset.y + r.y - 76.0).max(0.0)),
                            ScrollBehavior::Smooth,
                        );
                    }
                    selected.set(i);
                }),
            );
            nodes
        })
        .collect::<Vec<_>>();
    let sidebar = column((
        row((
            node()
                .style(
                    style()
                        .size(32, 32)
                        .radius(8)
                        .background(linear_gradient(135).colors([rgb(0x2b8bd8), rgb(0x5cbde8)])),
                )
                .draw(|d, _| {
                    let ellipse = kivact::draw::Path::new()
                        .move_to(Point::new(8.0, 10.0))
                        .cubic_to(
                            Point::new(8.0, 4.0),
                            Point::new(24.0, 4.0),
                            Point::new(24.0, 10.0),
                        )
                        .cubic_to(
                            Point::new(24.0, 16.0),
                            Point::new(8.0, 16.0),
                            Point::new(8.0, 10.0),
                        );
                    d.stroke_path(
                        &ellipse,
                        &Stroke::new(2.0).cap(LineCap::Round).join(LineJoin::Round),
                        Color::WHITE,
                    );
                    d.fill_path(
                        &kivact::draw::Path::new()
                            .move_to(Point::new(9.0, 26.0))
                            .line_to(Point::new(16.0, 14.0))
                            .line_to(Point::new(23.0, 26.0))
                            .close(),
                        Color::WHITE,
                    );
                }),
            column((
                text("POP UI").style(
                    style()
                        .font_family("Barlow")
                        .italic(true)
                        .weight(800)
                        .font_size(17)
                        .line_height(1.1),
                ),
                text("Soft SaaS Components")
                    .style(style().font_size(11.5).color(ColorToken::TextMuted)),
            )),
        ))
        .style(
            style()
                .gap(10)
                .align_items(Align::Center)
                .padding_x(8)
                .padding_top(6)
                .padding_bottom(14),
        ),
        scroll(column(nav).style(style().gap(1))).style(style().grow(1).min_height(0)),
        column((
            ui::panel((
                row((
                    text("活動ポイント")
                        .style(style().font_size(12).color(ColorToken::TextMuted).grow(1)),
                    text(move || format!("{}/160", activity_points.get()))
                        .style(style().font_size(13).weight(600)),
                )),
                kivact::ui::progress(move || Some(f64::from(activity_points.get()) / 160.0))
                    .name("活動ポイント")
                    .style(style().height(6)),
                text("次の回復まで 04:12")
                    .style(style().font_size(11.5).color(ColorToken::TextMuted)),
            ))
            .style(style().padding(12).gap(8).radius(10)),
            row((
                ui::avatar("SN").size(32).halo(true),
                column((
                    text("先生").style(style().weight(600)),
                    text("Lv.72 · 管理者")
                        .style(style().font_size(12).color(ColorToken::TextMuted)),
                ))
                .style(style().grow(1)),
                kivact::view::icon("logout"),
            ))
            .style(style().padding(8).gap(10).align_items(Align::Center)),
        ))
        .style(style().gap(8).padding_top(8)),
    ))
    .style(
        style()
            .width(SIDEBAR_WIDTH)
            .shrink(0)
            .height(pct(100))
            .padding_top(12)
            .padding_x(8)
            .padding_bottom(8)
            .background(ui::Color::Background)
            .border_right(1, ui::Color::Line),
    );
    let notices = signal(vec![
        ui::NotificationItem {
            icon: "target".into(),
            hue: ui::Hue::Blue,
            unread: true,
            ..ui::NotificationItem::new(
                1,
                "総力戦シーズン開始",
                "新しい作戦目標が追加されました。",
                "5分前",
            )
        },
        ui::NotificationItem {
            icon: "message-circle".into(),
            unread: true,
            ..ui::NotificationItem::new(
                2,
                "ミナトから新着メッセージ",
                "「報告書、置いておきました！」",
                "12分前",
            )
        },
        ui::NotificationItem {
            icon: "gift".into(),
            ..ui::NotificationItem::new(
                3,
                "ログインボーナス",
                "クレジット ×20,000 を受け取りました。",
                "今日 04:00",
            )
        },
    ]);
    let topbar = row((
        ui::breadcrumb(
            ["Pop UI".to_string()]
                .into_iter()
                .map(|item| item.to_string())
                .collect::<Vec<_>>(),
        ),
        kivact::view::icon("chevron-right").style(style().color(ColorToken::TextMuted)),
        text(move || SECTIONS[selected.get()].1.to_string())
            .test_id("pop-current-section")
            .style(style().grow(1).weight(500)),
        kivact::ui::button("コンポーネントを検索…　 Ctrl K")
            .variant(kivact::ui::ButtonVariant::Secondary)
            .size(kivact::ui::ControlSize::Small)
            .icon(kivact::view::icon("search"))
            .style(
                style()
                    .width(300)
                    .color(ColorToken::TextMuted)
                    .font_size(13),
            )
            .test_id("pop-search-trigger")
            .on_press(move || command.set(true)),
        kivact::ui::button("テーマ切替")
            .variant(kivact::ui::ButtonVariant::Ghost)
            .hide_label(true)
            .icon(kivact::view::icon("moon"))
            .test_id("pop-theme-toggle")
            .on_press(move || dark.update(|v| *v = !*v)),
        ui::notification_menu(
            kivact::ui::button("通知")
                .variant(kivact::ui::ButtonVariant::Ghost)
                .hide_label(true)
                .icon(kivact::view::icon("bell"))
                .test_id("pop-notifications"),
            notices,
        )
        .on_mark_read(move || {
            notices.update(|items| items.iter_mut().for_each(|item| item.unread = false))
        })
        .style(style().width(36)),
        ui::avatar("SN").size(32),
    ))
    .style(
        style()
            .height(TOPBAR_HEIGHT)
            .shrink(0)
            .padding_x(24)
            .gap(12)
            .align_items(Align::Center)
            .background(ui::Color::Background)
            .border_bottom(1, ui::Color::Line),
    );
    let content = column((
        section_views,
        row((
            text("Pop UI catalog").style(style().grow(1)),
            text("Icons: Tabler Icons (MIT)"),
        ))
        .style(
            style()
                .padding_top(32)
                .font_size(12)
                .color(ColorToken::TextMuted),
        ),
    ))
    .with_node(|node| node.node_ref(content_ref))
    .style(
        style()
            .width(pct(100))
            .max_width(1240)
            .padding_x(24)
            .padding_top(24)
            .padding_bottom(48),
    );
    let body = row((
        sidebar,
        column((
            topbar,
            scroll(column(content).style(style().align_items(Align::Center)))
                .with_node(|node| node.node_ref(scroll_ref))
                .test_id("pop-content-scroll")
                .style(style().grow(1).min_height(0))
                .draw(|draw, size| {
                    for x in (0..size.width as usize).step_by(32) {
                        draw.line(
                            Point::new(x as f64, 0.0),
                            Point::new(x as f64, size.height),
                            1.0,
                            rgba(0x2b8bd8, 0.025),
                        );
                    }
                    for y in (0..size.height as usize).step_by(32) {
                        draw.line(
                            Point::new(0.0, y as f64),
                            Point::new(size.width, y as f64),
                            1.0,
                            rgba(0x2b8bd8, 0.025),
                        );
                    }
                }),
        ))
        .style(style().grow(1).min_width(0).height(pct(100))),
    ))
    .style(style().background(ui::Color::Background))
    .style(style().size(pct(100), pct(100)));
    let mut commands = SECTIONS
        .iter()
        .enumerate()
        .map(|(i, (id, title, _))| {
            Command::new(PaletteCommand::Section(i), *title)
                .with_group("移動")
                .with_hint(format!("#{id}"))
                .with_icon(SECTIONS[i].2)
        })
        .collect::<Vec<_>>();
    commands.extend([
        Command::new(PaletteCommand::Theme, "テーマを切り替え")
            .with_group("アクション")
            .with_hint("ライト / ダーク")
            .with_icon("moon"),
        Command::new(PaletteCommand::Create, "新しい依頼を作成")
            .with_group("アクション")
            .with_hint("ダイアログ")
            .with_icon("plus"),
        Command::new(PaletteCommand::Start, "作戦を開始")
            .with_group("アクション")
            .with_hint("確認")
            .with_icon("sword"),
        Command::new(PaletteCommand::Profile, "生徒プロフィールを表示")
            .with_group("アクション")
            .with_hint("ドロワー")
            .with_icon("user"),
        Command::new(PaletteCommand::Reward, "報酬を受け取る")
            .with_group("アクション")
            .with_hint("トースト")
            .with_icon("gift"),
    ]);
    let run_command = callback(move |command| match command {
        PaletteCommand::Section(i) => {
            if let Some(r) = refs[i].rect().get() {
                let offset = scroll_ref.scroll_offset().get();
                scroll_ref.scroll_to(
                    Point::new(0.0, (offset.y + r.y - 76.0).max(0.0)),
                    ScrollBehavior::Smooth,
                );
            }
            selected.set(i);
        }
        PaletteCommand::Theme => dark.update(|value| *value = !*value),
        PaletteCommand::Create => form_dialog.set(true),
        PaletteCommand::Start => confirm.set(true),
        PaletteCommand::Profile => sheet.set(true),
        PaletteCommand::Reward => notify.call((
            ui::Hue::Yellow,
            "報酬を獲得！".into(),
            "クレジット ×12,000 / 活動報告書 ×2".into(),
        )),
    });
    kivact::style::provide_theme(theme, move || {
        column((
            body,
            kivact::ui::toast_region(toaster()),
            kivact::ui::dialog("作戦を開始しますか？", confirm)
                .content(move || {
                    column((
                        text("活動ポイントを 20 消費して「外郭地区のパトロール」を開始します。"),
                        row((
                            ui::avatar("MN").halo(true),
                            ui::avatar("HK").hue(330.0).halo(true),
                            ui::avatar("YZ").hue(45.0).halo(true),
                            ui::avatar("AR").hue(160.0).halo(true),
                        ))
                        .style(
                            style()
                                .gap(14)
                                .padding(20)
                                .justify_content(Justify::Center)
                                .background(ColorToken::SurfaceAlt)
                                .radius(10),
                        ),
                        kivact::ui::checkbox("今後この確認を表示しない", remember_confirmation),
                    ))
                    .style(style().gap(16))
                })
                .footer(move || {
                    row((
                        kivact::ui::button("キャンセル")
                            .variant(kivact_ui_pop::Slant)
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || confirm.set(false)),
                        kivact::ui::button("出撃する")
                            .variant(kivact_ui_pop::Slant)
                            .variant(kivact_ui_pop::ButtonTone::Accent)
                            .test_id("pop-confirm-start")
                            .on_press(move || {
                                confirm.set(false);
                                notify.call((
                                    ui::Hue::Blue,
                                    "作戦開始".into(),
                                    "部隊が外郭地区へ出撃しました。".into(),
                                ));
                            }),
                    ))
                    .style(style().gap(8).justify_content(Justify::End))
                }),
            kivact::ui::dialog("新しい依頼を作成", form_dialog)
                .content(move || {
                    column((
                        kivact::ui::field(
                            "依頼タイトル *",
                            kivact::ui::text_input("依頼タイトル", request_title)
                                .on_submit(move || {
                                    if !request_title.get_untracked().trim().is_empty() {
                                        form_dialog.set(false);
                                        notify.call((
                                            ui::Hue::Mint,
                                            "依頼を作成しました".into(),
                                            "生徒たちに通知が送られます。".into(),
                                        ));
                                    }
                                })
                                .placeholder("例：図書館の蔵書整理")
                                .test_id("pop-request-title"),
                        ),
                        row((
                            kivact::ui::field(
                                "優先度",
                                kivact::ui::select(
                                    "優先度",
                                    vec![
                                        ListItem::new("normal", "通常"),
                                        ListItem::new("high", "高"),
                                        ListItem::new("urgent", "緊急"),
                                    ],
                                    request_priority,
                                ),
                            )
                            .style(style().grow(1)),
                            kivact::ui::field(
                                "期限",
                                ui::date_input(
                                    "依頼期限",
                                    request_date,
                                    CalendarDate::new(2026, 10, 3),
                                )
                                .style(style().width(180)),
                            ),
                        ))
                        .style(style().gap(16)),
                        kivact::ui::field(
                            "詳細",
                            kivact::ui::text_area("依頼詳細", request_detail)
                                .placeholder("依頼の内容を入力…"),
                        ),
                    ))
                    .style(style().gap(16))
                })
                .footer(move || {
                    row((
                        kivact::ui::button("キャンセル")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .on_press(move || form_dialog.set(false)),
                        kivact::ui::button("作成する")
                            .disabled(move || request_title.get().trim().is_empty())
                            .test_id("pop-create-request")
                            .on_press(move || {
                                form_dialog.set(false);
                                notify.call((
                                    ui::Hue::Mint,
                                    "依頼を作成しました".into(),
                                    "生徒たちに通知が送られます。".into(),
                                ));
                            }),
                    ))
                    .style(style().gap(8).justify_content(Justify::End))
                }),
            kivact::ui::dialog(drawer_title, sheet)
                .mode(kivact::interaction::DialogMode::Drawer(
                    kivact::geometry::Edge::Right,
                ))
                .content(move || {
                    column((
                        row((
                            ui::avatar("MN").size(72).halo(true),
                            column((
                                text("アズール学園 · 調査部").style(style().weight(600)),
                                text("★★★").style(style().color(rgb(0xf2bf0a))),
                                wrap((
                                    ui::badge("ストライカー"),
                                    ui::badge("後衛").hue(ui::Hue::Gray),
                                )),
                            ))
                            .style(style().gap(6)),
                        ))
                        .style(
                            style()
                                .padding(16)
                                .padding_top(24)
                                .gap(18)
                                .align_items(Align::Center)
                                .radius(10)
                                .border(1, ColorToken::Border)
                                .background(ColorToken::SurfaceAlt),
                        ),
                        ui::key_value("レベル", text("Lv.86")),
                        ui::key_value("絆ランク", text("Rank 20")),
                        ui::key_value("攻撃タイプ", ui::badge("爆発").hue(ui::Hue::Red)),
                        ui::key_value("防御タイプ", ui::badge("重装甲").hue(ui::Hue::Yellow)),
                        text("個別設定").style(style().font_size(15).weight(600).margin_top(8)),
                        row((
                            text("メイン編成に固定").style(style().grow(1)),
                            kivact::ui::switch("メイン編成に固定", pin_student)
                                .hide_label(true)
                                .style(style().width(32).padding_left(0)),
                        ))
                        .style(style().align_items(Align::Center)),
                        row((
                            text("スキル自動使用").style(style().grow(1)),
                            kivact::ui::switch("スキル自動使用", auto_skill)
                                .hide_label(true)
                                .style(style().width(32).padding_left(0)),
                        ))
                        .style(style().align_items(Align::Center)),
                    ))
                    .style(style().gap(16))
                })
                .footer(move || {
                    row((
                        kivact::ui::button("閉じる")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .on_press(move || sheet.set(false)),
                        kivact::ui::button("保存")
                            .test_id("pop-drawer-save")
                            .on_press(move || {
                                sheet.set(false);
                                notify.call((
                                    ui::Hue::Mint,
                                    "保存しました".into(),
                                    "個別設定を保存しました。".into(),
                                ));
                            }),
                    ))
                    .style(style().gap(8))
                }),
            command_palette(commands, command)
                .on_activate(move |value| run_command.try_call(value))
                .close_on(is_palette_shortcut)
                .footer({
                    kivact::view::text(kivact::i18n::message(
                        "palette.hint",
                        "↑↓ 移動   Enter 実行   Esc 閉じる",
                    ))
                }),
        ))
        .style(
            style()
                .size(pct(100), pct(100))
                .font_family("Inter, 'Noto Sans JP', sans-serif"),
        )
        .on_key(move |key: KeyEvent| {
            if is_palette_shortcut(&key) {
                command.set(true);
                Handled::Yes
            } else {
                Handled::No
            }
        })
    })
}

/// Ctrl+K: opens the command palette, and closes it from inside (a modal
/// keeps the keys to itself, so the palette is given the same matcher).
fn is_palette_shortcut(key: &KeyEvent) -> bool {
    key.modifiers().ctrl
        && matches!(key.key(), Key::Char(character) if character.eq_ignore_ascii_case(&'k'))
}

fn overview(action: impl Into<kivact::state::Callback<()>>, command: Signal<bool>) -> Node {
    let stats = [
        ("在籍生徒", "124", "+6 今週"),
        ("進行中の依頼", "18", "3件 期限間近"),
        ("クレジット", "8.2M", "+12.4%"),
    ]
    .into_iter()
    .map(|(l, v, d)| {
        column((
            text(l).style(style().font_size(12).color(rgba(0xffffff, 0.85))),
            text(v).style(
                style()
                    .font_family("Barlow")
                    .italic(true)
                    .weight(800)
                    .font_size(28)
                    .color(Color::WHITE)
                    .line_height(1.1),
            ),
            text(d).style(style().font_size(12).color(rgb(0xffdc3c))),
        ))
        .style(
            style()
                .padding_x(14)
                .padding_y(12)
                .gap(2)
                .radius(10)
                .grow(1)
                .background(rgba(0xffffff, 0.14))
                .border(1, rgba(0xffffff, 0.24)),
        )
    })
    .collect::<Vec<_>>();
    let heading = kivact::view::rich_text(vec![
        kivact::view::span("おかえりなさい、"),
        kivact::view::span("先生。").background(
            linear_gradient(180)
                .stop(Color::TRANSPARENT, pct(62))
                .stop(rgba(0xffdc3c, 0.6), pct(62)),
        ),
    ])
    .role(Role::Heading)
    .style(
        style()
            .color(Color::WHITE)
            .font_size(28)
            .weight(700)
            .line_height(1.3),
    );
    column((hero(heading, "今日も生徒たちが待っています。やわらかく、明るく、迷わない——\nブルーアーカイブ風のネイティブSaaS UIコンポーネント集です。", (kivact::ui::button("作戦を開始").size(kivact::ui::ControlSize::Large).variant(kivact_ui_pop::ButtonTone::Accent).variant(kivact_ui_pop::Slant).icon(kivact::view::icon("player-play")).on_press(action), kivact::ui::button("コマンドパレット").size(kivact::ui::ControlSize::Large).variant(kivact_ui_pop::ButtonTone::Glass).icon(kivact::view::icon("command")).on_press(move || command.set(true))), stats), columns((ui::stat_card("アクティブ生徒", "1,284").trend("↗ 8.2%").icon("users").points(vec![2., 4., 3., 5., 4., 7., 6., 9., 8., 11.]).tone(ui::Hue::Mint), ui::stat_card("依頼完了率", "96.4%").trend("↗ 3.1%").icon("circle-check").points(vec![3., 4., 3., 6., 5., 7., 6., 9., 8., 11.]).tone(ui::Hue::Mint), ui::stat_card("週間支出", "¥2.4M").trend("↘ 1.4%").icon("coin").points(vec![10., 8., 9., 5., 7., 4., 5., 3., 4., 2.]).tone(ui::Hue::Red), ui::stat_card("平均絆ランク", "18.6").trend("↗ 12%").icon("heart").points(vec![2., 3., 5., 4., 7., 9., 8., 12., 14., 17.]).tone(ui::Hue::Mint)), 4))).style(style().gap(16))
}
