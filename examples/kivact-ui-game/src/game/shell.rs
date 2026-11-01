//! Role: The catalog shell: theme, palette, section sidebar, hero and app frame.
use super::common::{line, meta};
use super::controls::{buttons, inputs, navigation, selection};
use super::data::{display, records, table};
use super::specimens::foundations;
use super::state::Demo;
use super::system::{create_dialog, delete_dialog, drawer_body, feedback, layout, overlays};
use kivact::control::{Node, column, node, row, scroll};
use kivact::draw::Transform;
use kivact::draw::{Point, Rect};
use kivact::platform;
use kivact::prelude::NodeBuilder;
use kivact::prelude::{Key, Role, Signal, Theme, Value, effect, rgb, signal, spacer, text};
use kivact::primitive::NodeExt;
use kivact::style::ThemeMode;
use kivact::style::{Align, ColorToken, pct, style};
use kivact::style::{Position, rgba};
use kivact::ui::{Command, MenuEntry, Toast, Tone, toaster};
use kivact::view::{Handled, callback, node_ref};
use kivact::view::{IntoView, NodeRef};
use kivact_ui_game as g;
use kivact_ui_game::ThemeOptions as _;
use kivact_ui_game::{Color, Tokens};
use std::rc::Rc;
use std::time::Duration;

/// 252 px sidebar, 56 px topbar and centered 1320 px independently scrolling
/// content. The content scroll container is `scroll_node`, so the catalog can
/// scroll it for in-page navigation.
#[track_caller]
pub(super) fn app_shell(
    sidebar: impl IntoView,
    topbar: impl IntoView,
    content: impl IntoView,
    scroll_node: NodeRef,
) -> Node {
    let side = node()
        .style(
            style()
                .width(252)
                .shrink(0)
                .height(pct(100))
                .background(Color::Sidebar)
                .color(Color::SidebarText),
        )
        .child(sidebar);
    let top = row(topbar)
        .style(
            style()
                .height(56)
                .shrink(0)
                .padding_x(24)
                .align_items(Align::Center)
                .gap(12)
                .background(ColorToken::Surface)
                .border_bottom(1, ColorToken::Border),
        )
        .draw_over(|d, _| {
            d.fill_rect(Rect::new(0., 55., 180., 2.), ColorToken::Accent);
            d.fill_rect(
                Rect::new(180., 55., 60., 2.),
                d.theme().get::<Tokens>().cyan,
            );
        });
    let content = node()
        .style(
            style()
                .width(pct(100))
                .max_width(1320)
                .align_self(Align::Center)
                .padding_top(28)
                .padding_x(32)
                .padding_bottom(96),
        )
        .child(content);
    let canvas = scroll(content)
        .node_ref(scroll_node)
        .style(style().grow(1).min_height(0).align_items(Align::Center))
        .style(style().background(Color::Canvas))
        .draw(|d, s| {
            let p = d.theme().get::<Tokens>();
            let color = if p.mode == ThemeMode::Dark {
                kivact::style::rgba(0x3a9dff, 0.05)
            } else {
                kivact::style::rgba(0x1283f0, 0.055)
            };
            let mut x = 0.;
            while x < s.width {
                d.line(Point::new(x, 0.), Point::new(x, s.height), 1., color);
                x += 24.;
            }
            let mut y = 0.;
            while y < s.height {
                d.line(Point::new(0., y), Point::new(s.width, y), 1., color);
                y += 24.;
            }
        });
    row((
        side,
        column((top, canvas)).style(style().grow(1).min_width(0).height(pct(100))),
    ))
    .style(
        style()
            .width(pct(100))
            .height(pct(100))
            .align_items(Align::Stretch),
    )
}

/// What the command palette can do; the key of each of its commands.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PaletteCommand {
    Section(usize),
    Theme,
    Density,
    Create,
    Notify,
    Drawer,
}

/// The exact prototype font families, registered by the catalog executable.
fn catalog_theme(mode: kivact::style::ThemeMode, density: g::Density) -> Theme {
    let t = g::theme(mode).density(density);
    let text = kivact::style::theme::Typography {
        family: Some("Noto Sans JP".into()),
        ..t.text().clone()
    };
    let t = t.with_text(text);
    let mut tokens = (*t.get::<g::Tokens>()).clone();
    tokens.display_family = "Barlow Condensed".into();
    tokens.mono_family = "JetBrains Mono".into();
    t.with(tokens)
}

/// Mount the complete catalog through public component APIs.
pub fn catalog() -> impl IntoView {
    kivact::ui::toaster().max_visible(4);
    let mode = signal(kivact::style::ThemeMode::Light);
    let density = signal(g::Density::Comfortable);
    let theme = kivact::memo(move || catalog_theme(mode.get(), density.get()));
    let scene = Demo {
        create: signal(false),
        delete: signal(false),
        drawer: signal(false),
        palette: signal(false),
        rows: signal(records()),
        selected: signal(Vec::new()),
    };
    let sections: Rc<Vec<NodeRef>> = Rc::new((0..10).map(|_| node_ref()).collect());
    let rects: Rc<Vec<Value<Option<Rect>>>> = Rc::new(sections.iter().map(NodeRef::rect).collect());
    let scroll_node = node_ref();
    let offset = scroll_node.scroll_offset();
    let current = signal(0_usize);
    let search = node_ref();
    let for_spy = rects.clone();
    effect(move || {
        // The final chapter cannot reach the 72 px anchor when the scroll
        // container clamps at its trailing 96 px padding.
        if let Some(rect) = for_spy.last().and_then(|rect| rect.get())
            && rect.y + rect.height <= platform::window_size().height - 95.
            && rect.y + rect.height > 100.
        {
            current.set(for_spy.len() - 1);
            return;
        }
        for (i, r) in for_spy.iter().enumerate() {
            if let Some(rect) = r.get()
                && rect.y <= 200.
                && rect.y + rect.height > 100.
            {
                current.set(i);
            }
        }
    });
    let for_commands = rects.clone();
    let mut commands: Vec<Command<PaletteCommand>> = section_names()
        .iter()
        .enumerate()
        .map(|(i, (label, _, _))| {
            Command::new(PaletteCommand::Section(i), *label).hint(
                [
                    "TOKENS", "BUTTON", "INPUT", "SELECT", "NAV", "DISPLAY", "TABLE", "FEEDBACK",
                    "OVERLAY", "LAYOUT",
                ][i],
            )
        })
        .collect();
    commands.extend([
        Command::new(PaletteCommand::Theme, "テーマを切り替え").hint("THEME"),
        Command::new(PaletteCommand::Density, "表示密度を切り替え").hint("DENSITY"),
        Command::new(PaletteCommand::Create, "新規案件を作成").hint("DIALOG"),
        Command::new(PaletteCommand::Notify, "テスト通知を送る").hint("TOAST"),
        Command::new(PaletteCommand::Drawer, "詳細ドロワーを開く").hint("DRAWER"),
    ]);
    let run_command = callback(move |command| match command {
        PaletteCommand::Section(i) => reveal(scroll_node, offset, for_commands[i]),
        PaletteCommand::Theme => mode.update(|v| {
            *v = if *v == kivact::style::ThemeMode::Light {
                kivact::style::ThemeMode::Dark
            } else {
                kivact::style::ThemeMode::Light
            }
        }),
        PaletteCommand::Density => density.update(|v| {
            *v = if *v == g::Density::Comfortable {
                g::Density::Compact
            } else {
                g::Density::Comfortable
            }
        }),
        PaletteCommand::Create => scene.create.set(true),
        PaletteCommand::Notify => {
            toaster().push(
                Toast::new("通知")
                    .body("システムは正常に稼働しています。")
                    .tone(Tone::Info)
                    .duration(Some(Duration::from_secs(4))),
            );
        }
        PaletteCommand::Drawer => scene.drawer.set(true),
    });
    let content = column((
        hero(mode),
        section(0, sections[0], foundations(mode)),
        section(1, sections[1], buttons(scene)),
        section(2, sections[2], inputs(search)),
        section(3, sections[3], selection()),
        section(4, sections[4], navigation()),
        section(5, sections[5], display(scene)),
        section(6, sections[6], table(scene)),
        section(7, sections[7], feedback(scene)),
        section(8, sections[8], overlays(scene)),
        section(9, sections[9], layout()),
    ));
    let sidebar = sidebar(rects, scroll_node, offset, current, mode);
    let topbar = (
        g::breadcrumb(vec![
            "Workspace".into(),
            "Design System".into(),
            "Components".into(),
        ]),
        spacer(),
        node()
            .role(Role::Button)
            .name("コマンドパレットを開く")
            .focusable(true)
            .on_press(move || scene.palette.set(true))
            .child(
                row((
                    kivact::control::icon("search"),
                    text("コンポーネント・操作を検索...").style(style().grow(1)),
                    g::mono("Ctrl K"),
                ))
                .style(style().align_items(Align::Center).gap(10)),
            )
            .style(
                style()
                    .width(340)
                    .height(32)
                    .padding_x(10)
                    .background(ColorToken::SurfaceAlt)
                    .border(1, ColorToken::Border)
                    .color(ColorToken::TextMuted),
            ),
        g::button_group((
            kivact::ui::button("標準")
                .variant(kivact_ui_game::Slant)
                .test_id("game-density-comfortable")
                .size(kivact::ui::ControlSize::Small)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .selected(move || density.get() == g::Density::Comfortable)
                .style(
                    style().selected(
                        style()
                            .background(ColorToken::Accent)
                            .color(ColorToken::AccentText),
                    ),
                )
                .on_press(move || density.set(g::Density::Comfortable)),
            kivact::ui::button("高密度")
                .variant(kivact_ui_game::Slant)
                .test_id("game-density-compact")
                .size(kivact::ui::ControlSize::Small)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .selected(move || density.get() == g::Density::Compact)
                .style(
                    style().selected(
                        style()
                            .background(ColorToken::Accent)
                            .color(ColorToken::AccentText),
                    ),
                )
                .on_press(move || density.set(g::Density::Compact)),
        )),
        kivact::ui::tooltip("通知 (3件)")
            .placement(kivact::view::Placement::Below)
            .align(kivact::view::OverlayAlign::Center)
            .offset(6.)
            .attach(
                g::tool_button("通知")
                    .icon(kivact::control::icon("bell"))
                    .hide_label(true)
                    .on_press(move || {
                        toaster().push(
                            Toast::new("通知")
                                .body("3 件の未読通知があります。")
                                .tone(Tone::Info)
                                .duration(Some(Duration::from_secs(4))),
                        );
                    }),
            ),
        kivact::ui::tooltip("テーマ切替")
            .placement(kivact::view::Placement::Below)
            .align(kivact::view::OverlayAlign::Center)
            .offset(6.)
            .attach(
                g::tool_button("テーマ切替")
                    .test_id("game-theme-toggle")
                    .icon(kivact::control::icon("moon"))
                    .hide_label(true)
                    .on_press(move || {
                        mode.update(|v| {
                            *v = if *v == kivact::style::ThemeMode::Light {
                                kivact::style::ThemeMode::Dark
                            } else {
                                kivact::style::ThemeMode::Light
                            }
                        })
                    }),
            ),
        kivact::ui::menu(
            "OP ▾",
            vec![
                MenuEntry::item("profile", "プロフィール"),
                MenuEntry::item("settings", "環境設定").shortcut("Ctrl+,"),
                MenuEntry::item("sign-out", "サインアウト"),
            ],
            move |key: &'static str| match key {
                "profile" => scene.drawer.set(true),
                "settings" => scene.palette.set(true),
                _ => {
                    toaster().push(
                        Toast::new("サインアウト")
                            .body("カタログのデモ操作です。")
                            .tone(Tone::Info)
                            .duration(Some(Duration::from_secs(3))),
                    );
                }
            },
        )
        .offset(0.)
        .attach(
            kivact::ui::button("OP ▾")
                .variant(kivact_ui_game::Slant)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .into_node(),
        ),
    );
    let shell = super::shell::app_shell(sidebar, topbar, content, scroll_node)
        .test_id("game-catalog")
        .on_key(move |e| {
            if (e.modifiers.ctrl || e.modifiers.meta) && e.key == Key::Char('k') {
                scene.palette.update(|v| *v = !*v);
                Handled::Yes
            } else if e.key == Key::Char('/')
                && !matches!(
                    e.target_role,
                    Some(
                        Role::TextBox
                            | Role::CheckBox
                            | Role::Switch
                            | Role::RadioButton
                            | Role::Slider
                            | Role::SpinButton
                    )
                )
                && !scene.create.get()
                && !scene.delete.get()
                && !scene.palette.get()
            {
                search.scroll_into_view();
                search.focus();
                Handled::Yes
            } else {
                Handled::No
            }
        });
    kivact::view::provide_theme(theme, move || {
        column((
            shell,
            kivact::ui::toast_region(toaster()),
            create_dialog(scene),
            delete_dialog(scene),
            kivact::ui::dialog("案件詳細", scene.drawer)
                .kind(kivact::ui::DialogKind::Drawer(kivact::ui::Side::Right))
                .content(drawer_body)
                .footer(move || {
                    (
                        kivact::ui::button("閉じる")
                            .variant(kivact_ui_game::Slant)
                            .test_id("game-drawer-footer-close")
                            .variant(kivact::ui::ButtonVariant::Ghost)
                            .on_press(move || scene.drawer.set(false)),
                        kivact::ui::button("編集")
                            .variant(kivact_ui_game::Slant)
                            .test_id("game-drawer-footer-edit")
                            .on_press(move || scene.drawer.set(false)),
                    )
                }),
            kivact::ui::command_palette(scene.palette, commands, move |value| {
                run_command.try_call(value)
            })
            .footer(|| {
                g::mono(kivact::i18n::message(
                    "palette.hint",
                    "↑ ↓ 移動     Enter 実行     Esc 閉じる",
                ))
            }),
        ))
        .style(style().size(pct(100), pct(100)).font_family("Noto Sans JP"))
    })
}

fn section_names() -> [(&'static str, &'static str, &'static str); 10] {
    [
        (
            "基礎トークン",
            "Foundations",
            "色・書体・形状の基礎トークン",
        ),
        ("ボタン", "Buttons", "主要操作・補助操作・ツールバー"),
        ("テキスト入力", "Inputs", "テキスト・数値・ファイル"),
        (
            "選択コントロール",
            "Selection",
            "チェック・ラジオ・スイッチ・スライダー",
        ),
        ("ナビゲーション", "Navigation", "タブ・ステップ・ページング"),
        ("データ表示", "Data Display", "KPI・ステータス・進捗・履歴"),
        (
            "データテーブル",
            "Data Table",
            "ソート・検索・行選択・一括操作",
        ),
        (
            "フィードバック",
            "Feedback",
            "アラート・トースト・ローディング・空状態",
        ),
        (
            "オーバーレイ",
            "Overlays",
            "メニュー・ダイアログ・ドロワー・コマンドパレット",
        ),
        (
            "ネイティブレイアウト",
            "Native Layout",
            "ウィンドウ・ツリー・インスペクタ・ステータスバー",
        ),
    ]
}

fn section(i: usize, r: NodeRef, content: impl IntoView) -> Node {
    let (_, name, description) = section_names()[i];
    column((
        g::section_heading(format!("{i:02}"), name.to_uppercase(), description),
        content,
    ))
    .node_ref(r)
    .test_id(format!("game-section-{i}"))
    .style(style().margin_top(48).gap(18))
}

fn reveal(scroll: NodeRef, offset: Value<Point>, target: Value<Option<Rect>>) {
    if let Some(rect) = target.get() {
        scroll.scroll_to(
            Point::new(0., (offset.get().y + rect.y - 72.).max(0.)),
            kivact::view::ScrollBehavior::Smooth,
        );
    }
}

fn sidebar(
    refs: Rc<Vec<Value<Option<Rect>>>>,
    scroll_node: NodeRef,
    offset: Value<Point>,
    current: Signal<usize>,
    mode: Signal<kivact::style::ThemeMode>,
) -> Node {
    let mut navigation = Vec::new();
    for (i, (name, _, _)) in section_names().iter().enumerate() {
        if [0, 1, 5, 7].contains(&i) {
            navigation.push(
                g::mono(match i {
                    0 => "FOUNDATION",
                    1 => "CONTROLS",
                    5 => "DATA",
                    _ => "SYSTEM",
                })
                .style(
                    style()
                        .padding_x(10)
                        .padding_top(14)
                        .padding_bottom(6)
                        .font_size(10)
                        .color(rgb(0x7890ac)),
                ),
            );
        }
        let r = refs[i];
        let button = row((
            g::mono(format!("{i:02}")).style(style().color(rgb(0x7890ac)).width(18)),
            text((*name).to_owned()).style(
                style()
                    .grow(1)
                    .font_size(12.5)
                    .weight(500)
                    .text_wrap(false)
                    .color(rgb(0xcfdceb)),
            ),
            g::mono(
                [
                    "TOKENS", "BUTTON", "INPUT", "SELECT", "NAV", "DISPLAY", "TABLE", "FEEDBACK",
                    "OVERLAY", "LAYOUT",
                ][i],
            )
            .style(move || {
                style()
                    .position(Position::Absolute)
                    .right(10)
                    .font_size(9)
                    .color(rgb(0x7890ac))
                    .opacity(if current.get() == i { 1. } else { 0. })
            }),
        ))
        .role(Role::Button)
        .name((*name).to_owned())
        .focusable(true)
        .selected(move || current.get() == i)
        .test_id(format!(
            "game-nav-{}",
            [
                "foundations",
                "buttons",
                "inputs",
                "selection",
                "navigation",
                "display",
                "table",
                "feedback",
                "overlays",
                "layout"
            ][i]
        ))
        .on_press(move || {
            current.set(i);
            reveal(scroll_node, offset, r);
        })
        .style(
            style()
                .height(34)
                .padding_x(10)
                .gap(10)
                .align_items(Align::Center)
                .color(rgb(0xcfdceb))
                .hover(
                    style()
                        .background(rgba(0xffffff, 0.04))
                        .color(rgb(0xffffff)),
                )
                .selected(
                    style()
                        .background(rgba(0x1283f0, 0.22))
                        .border_left(3, ColorToken::Accent)
                        .color(rgb(0xffffff)),
                )
                .focus(style().ring(2, ColorToken::Focus)),
        );
        navigation.push(button);
    }
    column((
        row((
            g::halo().style(style().size(34, 34)),
            column((
                row((
                    text("KIV").style(
                        style()
                            .color(rgb(0xffffff))
                            .font_family("Barlow Condensed")
                            .font_size(24)
                            .weight(800)
                            .italic(true),
                    ),
                    text("ACT").style(
                        style()
                            .color(rgb(0x2ec4f0))
                            .font_family("Barlow Condensed")
                            .font_size(24)
                            .weight(800)
                            .italic(true),
                    ),
                ))
                .style(
                    style()
                        .font_family("Barlow Condensed")
                        .font_size(21)
                        .weight(800)
                        .italic(true),
                ),
                g::mono("OPS // CONSOLE").style(style().font_size(9.5).color(rgb(0x7890ac))),
            ))
            .style(style().gap(4)),
        ))
        .style(
            style()
                .height(64)
                .padding_x(18)
                .align_items(Align::Center)
                .gap(12)
                .border_bottom(1, rgba(0xffffff, 0.08)),
        ),
        scroll(column(navigation))
            .style(style().grow(1).min_height(0).padding_top(14).padding_x(10)),
        column((
            row((
                g::mono("SYS.LINK").style(style().font_size(9.5).color(rgb(0x7890ac))),
                spacer(),
                g::mono("• ONLINE").style(style().font_size(9.5).color(rgb(0x12a06e))),
            )),
            row((
                g::mono("BUILD").style(style().font_size(9.5).color(rgb(0x7890ac))),
                spacer(),
                g::mono("20261003-0940").style(style().font_size(9.5).color(rgb(0x7890ac))),
            )),
            row((
                g::mono("THEME").style(style().font_size(9.5).color(rgb(0x7890ac))),
                spacer(),
                g::mono(move || {
                    (if mode.get() == kivact::style::ThemeMode::Light {
                        "LIGHT"
                    } else {
                        "DARK"
                    })
                    .to_owned()
                })
                .style(style().font_size(9.5).color(rgb(0x7890ac))),
            )),
        ))
        .style(
            style()
                .padding(18)
                .gap(5)
                .font_size(9.5)
                .color(rgb(0x7890ac))
                .border_top(1, rgba(0xffffff, 0.08)),
        ),
    ))
    .style(style().height(pct(100)))
}

fn hero(mode: Signal<kivact::style::ThemeMode>) -> Node {
    row((column((row((node().style(style().size(22, 8).background(rgb(0xffc928)).transform(Transform::new().skew_x(-12))), g::mono("DESIGN SYSTEM // IDEA 20261003-0940").style(style().color(ColorToken::Accent).letter_spacing(2.2)))).style(style().gap(10).align_items(Align::Center)),
        row((text("COMPONENT").style(style().font_family("Barlow Condensed").font_size(54).weight(800).italic(true).line_height(0.95)),
            node().style(style().background(ColorToken::Accent).padding_x(8).transform(Transform::new().skew_x(-12)))
                .child(text("ARCHIVE").style(style().font_family("Barlow Condensed").font_size(54).weight(800).italic(true).line_height(0.95)
                    .color(ColorToken::AccentText).transform(Transform::new().skew_x(12)))))).style(style().gap(8).align_items(Align::Center)),
        text("青と白を基調に、斜行（スキュー）・カットコーナー・HUD ブラケットで構成した、業務向けネイティブ SaaS の UI コンポーネント目録。装飾は最小限、情報密度と操作の確実さを優先する。")
            .style(style().max_width(470).font_size(13.5).color(ColorToken::TextMuted)),
        line((meta("COMPONENTS 45"), meta("STACK KIVACT / RUST"), meta(move || format!("THEME {}", if mode.get() == kivact::style::ThemeMode::Light { "LIGHT" } else { "DARK" })), meta("ICONS Tabler (MIT)"))),
    )).style(style().gap(12).grow(1)), g::halo()))
        .style(style().min_height(250).padding_y(28).padding_x(32).gap(24).align_items(Align::Center).background(ColorToken::Surface).border(1, ColorToken::Border).border_left(6, ColorToken::Accent))
}
