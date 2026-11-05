//! Role: system specimens for the operation-console catalog.

use super::common::{cards, line, list, specimen, suppress_search_shortcut, wide};
use super::state::Demo;
use kivact::interaction::{MenuEntry, Toast, Tone, toaster};
use kivact::platform;
use kivact::prelude::{
    ColorToken, IntoView, Node, NodeBuilder, Role, column, node, rgb, row, signal, spacer, text,
};
use kivact::state::{binding, memo};
use kivact::style::{Align, Justify, pct, style};
use kivact::task::spawn;
use kivact::view::{rich_text, span};
use kivact_ui_game as g;
use std::time::Duration;

pub(super) fn feedback(scene: Demo) -> Node {
    let info = signal(true);
    let success = signal(true);
    let warning = signal(true);
    let danger = signal(true);
    cards(
        (
            wide(specimen(
                "FBK-01",
                "システムバナー",
                kivact::ui::alert("NOTICE").variant(kivact::ui::AlertVariant::Banner)
                .tone(kivact::interaction::Tone::Warning).actions(row((
                        g::mono("T-41:20:00").style(style().color(rgb(0x9fb7d2))),
                        kivact::ui::button("詳細")
                            .variant(kivact_ui_game::Slant)
                            .test_id("game-banner-details")
                            .size(kivact::ui::ControlSize::Small)
                            .style(style().background(rgb(0xffc928)).color(rgb(0x1a1a1a)))
                            .on_press(move || {
                                toaster().push(
                                    Toast::new("計画メンテナンス").body("10/05 02:00–04:00 JST · API は読み取り専用になります。").tone(Tone::Warning)
                                    .duration(Some(Duration::from_secs(6))),
                                );
                            }),
                    ))
                    .style(style().gap(14).align_items(Align::Center)))

                .with_node(|node| node.role(Role::Status).live_region(kivact::view::LiveRegion::Polite))
.content(move || rich_text(vec![
                    span("計画メンテナンス:").weight(700).color(rgb(0xffc928)),
                    span(" 10/05 (日) 02:00–04:00 JST に API が読み取り専用になります。"),
                ])),
            )),
            specimen(
                "FBK-02",
                "インラインアラート",
                list((
                    kivact::view::show(info, move || {
                        kivact::ui::alert("新しいバージョンがあります").description("v2.5.0 で監査ログのエクスポートが追加されました。").tone(kivact::interaction::Tone::Info).icon(kivact::view::icon("info-circle"))
                        .visible(info).dismissible(true).dismiss_label(kivact::i18n::message("game.alert.close", "Close alert")).dismiss_content(text("×")).with_node(|node| node.role(Role::Status).live_region(kivact::view::LiveRegion::Polite))
                    }),
                    kivact::view::show(success, move || {
                        kivact::ui::alert("バックアップ完了").description("2.4GB · 所要 3 分 12 秒").tone(kivact::interaction::Tone::Success).icon(kivact::view::icon("circle-check"))
                        .visible(success).dismissible(true).dismiss_label(kivact::i18n::message("game.alert.close", "Close alert")).dismiss_content(text("×")).with_node(|node| node.role(Role::Status).live_region(kivact::view::LiveRegion::Polite))
                    }),
                    kivact::view::show(warning, move || {
                        kivact::ui::alert("証明書の期限が近づいています").description("*.ops.example.internal — 残り 9 日").tone(kivact::interaction::Tone::Warning).icon(kivact::view::icon("alert-triangle"))
                        .actions( {
                            row((
                                kivact::ui::button("更新する")
                                    .variant(kivact_ui_game::Slant)
                                    .size(kivact::ui::ControlSize::Small)
                                    .variant(kivact::ui::ButtonVariant::Ghost)
                                    .on_press(move || {
                                        toaster().push(
                                            Toast::new("更新を開始しました").body("証明書の更新を開始しました。").tone(Tone::Info)
                                            .duration(Some(Duration::from_secs(4))),
                                        );
                                    }),
                                kivact::ui::button("後で")
                                    .variant(kivact_ui_game::Slant)
                                    .size(kivact::ui::ControlSize::Small)
                                    .variant(kivact::ui::ButtonVariant::Ghost),
                            ))
                            .style(style().gap(10).margin_top(6))
                        })
                        .visible(warning).dismissible(true).dismiss_label(kivact::i18n::message("game.alert.close", "Close alert")).dismiss_content(text("×"))
                    }),
                    kivact::view::show(danger, move || {
                        kivact::ui::alert("決済連携に失敗しました").description("ERR_UPSTREAM_502 · 3 回再試行後に停止").tone(kivact::interaction::Tone::Danger).icon(kivact::view::icon("alert-octagon"))
                        .visible(danger).dismissible(true).dismiss_label(kivact::i18n::message("game.alert.close", "Close alert")).dismiss_content(text("×"))
                    }),
                )),
            ),
            specimen(
                "FBK-03",
                "トースト",
                list((
                    line(
                        [
                            Tone::Info,
                            Tone::Success,
                            Tone::Warning,
                            Tone::Danger,
                        ]
                        .into_iter()
                        .map(|tone| {
                            let label = match tone {
                                Tone::Success => "Success",
                                Tone::Warning => "Warning",
                                Tone::Danger => "Error",
                                Tone::Info | Tone::Neutral | Tone::Accent => "Info",
                            };
                            kivact::ui::button(label)
                                .variant(kivact_ui_game::Slant)
                                .variant(kivact::ui::ButtonVariant::Secondary)
                                .size(kivact::ui::ControlSize::Small)
                                .on_press(move || {
                                    toaster().push(
                                        Toast::new(label).body("ホバーで自動消去タイマーを一時停止します。").tone(tone)
                                        .duration(Some(Duration::from_millis(4200))),
                                    );
                                })
                        })
                        .collect::<Vec<_>>(),
                    ),
                    text(
                        "ホバーで自動消去タイマーを一時停止。aria-live=polite の領域に追加される。",
                    )
                    .style(style().font_size(11.5).color(ColorToken::TextMuted)),
                )),
            ),
            specimen(
                "FBK-04",
                "スケルトン",
                list((
                    kivact::ui::skeleton().variant(g::SkeletonVariant::Rows).lines(2).with_node(|node| node.role(Role::Status)).name(kivact::i18n::message("game.skeleton.label", "Loading")),
                    node().style(
                        style()
                            .height(64)
                            .width(pct(100))
                            .background(ColorToken::SurfaceAlt),
                    ),
                )),
            ),
            specimen(
                "FBK-05",
                "空状態",
                kivact::ui::empty_state("NO RECORDS").icon(()).description("条件に一致する案件はありません。フィルタを解除するか、新しい案件を登録してください。").actions( line((
                        kivact::ui::button("フィルタ解除")
                            .variant(kivact_ui_game::Slant)
                            .size(kivact::ui::ControlSize::Small)
                            .variant(kivact::ui::ButtonVariant::Ghost),
                        kivact::ui::button("新規登録")
                            .variant(kivact_ui_game::Slant)
                            .icon(kivact::view::icon("plus"))
                            .size(kivact::ui::ControlSize::Small)
                            .on_press(move || scene.create.set(true)),
                    ))),
            ),
        ),
        460.,
    )
}
pub(super) fn overlays(scene: Demo) -> Node {
    let pinned = signal(true);
    let archived = signal(false);
    let user_open = signal(false);
    cards(
        (
            specimen(
                "OVL-01",
                "ドロップダウンメニュー",
                line((
                    kivact::ui::menu("アクション ▾", move || {
                        vec![
                            MenuEntry::item("rename", "名前を変更").with_shortcut("F2"),
                            MenuEntry::item("copy", "複製").with_shortcut("Ctrl+D"),
                            MenuEntry::item("window", "新しいウィンドウで開く"),
                            MenuEntry::separator(),
                            MenuEntry::checkbox("pinned", "詳細列を表示", pinned.get()),
                            MenuEntry::checkbox("archived", "アーカイブを含める", archived.get()),
                            MenuEntry::separator(),
                            MenuEntry::item("delete", "削除…").with_shortcut("Del"),
                        ]
                    })
                    .on_activate(move |key: &'static str| match key {
                        "rename" => scene.create.set(true),
                        "copy" => {
                            spawn(platform::copy_text("OPS-2041"));
                        }
                        "window" => scene.drawer.set(true),
                        "pinned" => pinned.update(|on| *on = !*on),
                        "archived" => archived.update(|on| *on = !*on),
                        "delete" => scene.delete.set(true),
                        _ => {}
                    })
                    .offset(0.)
                    .attach(
                        kivact::ui::button("アクション ▾")
                            .variant(kivact_ui_game::Slant)
                            .variant(kivact::ui::ButtonVariant::Secondary),
                    ),
                    g::icon_menu(
                        "その他",
                        vec![
                            MenuEntry::item("favorite", "お気に入り"),
                            MenuEntry::item("export", "エクスポート"),
                        ],
                    )
                    .icon("dots")
                    .on_select(move |key: &'static str| match key {
                        "favorite" => scene.drawer.set(true),
                        _ => scene.palette.set(true),
                    }),
                )),
            ),
            specimen(
                "OVL-02",
                "モーダルダイアログ",
                line((
                    kivact::ui::button("フォームダイアログ")
                        .variant(kivact_ui_game::Slant)
                        .icon(kivact::view::icon("plus"))
                        .test_id("game-create-open")
                        .on_press(move || scene.create.set(true)),
                    kivact::ui::button("確認ダイアログ")
                        .variant(kivact_ui_game::Slant)
                        .icon(kivact::view::icon("alert-triangle"))
                        .variant(kivact::ui::ButtonVariant::Danger)
                        .on_press(move || scene.delete.set(true)),
                )),
            ),
            specimen(
                "OVL-03",
                "ドロワー / パレット",
                line((
                    kivact::ui::button("詳細ドロワー")
                        .variant(kivact_ui_game::Slant)
                        .test_id("game-drawer-open")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .on_press(move || scene.drawer.set(true)),
                    kivact::ui::button("コマンドパレット")
                        .variant(kivact_ui_game::Slant)
                        .icon(kivact::view::icon("command"))
                        .variant(kivact_ui_game::ButtonTone::Dark)
                        .on_press(move || scene.palette.set(true)),
                )),
            ),
            specimen(
                "OVL-04",
                "ツールチップ",
                line((
                    kivact::ui::tooltip("上書き保存  Ctrl+S")
                        .placement(kivact::view::Placement::Below)
                        .align(kivact::view::OverlayAlign::Center)
                        .offset(6.)
                        .attach(
                            g::tool_button("保存")
                                .test_id("game-tooltip-save")
                                .icon(kivact::view::icon("device-floppy"))
                                .hide_label(true),
                        ),
                    kivact::ui::tooltip("再読み込み")
                        .placement(kivact::view::Placement::Below)
                        .align(kivact::view::OverlayAlign::Center)
                        .offset(6.)
                        .attach(
                            g::tool_button("再読み込み")
                                .icon(kivact::view::icon("refresh"))
                                .hide_label(true),
                        ),
                    kivact::ui::tooltip("権限: 管理者のみ")
                        .placement(kivact::view::Placement::Below)
                        .align(kivact::view::OverlayAlign::Center)
                        .offset(6.)
                        .attach(
                            kivact::ui::button("ロック中")
                                .variant(kivact_ui_game::Slant)
                                .size(kivact::ui::ControlSize::Small)
                                .icon(kivact::view::icon("lock"))
                                .variant(kivact::ui::ButtonVariant::Secondary),
                        ),
                )),
            ),
            specimen(
                "OVL-05",
                "ユーザーポップオーバー",
                (kivact::ui::dialog("ユーザー詳細", user_open)
                    .mode(kivact::interaction::DialogMode::Popover)
                    .content(move || {
                        list((
                            row((
                                kivact::ui::avatar("MS")
                                    .fallback_content(|| text("MS"))
                                    .variant(g::AvatarVariant {
                                        side: 44.,
                                        color: rgb(0x0a62c2),
                                    })
                                    .status(None),
                                column((
                                    text("真柴 ソウ").style(style().weight(700)),
                                    g::mono("ADMIN · 整備部")
                                        .style(style().color(ColorToken::TextMuted)),
                                )),
                            ))
                            .style(style().gap(12)),
                            row((
                                text("ステータス").style(style().color(ColorToken::TextMuted)),
                                spacer(),
                                kivact::ui::badge("Online")
                                    .tone(kivact::interaction::Tone::Success),
                            )),
                            row((
                                text("担当中").style(style().color(ColorToken::TextMuted)),
                                spacer(),
                                g::mono("12 件"),
                            )),
                            kivact::ui::button("メッセージを送る")
                                .variant(kivact_ui_game::Slant)
                                .size(kivact::ui::ControlSize::Small)
                                .on_press(move || scene.drawer.set(true)),
                        ))
                    })
                    .attach(
                        node()
                            .role(Role::Button)
                            .name("ユーザー詳細")
                            .focusable(true)
                            .child(
                                row((
                                    kivact::ui::avatar("MS")
                                        .fallback_content(|| text("MS"))
                                        .variant(g::AvatarVariant {
                                            side: 32.,
                                            color: rgb(0x0a62c2),
                                        })
                                        .status(Some(kivact::interaction::Tone::Success)),
                                    column((
                                        text("真柴 ソウ").style(style().weight(700)),
                                        text("クリックで詳細").style(
                                            style().font_size(11).color(ColorToken::TextMuted),
                                        ),
                                    )),
                                ))
                                .style(style().gap(10).align_items(Align::Center)),
                            ),
                    ),),
            ),
        ),
        330.,
    )
}
pub(super) fn create_dialog(scene: Demo) -> impl IntoView {
    let title = signal(String::new());
    // A dialog opens empty each time, however it was closed.
    kivact::state::effect(move || {
        if !scene.create.get() {
            title.set(String::new());
        }
    });
    kivact::ui::dialog("新規案件", scene.create)
        .content(move || {
            list((
                kivact::ui::field(
                    "件名 *",
                    kivact::ui::text_input("新規案件名", title)
                        .placeholder("例: 第 4 区画 夜間巡回"),
                ),
                row((
                    kivact::ui::field(
                        "担当",
                        kivact::ui::select(
                            "Select",
                            super::controls::choice_options(&[
                                ("maintenance", "整備部"),
                                ("information", "情報管理部"),
                                ("security", "警備局"),
                            ]),
                            signal(Some("maintenance".to_owned())),
                        )
                        .placeholder("")
                        .name("新規案件の担当部署"),
                    )
                    .style(style().grow(1).basis(0)),
                    kivact::ui::field(
                        "期限",
                        kivact::ui::text_input("新規案件の期限", signal("2026-10-10".to_owned()))
                            .style(style().font_family("JetBrains Mono")),
                    )
                    .style(style().grow(1).basis(0)),
                ))
                .style(style().gap(12)),
                kivact::ui::checkbox("作成後に担当者へ通知する", signal(true)),
            ))
        })
        .footer(move || {
            (
                kivact::ui::button("キャンセル")
                    .variant(kivact_ui_game::Slant)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || scene.create.set(false)),
                kivact::ui::button("作成")
                    .variant(kivact_ui_game::Slant)
                    .disabled(move || title.get().trim().is_empty())
                    .on_press(move || {
                        scene.create.set(false);
                        toaster().push(
                            Toast::new("案件を作成しました")
                                .body(title.get())
                                .tone(Tone::Success)
                                .duration(Some(Duration::from_secs(4))),
                        );
                    }),
            )
        })
}
pub(super) fn delete_dialog(scene: Demo) -> impl IntoView {
    let confirmation = signal(String::new());
    kivact::state::effect(move || {
        if !scene.delete.get() {
            confirmation.set(String::new());
        }
    });
    // An irreversible question: an outside press does not answer it.
    kivact::ui::dialog("案件を削除", scene.delete).mode(kivact::interaction::DialogMode::AlertModal)
        .content(move || {
            list((
                row((
                    node().child(kivact::view::icon("alert-triangle").style(style().size(24, 24))).style(style().size(44, 44).align_items(Align::Center).justify_content(Justify::Center).background(rgb(0xfde4e8)).color(ColorToken::Danger)),
                    column((
                        text("選択した案件を削除します。").style(style().weight(700)),
                        text("関連するタスクと添付ファイルも削除され、元に戻せません。確認のため DELETE と入力してください。").style(style().color(ColorToken::TextMuted).font_size(13)),
                    )).style(style().grow(1).gap(4)),
                )).style(style().gap(14)),
                kivact::ui::text_input("DELETE 確認", confirmation)
                    .placeholder("DELETE"),
            ))
        }).footer(move || {
            (
                kivact::ui::button("キャンセル").variant(kivact_ui_game::Slant)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .on_press(move || scene.delete.set(false)),
                kivact::ui::button("削除する").variant(kivact_ui_game::Slant)
                    .icon(kivact::view::icon("trash"))
                    .variant(kivact::ui::ButtonVariant::Danger)
                    .disabled(move || confirmation.get() != "DELETE")
                    .on_press(move || {
                        let ids = scene.selected.get();
                        scene
                            .rows
                            .update(|rows| rows.retain(|r| !ids.contains(&r.id)));
                        scene.selected.set(Vec::new());
                        scene.delete.set(false);
                        toaster().push(Toast::new("削除しました").body(format!("{} 件の案件を削除しました。", ids.len())).tone(Tone::Danger).duration(Some(Duration::from_secs(4))));
                    }),
            )
        })
}
pub(super) fn drawer_body() -> Node {
    list((
        kivact::ui::description_list(
            (vec![
                (
                    "Status".into(),
                    kivact::ui::badge("RUNNING")
                        .tone(kivact::interaction::Tone::Info)
                        .into_view(),
                ),
                ("Owner".into(), text("整備部 / 真柴").into_view()),
                ("Due".into(), g::mono("2026-10-10").into_view()),
            ])
            .into_iter()
            .map(|(term, detail): (String, _)| kivact::ui::description(term, detail))
            .collect::<Vec<_>>(),
        ),
        row((
            text("進捗").style(style().width(46).font_size(12)),
            kivact::ui::progress(Some(0.68))
                .name("案件の進捗")
                .variant(kivact::interaction::Tone::Info)
                .variant(g::Striped)
                .moving(true)
                .style(style().grow(1)),
            g::mono("68%"),
        ))
        .style(style().gap(10).align_items(Align::Center)),
        g::activity(vec![
            g::ActivityEntry::new("09:40", "作業を開始", kivact::interaction::Tone::Info),
            g::ActivityEntry::new(
                "08:30",
                rich_text(vec![span("真柴").weight(700), span(" が承認")]),
                kivact::interaction::Tone::Success,
            ),
        ]),
    ))
}
pub(super) fn layout() -> impl kivact::view::Component {
    let document = signal("OPS-2042.md".to_owned());
    let document_title = signal("第 3 区画 定期点検".to_owned());
    let selection = binding(
        memo(move || Some(document.get())),
        move |id: Option<String>| {
            if let Some(id) = id {
                document_title.set(id.clone());
                document.set(id);
            }
        },
    );
    let nodes = vec![
        kivact::interaction::TreeNode::new("operations".to_owned(), None, "operations"),
        kivact::interaction::TreeNode::new(
            "OPS-2041.md".to_owned(),
            Some("operations".to_owned()),
            "OPS-2041.md",
        ),
        kivact::interaction::TreeNode::new(
            "OPS-2042.md".to_owned(),
            Some("operations".to_owned()),
            "OPS-2042.md",
        ),
        kivact::interaction::TreeNode::new(
            "OPS-2043.md".to_owned(),
            Some("operations".to_owned()),
            "OPS-2043.md",
        ),
        kivact::interaction::TreeNode::new("inventory".to_owned(), None, "inventory"),
        kivact::interaction::TreeNode::new(
            "stock.db".to_owned(),
            Some("inventory".to_owned()),
            "stock.db",
        ),
        kivact::interaction::TreeNode::new("archive".to_owned(), None, "archive"),
        kivact::interaction::TreeNode::new(
            "2025_summary.md".to_owned(),
            Some("archive".to_owned()),
            "2025_summary.md",
        ),
    ];
    let expanded = signal(vec!["operations".to_owned()]);
    let files = kivact::ui::tree("Workspace files", nodes)
        .expanded(expanded)
        .selection(binding(selection, move |key: Option<String>| {
            if key
                .as_deref()
                .is_none_or(|id| !["operations", "inventory", "archive"].contains(&id))
            {
                selection.set(key);
            }
        }))
        .on_activate(move |key| {
            if ["operations", "inventory", "archive"].contains(&key.as_str()) {
                expanded.update(|keys| {
                    if keys.contains(&key) {
                        keys.retain(|k| k != &key);
                    } else {
                        keys.push(key);
                    }
                });
            }
        })
        .item(|item, _| {
            let key = item.get_untracked().key;
            let folder = ["operations", "inventory", "archive"].contains(&key.as_str());
            let marker = match key.as_str() {
                "OPS-2041.md" => "M",
                "OPS-2043.md" => "A",
                _ => "",
            };
            row((
                kivact::view::icon(if folder { "folder" } else { "file-text" }).style(
                    style().size(14, 14).color(if folder {
                        ColorToken::Warning
                    } else {
                        ColorToken::TextMuted
                    }),
                ),
                text(key).style(style().grow(1).font_size(12.5)),
                g::mono(marker),
            ))
            .style(style().grow(1).gap(6).align_items(Align::Center))
        });
    let title = signal("第 3 区画 定期点検".to_owned());
    let priority = signal(Some("medium".to_owned()));
    let notify = signal(true);
    let opacity = signal(90_f64);
    let document_body = list((
        g::mono(move || document.get().trim_end_matches(".md").to_uppercase()).style(style().color(ColorToken::Accent)),
        text(document_title).test_id("game-document-title").style(style().font_size(18).weight(900)),
        line((kivact::ui::badge("RUNNING").tone(kivact::interaction::Tone::Info), kivact::ui::badge("整備部").tone(kivact::interaction::Tone::Neutral) .variant(g::BadgeVariant::Plain), g::mono("updated 09:38").style(style().color(ColorToken::TextMuted)))),
        text("点検対象は配電盤 12 基、冷却ユニット 4 基。前回点検で指摘された 2 件の是正状況を確認し、結果を監査ログへ記録する。作業時間帯は 10:00–15:00 を予定。")
            .style(style().font_size(12.5).color(ColorToken::TextMuted).line_height(1.75)),
        node().style(style().padding_x(12).padding_y(10).background(rgb(0x0c1f36)).border_left(3, rgb(0x2ec4f0)))
            .child(g::mono("# checklist.yaml\ntargets:\n  - id: \"PB-01..PB-12\"\n    mode: inspect\n  - id: \"CU-01..CU-04\"\n    mode: thermal").style(style().font_size(11.5).line_height(1.7).color(rgb(0xcfe0f2)))),
    )).style(style().padding_y(18).padding_x(22));
    let inspector = column((
        kivact::ui::disclosure("基本", signal(true)).content(move || {
            list((
                inspector_row(
                    "件名",
                    kivact::ui::text_input("Inspector 件名", title)
                        .style(style().height(26).min_height(26).font_size(11.5)),
                ),
                inspector_row(
                    "優先度",
                    kivact::ui::select(
                        "Select",
                        super::controls::choice_options(&[
                            ("high", "P1 高"),
                            ("medium", "P2 中"),
                            ("low", "P3 低"),
                        ]),
                        priority,
                    )
                    .placeholder("")
                    .name("Inspector 優先度")
                    .style(style().height(26)),
                ),
                inspector_row(
                    "通知",
                    kivact::ui::switch("", notify).name("Inspector 通知"),
                ),
            ))
            .style(style().padding(10).gap(8))
        }),
        kivact::ui::disclosure("表示", signal(true)).content(move || {
            list((
                inspector_row(
                    "アクセント",
                    row((
                        node().style(style().size(16, 16).background(ColorToken::Accent)),
                        g::mono("--c-primary"),
                    ))
                    .style(style().gap(8).align_items(Align::Center)),
                ),
                inspector_row(
                    "不透明度",
                    kivact::ui::slider("Inspector 不透明度", opacity),
                ),
            ))
            .style(style().padding(10).gap(8))
        }),
        kivact::ui::disclosure("詳細", signal(false)).content(|| {
            kivact::ui::description_list(
                (vec![("TYPE".into(), "Markdown"), ("STATUS".into(), "Saved")])
                    .into_iter()
                    .map(|(term, detail): (String, _)| kivact::ui::description(term, detail))
                    .collect::<Vec<_>>(),
            )
        }),
    ));
    g::hud(column((
        row((
            g::mono("LAY-01").style(style().color(ColorToken::Accent)),
            text("デスクトップアプリ ウィンドウ").style(style().weight(700)),
            spacer(),
            g::mono("3-pane").style(style().color(ColorToken::TextMuted)),
        ))
        .style(
            style()
                .height(36)
                .padding_x(14)
                .gap(10)
                .align_items(Align::Center)
                .background(ColorToken::SurfaceAlt)
                .border_bottom(1, ColorToken::Border),
        ),
        super::window::frame(document, document_title, files, document_body, inspector),
    )))
    .test_id("LAY-01")
}
fn inspector_row(label: &str, control: impl IntoView) -> Node {
    row((
        text(label.to_owned()).style(
            style()
                .width(84)
                .font_size(11.5)
                .color(ColorToken::TextMuted),
        ),
        node().child(control).style(style().grow(1).min_width(0)),
    ))
    .on_key(suppress_search_shortcut)
    .style(style().align_items(Align::Center).gap(6).width(pct(100)))
}
