//! Role: Feedback UI including menus and toasts.

use super::common::{SECTIONS, col, keyed_options, line, note, spec, specs};
use super::controls::num;
use super::controls::slide;
use kivact::geometry::Point;
use kivact::interaction::MenuEntry;
use kivact::prelude::{
    IntoView, Key, NodeBuilder, NodeRef, Role, Signal, Theme, column, for_each, memo, node,
    on_mount, row, scroll, show, signal, text,
};
use kivact::state::callback;
use kivact::style::GridPlacement;
use kivact::style::use_theme;
use kivact::style::{Align, Justify, Style, pct, rgba, style};
use kivact::view::{Handled, Placement, node_ref, overlay};
use kivact_ui_creative as ui;

pub(super) fn menus(
    status: Signal<String>,
    pie: Signal<ui::PieState>,
    pointer: Signal<Point>,
    pie_zone: NodeRef,
    search: Signal<bool>,
) -> impl IntoView {
    let entries = vec![
        MenuEntry::item("move", "Move").with_shortcut("G"),
        MenuEntry::item("rotate", "Rotate").with_shortcut("R"),
        MenuEntry::item("scale", "Scale").with_shortcut("S"),
        MenuEntry::separator(),
        MenuEntry::item("duplicate", "Duplicate").with_shortcut("Shift D"),
        MenuEntry::item("delete", "Delete").with_shortcut("X"),
        MenuEntry::item("apply", "Apply Transform").disabled(true),
    ];
    specs((
        spec(
            "Context Menu",
            "right click",
            kivact::ui::menu("Object", entries)
                .on_activate(move |key: &'static str| status.set(format!("Context · {key}")))
                .attach_context(
                    node()
                        .role(Role::Group)
                        .name("Context menu target")
                        .focusable(true)
                        .style(
                            style()
                                .height(156)
                                .padding(14)
                                .radius(8)
                                .align_items(Align::Center)
                                .justify_content(Justify::Center),
                        )
                        .style(
                            style()
                                .background(ui::Color::Canvas)
                                .border(1, ui::Color::Hair),
                        )
                        .child(note("Right click here · G Move / R Rotate / S Scale")),
                ),
        ),
        spec(
            "Pie Menu",
            "eight directions",
            kivact::ui::button("Hover here and hold Z — release to choose")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .with_node(|node| node.node_ref(pie_zone))
                .test_id("creative-pie-zone")
                .on_pointer(move |event| {
                    pointer.set(event.window_pos());
                    Handled::No
                })
                .style(style().height(156).width(pct(100)))
                .on_press(move || pie.set(ui::PieState::open_at(pointer.get(), false))),
        ),
        spec(
            "Dropdown / Search",
            "ドロップダウン・検索",
            col((
                note(
                    "F3 または右上の検索アイコンから、全 22 章をコマンドとして検索。↑↓ 選択 / Enter 実行 / Esc 閉じる。",
                ),
                line((
                    kivact::ui::menu(
                        "Add",
                        vec![
                            MenuEntry::submenu(
                                "mesh",
                                "Mesh",
                                vec![
                                    MenuEntry::item("plane", "Plane"),
                                    MenuEntry::item("cube", "Cube"),
                                ],
                            ),
                            MenuEntry::item("camera", "Camera"),
                        ],
                    )
                    .on_activate(move |key: &'static str| status.set(format!("Add · {key}")))
                    .offset(0.0)
                    .attach(
                        kivact::ui::button("Add")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .trailing(kivact::view::icon("chevron-down")),
                    ),
                    kivact::ui::menu(
                        "Select",
                        vec![
                            MenuEntry::item("all", "All"),
                            MenuEntry::item("none", "None"),
                            MenuEntry::item("invert", "Invert"),
                        ],
                    )
                    .on_activate(move |key: &'static str| status.set(format!("Select · {key}")))
                    .offset(0.0)
                    .attach(
                        kivact::ui::button("Select")
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .trailing(kivact::view::icon("chevron-down")),
                    ),
                    kivact::ui::button("Operator Search")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .icon(kivact::view::icon("search"))
                        .test_id("creative-section-operator-search")
                        .on_press(move || search.set(true)),
                )),
            )),
        ),
    ))
}

fn glass(t: &Theme) -> Style {
    let p = t.ext::<ui::Tokens>();
    style()
        .background(rgba(0x1e1f26, 0.94))
        .backdrop_blur(22)
        .color(p.text)
        .border(1, p.hair_strong)
        .shadow(kivact::draw::Shadow::new(0.0, 18.0, 45.0, rgba(0, 0.65)))
}

pub(super) fn dialogs(status: Signal<String>) -> impl IntoView {
    specs((
        spec(
            "Dialog",
            "modal · focus trap",
            col((
                line((
                    render_settings(status),
                    close_project(status),
                )),
                note("モーダルは背景入力を遮断し、Escape と Close でフォーカスが戻ります。"),
            )),
        ),
        spec(
            "Popover",
            "anchored · nonmodal",
            ui::popup_button("Viewport Options", move || {
                col((
                    text("Overlays").style(style().weight(600)),
                    kivact::ui::checkbox("Grid Floor", signal(true)),
                    kivact::ui::checkbox("Axes X", signal(true)),
                    kivact::ui::checkbox("Axes Y", signal(true)),
                    kivact::ui::checkbox("Relationship Lines", signal(true)),
                    ui::property_row("Opacity", slide("", 1.0)),
                    ui::property_row("Scale", num("", 1.0, 0.1, 10.0, 2, "")),
                ))
            }),
        ),
        spec(
            "Tooltip",
            "hover · anchor",
            col((line((
                kivact::ui::tooltip("Camera View").placement(kivact::view::Placement::Below).align(kivact::view::OverlayAlign::Center).offset(6.0).content(move || column((text("Camera View").style(style().weight(600)),text("Align viewport to the active camera").style(style().font_size(11)),text("Numpad 0").style(move || { let theme=kivact::style::use_theme().get(); style().font_family(theme.ext::<ui::Tokens>().mono_font.as_str()).font_size(10).color(theme.ext::<ui::Tokens>().dim) }),))).attach(kivact::ui::button("Hover me").variant(kivact::ui::ButtonVariant::Secondary)),
                kivact::ui::tooltip("Scale").placement(kivact::view::Placement::Below).align(kivact::view::OverlayAlign::Center).offset(6.0).content(move || column((text("Scale").style(style().weight(600)),text("Scale the selection").style(style().font_size(11)),text("S").style(move || { let theme=kivact::style::use_theme().get(); style().font_family(theme.ext::<ui::Tokens>().mono_font.as_str()).font_size(10).color(theme.ext::<ui::Tokens>().dim) }),))).attach(kivact::ui::button("S").variant(kivact::ui::ButtonVariant::Secondary)),
                kivact::ui::tooltip("Tool information").placement(kivact::view::Placement::Below).align(kivact::view::OverlayAlign::Center).offset(6.0).content(move || column((text("Tool information").style(style().weight(600)),text("Public Kivact view and modifier API").style(style().font_size(11)),))).attach(kivact::ui::button("API tooltip").variant(kivact::ui::ButtonVariant::Secondary).hide_label(true).icon(kivact::view::icon("info"))),
            )),note("0.5 秒ホバーで表示。タイトル・説明・ショートカット・API パスの4要素。"))),
        ),
        spec(
            "Banner",
            "インラインバナー",
            col((
                ui::notice(
                    kivact::interaction::Tone::Warning,
                    "Media Offline",
                    "3 clips could not be found at D:/Footage/Day02/",
                )
                .dismissible(true),
                ui::notice(kivact::interaction::Tone::Danger, "Shader compile error", "Metal.002 · line 42 · undeclared identifier rm").dismissible(true),
                ui::notice(kivact::interaction::Tone::Info,"Proxy media", "Proxy media is being used for playback. Final render uses full-resolution sources.").dismissible(true),
            )),
        ).style(style().grid_column(GridPlacement::span(2)).min_height(250)),
    ))
}

fn window_options(title: &str, width: f64) -> ui::WindowOptions {
    ui::WindowOptions::default()
        .title(title)
        .width(width)
        .frame_padding(0.0)
        .body_padding(0.0)
        .gap(0.0)
        .radius(16.0)
        .titlebar_height(40.0)
        .titlebar_padding(14.0, 8.0)
}

fn window_footer(content: impl IntoView) -> impl kivact::view::Component {
    row(content)
        .style(
            style()
                .padding_y(10)
                .padding_x(12)
                .gap(8)
                .align_items(Align::Center),
        )
        .style(style().border_top(1, ui::Color::Hair))
}

fn render_settings(status: Signal<String>) -> impl kivact::view::Component {
    ui::window_button(
        "Render Settings…",
        window_options("Render Settings", 340.0)
            .icon("settings")
            .position(ui::WindowPosition::TopRight {
                right: 40.0,
                top: 120.0,
            }),
        move |close| {
            let tab = signal("format".to_owned());
            column((
                kivact::ui::tabs(
                    "Format options",
                    keyed_options([("format", "Format"), ("encoding", "Encoding")]),
                    tab,
                )
                .style(style().height(40).padding_x(8).padding_y(0).radius(0))
                .style(
                    style()
                        .background(rgba(0, 0.0))
                        .border(0, rgba(0, 0.0))
                        .border_bottom(1, ui::Color::Hair),
                )
                .keep_mounted()
                .content(|key| {
                    if key == "format" {
                        column((
                            ui::property_row(
                                "Preset",
                                kivact::ui::select(
                                    "Render preset",
                                    keyed_options([
                                        ("hdtv-1080p", "HDTV 1080p"),
                                        ("4k-uhd", "4K UHD"),
                                        ("4k-dci", "4K DCI"),
                                    ]),
                                    signal(Some("hdtv-1080p".to_owned())),
                                )
                                .placeholder("")
                                .name("Render preset")
                                .style(style().width(pct(100))),
                            ),
                            ui::property_row(
                                "Resolution",
                                column((
                                    num("X", 1920.0, 1.0, 65536.0, 0, " px"),
                                    num("Y", 1080.0, 1.0, 65536.0, 0, " px"),
                                ))
                                .style(style().gap(1)),
                            ),
                            ui::property_row(
                                "Scale",
                                ui::value_slider("Resolution scale", signal(100.0))
                                    .label_visible(false)
                                    .range(1.0..=100.0)
                                    .step(1.0)
                                    .precision(0)
                                    .unit(" %")
                                    .style(style().width(pct(100))),
                            ),
                            ui::property_row(
                                "Frame Rate",
                                kivact::ui::select(
                                    "Render frame rate",
                                    keyed_options([
                                        ("23-98", "23.98"),
                                        ("24", "24"),
                                        ("25", "25"),
                                        ("30", "30"),
                                        ("60", "60"),
                                    ]),
                                    signal(Some("24".to_owned())),
                                )
                                .placeholder("")
                                .name("Render frame rate")
                                .style(style().width(pct(100))),
                            ),
                        ))
                        .style(style().padding(14).gap(3))
                    } else {
                        column((
                            ui::property_row(
                                "Container",
                                kivact::ui::select(
                                    "Render container",
                                    keyed_options([
                                        ("quicktime", "QuickTime"),
                                        ("mp4", "MP4"),
                                        ("mxf-op1a", "MXF OP1a"),
                                        ("matroska", "Matroska"),
                                    ]),
                                    signal(Some("quicktime".to_owned())),
                                )
                                .placeholder("")
                                .name("Render container")
                                .style(style().width(pct(100))),
                            ),
                            ui::property_row(
                                "Codec",
                                kivact::ui::select(
                                    "Render codec",
                                    keyed_options([
                                        ("prores-422-hq", "ProRes 422 HQ"),
                                        ("prores-4444", "ProRes 4444"),
                                        ("dnxhr-hqx", "DNxHR HQX"),
                                        ("h265", "H.265"),
                                    ]),
                                    signal(Some("prores-422-hq".to_owned())),
                                )
                                .placeholder("")
                                .name("Render codec")
                                .style(style().width(pct(100))),
                            ),
                            ui::property_row(
                                "Audio",
                                kivact::ui::checkbox("Embed 48 kHz PCM", signal(true)),
                            ),
                        ))
                        .style(style().padding(14).gap(3))
                    }
                }),
                window_footer((
                    text("~ 3.2 GB")
                        .style(style().font_family("Geist Mono").font_size(10.5).grow(1))
                        .style(style().color(ui::Color::Dim)),
                    kivact::ui::button("Cancel")
                        .variant(kivact::ui::ButtonVariant::Secondary)
                        .on_press(move || {
                            status.set("Render settings cancelled".into());
                            close.close();
                        }),
                    kivact::ui::button("Render")
                        .variant(kivact::ui::ButtonVariant::Primary)
                        .on_press(move || {
                            status.set("Render requested (demo)".into());
                            close.close();
                        }),
                )),
            ))
            .style(style().gap(0))
        },
    )
    .test_id("creative-export-dialog")
}

fn close_project(status: Signal<String>) -> impl kivact::view::Component {
    ui::window_button(
        "Close Project…",
        window_options("Close Project", 440.0).modal(true).backdrop_blur(6.0),
        move |close| {
            let default_action = node_ref();
            on_mount(move || default_action.focus());
            let save = callback(move |()| {
                status.set("Saved and closed (demo)".into());
                close.close();
            });
            column((
                row((
                    node().style(style().size(40, 40).padding(8).radius(12).shrink(0))
                        .style(move || { let t = &use_theme().get(); style().background(t.ext::<ui::Tokens>().warning.with_alpha(0.12)) })
                        .child(kivact::view::icon("warning").style(style().size(24, 24))
                            .style( style().color(ui::Color::Warning))),
                    column((
                        text("Save changes to “Shot_010_Comp_v012” before closing?")
                            .role(Role::Heading).style(style().font_size(14).weight(600).line_height(1.4)),
                        text("3 unsaved changes. If you don't save, changes since 10:42 will be lost.")
                            .style(style().font_size(12.5).line_height(1.65))
                            .style( style().color(ui::Color::Muted)),
                    )).style(style().width(0).grow(1).gap(6)),
                )).style(style().padding(14).gap(14).align_items(Align::Start)),
                window_footer((
                    kivact::ui::button("Don't Save").variant(kivact::ui::ButtonVariant::Secondary).on_press(move || {
                        status.set("Closed without saving (demo)".into());
                        close.close();
                    }),
                    node().style(style().grow(1)),
                    kivact::ui::button("Cancel").variant(kivact::ui::ButtonVariant::Secondary).on_press(move || {
                        status.set("Close cancelled".into());
                        close.close();
                    }),
                    kivact::ui::button("Save").variant(kivact::ui::ButtonVariant::Primary).with_node(|node| node.node_ref(default_action)).on_press(move || save.call(())),
                )),
            ))
            .style(style().gap(0))
            .on_key(move |event| {
                if event.key() == Key::Enter {
                    save.call(());
                    Handled::Yes
                } else {
                    Handled::No
                }
            })
        },
    ).test_id("creative-close-project-dialog")
}

pub(super) fn operator_search(
    open: Signal<bool>,
    status: Signal<String>,
    refs: std::rc::Rc<Vec<NodeRef>>,
) -> impl IntoView {
    show(open, move || {
        let query = signal(String::new());
        let hot = signal(0_usize);
        let field = node_ref();
        on_mount(move || field.focus());
        let matches = memo(move || {
            let q = query.get().to_lowercase();
            SECTIONS
                .iter()
                .enumerate()
                .filter(|(_, s)| s.0.to_lowercase().contains(&q))
                .map(|(i, _)| i)
                .collect::<Vec<_>>()
        });
        let choices = refs.clone();
        let keys = refs.clone();
        let rows = for_each(
            matches,
            |i| *i,
            move |item| {
                let i = item.get();
                let anchor = choices[i];
                kivact::ui::button(SECTIONS[i].0)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .style(style().justify_content(Justify::Start).height(30))
                    .with_node(|node| {
                        node.selected(move || {
                            matches.with(|m| m.get(hot.get()).copied()) == Some(i)
                        })
                    })
                    .on_press(move || {
                        anchor.scroll_into_view();
                        open.set(false);
                        status.set(format!("Operator · {}", SECTIONS[i].0));
                    })
            },
        );
        overlay(
            node_ref(),
            Placement::Center,
            col((
                line((
                    kivact::view::icon("search"),
                    kivact::ui::text_input("Search operators", query)
                        .placeholder("Search operators...")
                        .with_node(|node| node.node_ref(field))
                        .style(style().grow(1)),
                    text("ESC").style(style().font_family("Geist Mono").font_size(10)),
                )),
                scroll(column(rows)).style(style().max_height(330)),
                note("↑ ↓ navigate  ·  ↵ execute  ·  Esc close"),
            ))
            .style(move || glass(&use_theme().get()))
            .style(style().width(520).padding(14).radius(12).gap(10)),
        )
        .with_node(|node| node.role(Role::Dialog))
        .name("Operator Search")
        .modal()
        .on_dismiss(move |_| open.set(false))
        .on_key(move |e| {
            match e.key() {
                Key::ArrowDown => {
                    hot.update(|i| *i = (*i + 1).min(matches.with(Vec::len).saturating_sub(1)))
                }
                Key::ArrowUp => hot.update(|i| *i = i.saturating_sub(1)),
                Key::Enter => {
                    if let Some(i) = matches.with(|m| m.get(hot.get()).copied()) {
                        keys[i].scroll_into_view();
                        open.set(false);
                        status.set(format!("Operator · {}", SECTIONS[i].0));
                    }
                }
                _ => return Handled::No,
            };
            Handled::Yes
        })
    })
}
