//! Role: The catalog shell: top bar, toolbar, chapter sidebar, hero and chapters.

use super::arrange::timeline;
use super::audio::mixer;
use super::audio_samples::knobs;
use super::audio_samples::meters;
use super::audio_samples::transport_section;
use super::audio_values::smpte;
use super::common::{SECTIONS, keyed_options, line, separator};
use super::controls::buttons;
use super::controls::selection;
use super::controls::tool_palette_size;
use super::editors::animation;
use super::editors::grading;
use super::editors::nodes;
use super::editors::picker;
use super::editors::viewport;
use super::feedback::dialogs;
use super::feedback::menus;
use super::feedback::operator_search;
use super::indicators::indicators;
use super::jobs::jobs;
use super::layout::docking;
use super::layout::layers;
use super::layout::outliner;
use super::layout::properties;
use super::media_pool::media;
use super::numeric_samples::numeric;
use crate::creative::audio_values::TransportState;
use kivact::draw::radial_gradient;
use kivact::draw::{Point, Rect};
use kivact::prelude::{
    ColorToken, IntoView, Key, Node, NodeBuilder, NodeRef, Role, Signal, Value, column, node, rgb,
    row, scroll, signal, spacer, text,
};
use kivact::primitive::NodeExt;
use kivact::style::TextAlign;
use kivact::style::current_theme;
use kivact::style::{Align, Justify, pct, rgba, style};
use kivact::ui::MenuEntry;
use kivact::view::Handled;
use kivact::view::{LiveRegion, PointerPhase, node_ref};
use kivact_ui_creative as ui;
use ui::ThemeOptions;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Category {
    All,
    Controls,
    Audio,
    Layout,
    Animation,
    Compositing,
    Feedback,
}

fn category_for_group(group: usize) -> Category {
    match group {
        1 => Category::Controls,
        2 => Category::Audio,
        3 => Category::Layout,
        4 => Category::Animation,
        5 => Category::Compositing,
        6 => Category::Feedback,
        _ => Category::All,
    }
}

const SOURCE_IDS: [&str; 22] = [
    "buttons",
    "numeric",
    "selection",
    "indicators",
    "transport",
    "knobs",
    "meters",
    "mixer",
    "properties",
    "docking",
    "outliner",
    "media",
    "layers",
    "timeline",
    "animation",
    "nodes",
    "grading",
    "picker",
    "viewport",
    "menus",
    "dialogs",
    "jobs",
];

/// Mount the complete catalog through public component APIs.
pub fn catalog() -> impl IntoView {
    let accent = signal(ui::Accent::Iris);
    let compact = signal(false);
    let category = signal(Some(Category::All));
    let filter = signal(String::new());
    let current = signal(0_usize);
    let sidebar_open = signal(true);
    let status = signal("Ready — Space: Play · F3: Search · F12: Render".to_string());
    let transport = signal(TransportState::default());
    super::audio_values::playback(transport);
    let search = signal(false);
    let pie = signal(ui::PieState::default());
    let pointer = signal(Point::new(0.0, 0.0));
    let pie_zone = node_ref();
    let viewport_ref = node_ref();
    let shading = signal(1_usize);
    let viewport_grid = signal(true);
    let pie_entries = std::rc::Rc::new(vec![
        ui::PieEntry::new("Wireframe", 180.0, "aperture"),
        ui::PieEntry::new("Rendered", 0.0, "eye"),
        ui::PieEntry::new("Solid", 90.0, "box"),
        ui::PieEntry::new("Material Preview", 270.0, "palette"),
        ui::PieEntry::new("Toggle X-Ray", 225.0, "box"),
        ui::PieEntry::new("Toggle Overlays", 315.0, "layout-columns"),
        ui::PieEntry::new("Flat Lighting", 135.0, "app-window"),
        ui::PieEntry::new("Studio Lighting", 45.0, "bulb"),
    ]);
    let choose_pie = move |index: usize| {
        match index {
            0 => shading.set(0),
            1..=3 => shading.set(1),
            4 => shading.set(4),
            5 => viewport_grid.update(|v| *v = !*v),
            _ => {}
        }
        status.set(format!(
            "Shading: {}",
            [
                "Wireframe",
                "Rendered",
                "Solid",
                "Material Preview",
                "Toggle X-Ray",
                "Toggle Overlays",
                "Flat Lighting",
                "Studio Lighting"
            ][index]
        ));
    };
    let refs = std::rc::Rc::new((0..SECTIONS.len()).map(|_| node_ref()).collect::<Vec<_>>());
    let refs_for_shell = refs.clone();
    let rects = std::rc::Rc::new(refs.iter().map(NodeRef::rect).collect::<Vec<_>>());
    let main_scroll = node_ref();
    let main_offset = main_scroll.scroll_offset();
    kivact::view::provide_theme(
        move || {
            let theme = ui::theme(kivact::style::ThemeMode::Dark)
                .accent(accent.get())
                .density(if compact.get() {
                    ui::Density::Compact
                } else {
                    ui::Density::Comfortable
                });
            let mut palette = (*theme.get::<ui::Tokens>()).clone();
            palette.body_font = "Geist".into();
            palette.mono_font = "Geist Mono".into();
            let text = kivact::style::theme::Typography {
                family: Some(palette.body_font.clone()),
                ..theme.text().clone()
            };
            theme.with_text(text).with(palette)
        },
        move || {
            let bodies = vec![
                buttons(status).into_view(),
                numeric(transport).into_view(),
                selection().into_view(),
                indicators().into_view(),
                transport_section(transport).into_view(),
                knobs().into_view(),
                meters(transport).into_view(),
                mixer(transport).into_view(),
                properties().into_view(),
                docking().into_view(),
                outliner().into_view(),
                media().into_view(),
                layers(transport).into_view(),
                timeline(transport, status).into_view(),
                animation(transport).into_view(),
                nodes().into_view(),
                grading().into_view(),
                picker().into_view(),
                viewport(shading, viewport_grid, viewport_ref, pointer).into_view(),
                menus(status, pie, pointer, pie_zone, search).into_view(),
                dialogs(status).into_view(),
                jobs(status).into_view(),
            ];
            let sections = bodies
                .into_iter()
                .enumerate()
                .map(|(i, body)| {
                    let (title, _, group, subtitle) = SECTIONS[i];
                    let collapsed = signal(false);
                    let query_title = title.to_lowercase();
                    column((
                        row((
                            text(format!("{:02}", i + 1))
                                .style(
                                    style()
                                        .font_family("Geist Mono")
                                        .font_size(10.5)
                                        .padding_x(6)
                                        .padding_y(2)
                                        .radius(6),
                                )
                                .style(move || {
                                    let t = &current_theme();
                                    let p = t.get::<ui::Tokens>();
                                    style()
                                        .background(p.accent.with_alpha(0.14))
                                        .color(p.accent)
                                }),
                            text(title)
                                .role(Role::Heading)
                                .style(style().font_size(14).weight(600)),
                            text(subtitle)
                                .style(style().font_size(11.5).grow(1).min_width(0))
                                .style(style().color(ui::Color::Dim)),
                            kivact::ui::button("Copy component link")
                                .hide_label(true)
                                .icon(kivact::control::icon("link"))
                                .variant(kivact::ui::ButtonVariant::Ghost)
                                .on_press(move || {
                                    status.set(format!("Component {:02} · {title}", i + 1))
                                }),
                            kivact::ui::button(format!("Collapse {title}"))
                                .hide_label(true)
                                .icon(kivact::control::icon("chevron-down"))
                                .variant(kivact::ui::ButtonVariant::Ghost)
                                .style(move || {
                                    style().rotate(if collapsed.get() { -90.0 } else { 0.0 })
                                })
                                .on_press(move || collapsed.update(|v| *v = !*v)),
                        ))
                        .style(
                            style()
                                .height(46)
                                .padding_left(16)
                                .padding_right(10)
                                .gap(10)
                                .align_items(Align::Center),
                        )
                        .style(style().border_bottom(1, ui::Color::Hair)),
                        node().hidden(collapsed).child(body),
                    ))
                    .node_ref(refs[i])
                    .test_id(format!("creative-section-{}", i + 1))
                    .hidden(move || {
                        category.get().is_some_and(|selected| {
                            selected != Category::All && selected != category_for_group(group)
                        }) || !query_title.contains(&filter.get().to_lowercase())
                    })
                    .style(
                        style()
                            .background(ui::Color::Island)
                            .radius(12)
                            .border(1, ui::Color::Hair)
                            .min_width(0)
                            .shrink(0),
                    )
                })
                .collect::<Vec<_>>();
            let main =
                scroll(column((hero(compact), sections)).style(style().gap(8).padding_right(2)))
                    .node_ref(main_scroll)
                    .test_id("creative-main")
                    .style(style().grow(1).min_width(0).min_height(0))
                    .style(
                        style()
                            .scrollbar_color(ui::Color::HairStrong, rgba(0, 0.0))
                            .scrollbar_width(kivact::style::ScrollbarWidth::Thin),
                    );
            column((
                topbar(accent, compact, category, sidebar_open, search, status),
                toolbar(transport, status),
                row((
                    sidebar(
                        category,
                        filter,
                        current,
                        rects.clone(),
                        main_scroll,
                        main_offset,
                    )
                    .hidden(move || !sidebar_open.get()),
                    main,
                ))
                .style(move || {
                    style()
                        .grow(1)
                        .basis(0)
                        .min_height(0)
                        .min_width(0)
                        .padding(if compact.get() { 4 } else { 8 })
                        .gap(if compact.get() { 4 } else { 8 })
                        .align_items(Align::Stretch)
                }),
                row((
                    text("◉ Select   ◉ Pan View   ◉ Context Menu").style(style().font_size(10.5)),
                    text(status)
                        .role(Role::Status)
                        .live_region(LiveRegion::Polite)
                        .style(style().grow(1).text_align(TextAlign::Center).font_size(11)),
                    text(move || {
                        format!(
                            "Scene Shot 010   {:.0} f   1920×1080  24 fps   v0.1.0",
                            transport.get().frame
                        )
                    })
                    .style(style().font_family("Geist Mono").font_size(10.5)),
                ))
                .style(
                    style()
                        .height(26)
                        .shrink(0)
                        .padding_x(14)
                        .align_items(Align::Center)
                        .gap(16),
                )
                .style(style().color(ui::Color::Dim)),
                operator_search(search, status, refs_for_shell.clone()),
                kivact::ui::toast_region(kivact::ui::toaster()),
                ui::pie_menu(pie_entries.as_ref().clone(), pie, choose_pie),
            ))
            .test_id("creative-catalog")
            .style(style().size(pct(100), pct(100)).min_height(0))
            .style(
                style()
                    .background(ui::Color::Canvas)
                    .color(ui::Color::Text)
                    .font_family("Geist, 'Noto Sans JP', sans-serif")
                    .font_size(12.5)
                    .line_height(1.4),
            )
            .on_pointer(move |e| {
                if matches!(
                    e.phase(),
                    PointerPhase::Move | PointerPhase::Enter | PointerPhase::Down(_)
                ) {
                    pointer.set(e.window_pos());
                    if pie.get().open {
                        let mut next = pie.get();
                        next.pointer(e.window_pos(), &pie_entries);
                        pie.set(next);
                    }
                }
                Handled::No
            })
            .on_key_up(move |e| {
                if e.key() == Key::Char('z') && pie.get().held {
                    if let Some(index) = pie.get().hot {
                        choose_pie(index);
                    }
                    pie.set(ui::PieState::default());
                    return Handled::Yes;
                }
                Handled::No
            })
            .on_key(move |e| match e.key() {
                Key::Char('z')
                    if !e.repeat()
                        && !e.modifiers().ctrl
                        && !e.modifiers().alt
                        && !pie.get().open
                        && (pie_zone.hit_test(pointer.get())
                            || viewport_ref.hit_test(pointer.get())) =>
                {
                    pie.set(ui::PieState::open_at(pointer.get(), true));
                    Handled::Yes
                }
                Key::F(3) => {
                    search.set(true);
                    Handled::Yes
                }
                Key::F(12) => {
                    status.set("Render Image · Catalog simulation complete".into());
                    Handled::Yes
                }
                Key::Char('d') if e.modifiers().ctrl && e.modifiers().alt => {
                    compact.update(|v| *v = !*v);
                    Handled::Yes
                }
                _ => Handled::No,
            })
        },
    )
}

fn topbar(
    accent: Signal<ui::Accent>,
    compact: Signal<bool>,
    category: Signal<Option<Category>>,
    sidebar: Signal<bool>,
    search: Signal<bool>,
    status: Signal<String>,
) -> impl IntoView {
    let choices = [
        ui::Accent::Iris,
        ui::Accent::Lime,
        ui::Accent::Aqua,
        ui::Accent::Coral,
        ui::Accent::Amber,
    ];
    let colors = choices
        .into_iter()
        .map(|choice| {
            node()
                .role(Role::Button)
                .name(format!("Accent {choice:?}"))
                .focusable(true)
                .selected(move || accent.get() == choice)
                .on_press(move || accent.set(choice))
                .style(
                    style()
                        .size(14, 14)
                        .radius(7)
                        .background(choice.colors().0)
                        .shrink(0)
                        .selected(style().ring(2, choice.colors().0))
                        .hover(style().scale(1.15)),
                )
        })
        .collect::<Vec<_>>();
    let menus = ["File", "Edit", "View", "Render", "Window", "Help"]
        .into_iter()
        .map(|label| {
            kivact::ui::menu(
                label,
                vec![
                    MenuEntry::item("new", "New Project").shortcut("Ctrl N"),
                    MenuEntry::item("open", "Open...").shortcut("Ctrl O"),
                    MenuEntry::separator(),
                    MenuEntry::item("preferences", "Preferences..."),
                ],
                move |key: &'static str| status.set(format!("{label} · {key}")),
            )
            .offset(0.0)
            .attach(
                kivact::ui::button(label)
                    .variant(kivact::ui::ButtonVariant::Ghost)
                    .style(style().height(28).padding_x(8))
                    .into_node(),
            )
        })
        .collect::<Vec<_>>();
    row((
        row((
            node()
                .style(style().size(22, 22).radius(7))
                .style(style().background(ui::Color::Accent))
                .draw(|d, _| {
                    let p = d.theme().get::<ui::Tokens>();
                    for (x, y) in [(4.0, 4.0), (12.0, 4.0), (4.0, 12.0), (12.0, 12.0)] {
                        d.stroke_rect(Rect::new(x, y, 6.0, 6.0), 1.0, p.accent_ink);
                    }
                }),
            text("KIVACT").style(style().weight(700)),
            text("PRO UI").style(style().color(ui::Color::Dim)),
        ))
        .style(style().gap(8).padding_x(8).align_items(Align::Center)),
        kivact::ui::button("Outliner visibility")
            .hide_label(true)
            .icon(kivact::control::icon("sidebar"))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .on_press(move || sidebar.update(|v| *v = !*v)),
        row(menus).style(style().gap(1)),
        kivact::ui::tabs(
            "Category",
            vec![
                kivact::ui::ListItem::new(Category::All, "All 22"),
                kivact::ui::ListItem::new(Category::Controls, "Controls 4"),
                kivact::ui::ListItem::new(Category::Audio, "Audio 4"),
                kivact::ui::ListItem::new(Category::Layout, "Layout 4"),
                kivact::ui::ListItem::new(Category::Animation, "Animation 3"),
                kivact::ui::ListItem::new(Category::Compositing, "Compositing 4"),
                kivact::ui::ListItem::new(Category::Feedback, "Feedback 3"),
            ],
            kivact::binding(
                move || category.get().unwrap_or(Category::All),
                move |next| category.set(Some(next)),
            ),
        )
        .style(style().margin_left(10)),
        spacer(),
        row(colors).style(style().gap(7).padding_x(8).align_items(Align::Center)),
        kivact::ui::button("Compact density")
            .hide_label(true)
            .icon(kivact::control::icon("density"))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .selected(compact)
            .on_press(move || compact.update(|v| *v = !*v)),
        kivact::ui::button("Operator Search")
            .hide_label(true)
            .icon(kivact::control::icon("search"))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .test_id("creative-top-operator-search")
            .on_press(move || search.set(true)),
    ))
    .style(
        style()
            .height(42)
            .gap(4)
            .align_items(Align::Center)
            .padding_right(10)
            .shrink(0),
    )
}

fn toolbar(state: Signal<TransportState>, status: Signal<String>) -> impl IntoView {
    row((
        row((
            kivact::ui::button("Undo")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .hide_label(true)
                .icon(kivact::control::icon("undo"))
                .on_press(move || status.set("Undo · Last UI operation".into())),
            kivact::ui::button("Redo")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .hide_label(true)
                .icon(kivact::control::icon("redo"))
                .on_press(move || status.set("Redo · UI operation".into())),
        ))
        .style(style().gap(2)),
        separator(),
        tool_palette_size(false, true)
            .test_id("creative-toolbar-tools")
            .style(style().align_self(Align::Center)),
        separator(),
        kivact::ui::toggle("", signal(true)).icon(kivact::control::icon("magnet")),
        kivact::ui::select(
            "Snap Target",
            keyed_options([
                ("bar", "Bar"),
                ("beat", "Beat"),
                ("1-8", "1/8"),
                ("1-16", "1/16"),
                ("frame", "Frame"),
                ("second", "Second"),
            ]),
            signal(Some("beat".to_owned())),
        )
        .placeholder("")
        .name("Snap Target")
        .style(style().width(76)),
        separator(),
        ui::transport(super::audio_values::controls(state))
            .on_to_start(move || state.update(|s| s.frame = 0.0)),
        row((
            text("TC")
                .style(style().font_size(9.5))
                .style(style().color(ui::Color::Accent)),
            text(move || smpte(state.get().frame))
                .test_id("catalog-playback-frame")
                .value_text(move || state.get().frame.to_string())
                .style(style().weight(600)),
            text("BAR")
                .style(style().font_size(9.5))
                .style(style().color(ui::Color::Accent)),
            text("1.1.1"),
        ))
        .style(
            style()
                .height(30)
                .padding_x(12)
                .gap(10)
                .radius(9)
                .font_family("Geist Mono")
                .align_items(Align::Center),
        )
        .style(style().background(ui::Color::Canvas)),
        spacer(),
        column(
            [("CPU", 0.22_f64), ("DSK", 0.07_f64)]
                .into_iter()
                .map(|(label, value)| {
                    line((
                        text(label).style(style().font_size(8.0)),
                        kivact::ui::progress(Some(f64::from(value)))
                            .name(label)
                            .style(style().size(56, 4))
                            .draw(move |d, size| {
                                d.rounded_rect(
                                    Rect::new(0.0, 0.0, size.width * value, size.height),
                                    2.0,
                                    rgb(0x34d399),
                                )
                            }),
                    ))
                })
                .collect::<Vec<_>>(),
        )
        .style(style().font_family("Geist Mono").gap(3)),
        separator(),
        kivact::ui::button("Render  F12")
            .icon(kivact::control::icon("aperture"))
            .variant(kivact::ui::ButtonVariant::Primary)
            .on_press(move || status.set("Render Image · Catalog simulation complete".into())),
    ))
    .role(Role::Toolbar)
    .name("Creative toolbar")
    .style(
        style()
            .height(46)
            .margin_x(8)
            .padding_x(8)
            .gap(8)
            .align_items(Align::Center)
            .radius(12)
            .shrink(0),
    )
    .style(
        style()
            .background(ui::Color::Island)
            .border(1, ui::Color::Hair),
    )
}

fn sidebar(
    category: Signal<Option<Category>>,
    filter: Signal<String>,
    current: Signal<usize>,
    rects: std::rc::Rc<Vec<Value<Option<Rect>>>>,
    main_scroll: NodeRef,
    main_offset: Value<Point>,
) -> Node {
    let groups = [
        "CONTROLS",
        "AUDIO",
        "LAYOUT",
        "ANIMATION",
        "COMPOSITING",
        "FEEDBACK",
    ]
    .into_iter()
    .enumerate()
    .map(|(g, title)| {
        let collapsed = signal(false);
        let rects = rects.clone();
        let rows = SECTIONS
            .iter()
            .enumerate()
            .filter(|(_, item)| item.2 == g + 1)
            .map(|(i, (title, code, _, _))| {
                let title = *title;
                let query = title.to_lowercase();
                let rect = rects[i];
                row((
                    text(format!("{:02}", i + 1))
                        .style(style().font_family("Geist Mono").font_size(9).single_line()),
                    kivact::control::icon(section_icon(i)),
                    text(title).style(style().grow(1).single_line()),
                    text(*code)
                        .style(style().font_family("Geist Mono").font_size(9).single_line())
                        .style(style().color(ui::Color::Dim)),
                ))
                .role(Role::TreeItem)
                .name(title)
                .focusable(true)
                .selected(move || current.get() == i)
                .test_id(format!("creative-nav-{}", SOURCE_IDS[i]))
                .hidden(move || collapsed.get() || !query.contains(&filter.get().to_lowercase()))
                .on_press(move || {
                    current.set(i);
                    category.set(Some(Category::All));
                    if let Some(rect) = rect.get() {
                        main_scroll.scroll_to(
                            Point::new(0.0, (main_offset.get().y + rect.y - 100.0).max(0.0)),
                            kivact::view::ScrollBehavior::Smooth,
                        );
                    }
                })
                .style(
                    style()
                        .height(28)
                        .padding_x(8)
                        .gap(7)
                        .radius(7)
                        .align_items(Align::Center)
                        .color(ui::Color::Muted)
                        .hover(
                            style()
                                .background(ColorToken::Hover)
                                .color(ui::Color::TextHigh),
                        )
                        .selected(
                            style()
                                .background(ui::Color::Selection)
                                .color(ui::Color::TextHigh),
                        )
                        .focus(style().ring(1, ui::Color::Accent)),
                )
            })
            .collect::<Vec<_>>();
        column((
            row((
                kivact::control::icon("chevron-down")
                    .style(move || style().rotate(if collapsed.get() { -90.0 } else { 0.0 })),
                kivact::control::icon(
                    [
                        "click",
                        "volume",
                        "layout-columns",
                        "timeline",
                        "schema",
                        "app-window",
                    ][g],
                ),
                text(title).style(style().grow(1)),
                text(format!(
                    "{}",
                    SECTIONS.iter().filter(|s| s.2 == g + 1).count()
                )),
            ))
            .role(Role::Button)
            .name(title)
            .focusable(true)
            .expanded(move || !collapsed.get())
            .style(
                style()
                    .height(26)
                    .font_size(10.5)
                    .justify_content(Justify::Start)
                    .gap(6)
                    .padding_x(7)
                    .align_items(Align::Center)
                    .margin_top(6),
            )
            .style(
                style()
                    .color(ui::Color::Dim)
                    .focus(style().ring(1, ui::Color::Accent)),
            )
            .on_press(move || collapsed.update(|v| *v = !*v)),
            rows,
        ))
        .style(style().shrink(0))
        .hidden(move || {
            category.get().is_some_and(|selected| {
                selected != Category::All && selected != category_for_group(g + 1)
            })
        })
    })
    .collect::<Vec<_>>();
    column((
        row((
            kivact::control::icon("sidebar"),
            text("Outliner").style(style().weight(600).grow(1)),
            text("22 / 22").style(style().font_size(10.5)),
        ))
        .style(
            style()
                .height(36)
                .padding_x(12)
                .gap(7)
                .align_items(Align::Center),
        ),
        kivact::ui::text_input("Filter components", filter)
            .placeholder("Filter components")
            .style(style().margin_x(8).margin_y(8)),
        scroll(
            column(groups)
                .role(Role::Tree)
                .style(style().padding_x(6).padding_bottom(10)),
        )
        .test_id("creative-nav-scroll")
        .style(style().grow(1).basis(0).min_height(0)),
    ))
    .style(
        style()
            .width(248)
            .height(pct(100))
            .min_height(0)
            .shrink(0)
            .radius(12),
    )
    .style(
        style()
            .background(ui::Color::Island)
            .border(1, ui::Color::Hair),
    )
}

fn hero(compact: Signal<bool>) -> impl IntoView {
    row((column((
        text("● COMPONENT CATALOG · 2026 EDITION").style(style().font_family("Geist Mono").font_size(10.5).padding_x(10).padding_y(4).radius(99).align_self(Align::Start))
            .style( style().background(ui::Color::Hair).color(ui::Color::Muted)),
        text("Professional Creative Tool — Native UI Kit").role(Role::Heading).style(style().font_size(30).weight(650).letter_spacing(-0.9).margin_top(14).margin_bottom(10)),
        text("DAW・モーショングラフィックス・3DCG・カラーグレーディング系アプリケーションの文法で組んだネイティブ UI コンポーネント群の 2026 年版。浮遊するアイランド型レイアウト、ベベルの代わりのヘアライン、グラスのオーバーレイ、アクセント色のティントで選択を表現します。数値フィールドはドラッグで値変更・クリックで直接入力・Ctrl で微調整・Shift で粗調整・Backspace で既定値。")
            .style(style().max_width(660).line_height(1.7)).style( style().color(ui::Color::Muted)),
    )).style(style().grow(1)),
        column((text("SECTIONS      22 editors"), text(move || format!("ROW HEIGHT    {} px", if compact.get() { 20 } else { 24 })), text("BASE FONT     12.5 px Geist"), text("ICONS         Tabler (MIT)"), text("DEPENDENCIES  kivact")))
            .style(style().font_family("Geist Mono").font_size(10.5).gap(6).padding(14).radius(10).align_self(Align::End))
            .style(move || { let t = &current_theme(); style().background(t.get::<ui::Tokens>().canvas.with_alpha(0.75)).border(1, t.get::<ui::Tokens>().hair) }),
    )).style(style().min_height(242).padding_x(28).padding_top(30).padding_bottom(26).gap(24).radius(12).shrink(0))
        .style(move || { let t = &current_theme(); let p = t.get::<ui::Tokens>(); style().background((radial_gradient().radii(520, 260).at(pct(92), pct(10)).colors([p.accent.with_alpha(0.3), rgba(0, 0.0)]), p.island)).border(1, p.hair) })
        .draw(|d, size| { for x in (0..size.width as usize).step_by(18) { for y in (0..size.height as usize).step_by(18) { d.rounded_rect(Rect::new(x as f64, y as f64, 1.0, 1.0), 0.5, rgba(0xffffff, 0.06)); } } })
}

fn section_icon(i: usize) -> &'static str {
    [
        "click",
        "adjustments-horizontal",
        "checkbox",
        "bulb",
        "play",
        "adjustments",
        "chart-bar",
        "adjustments-horizontal",
        "adjustments",
        "layout-columns",
        "hierarchy",
        "table",
        "stack",
        "timeline",
        "keyframes",
        "schema",
        "palette",
        "color-picker",
        "box",
        "menu",
        "app-window",
        "loader",
    ][i]
}
