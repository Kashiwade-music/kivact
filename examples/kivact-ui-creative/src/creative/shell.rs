//! Role: The catalog shell: top bar, toolbar, chapter sidebar, hero and chapters.

use super::arrange::timeline;
use super::audio::mixer;
use super::audio_samples::knobs;
use super::audio_samples::meters;
use super::audio_samples::transport_section;
use super::chrome::{
    accent_controls, application_menus, catalog_status, performance_meters, transport_readout,
};
use super::common::{SECTIONS, keyed_options, separator};
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
use kivact::geometry::Point;
use kivact::geometry::Rect;
use kivact::prelude::{
    ColorToken, Element, IntoView, Key, Node, NodeRef, Role, Signal, Value, column, node, row,
    scroll, signal, spacer, text,
};
use kivact::style::use_theme;
use kivact::style::{Align, Justify, pct, rgba, style};
use kivact::view::Handled;
use kivact::view::{PointerPhase, node_ref};
use kivact_ui_creative as ui;

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
    let controls = CatalogControls::new();
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
    let refs = std::rc::Rc::new((0..SECTIONS.len()).map(|_| node_ref()).collect::<Vec<_>>());
    let rects = std::rc::Rc::new(refs.iter().map(NodeRef::rect).collect::<Vec<_>>());
    kivact::style::provide_theme(
        move || catalog_theme(controls.accent, controls.compact),
        move || catalog_frame(controls, refs.clone(), rects.clone(), pie_entries.clone()),
    )
}

fn topbar(
    accent: Signal<ui::Accent>,
    compact: Signal<bool>,
    category: Signal<Category>,
    sidebar: Signal<bool>,
    search: Signal<bool>,
    status: Signal<String>,
) -> impl IntoView {
    let colors = accent_controls(accent);
    let menus = application_menus(status);
    row((
        row((
            node()
                .style(style().size(22, 22).radius(7))
                .style(style().background(ui::ColorName::Accent))
                .draw(|d, _| {
                    let p = d.theme().ext::<ui::Tokens>();
                    for (x, y) in [(4.0, 4.0), (12.0, 4.0), (4.0, 12.0), (12.0, 12.0)] {
                        d.stroke_rect(Rect::new(x, y, 6.0, 6.0), 1.0, p.accent_ink);
                    }
                }),
            text("KIVACT").style(style().weight(700)),
            text("PRO UI").style(style().color(ui::ColorName::Dim)),
        ))
        .style(style().gap(8).padding_x(8).align_items(Align::Center)),
        kivact::ui::button("Outliner visibility")
            .hide_label(true)
            .icon(kivact::view::icon("sidebar"))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .on_press(move || sidebar.update(|v| *v = !*v)),
        row(menus).style(style().gap(1)),
        kivact::ui::tabs(
            "Category",
            vec![
                kivact::interaction::ListItem::new(Category::All, "All 22"),
                kivact::interaction::ListItem::new(Category::Controls, "Controls 4"),
                kivact::interaction::ListItem::new(Category::Audio, "Audio 4"),
                kivact::interaction::ListItem::new(Category::Layout, "Layout 4"),
                kivact::interaction::ListItem::new(Category::Animation, "Animation 3"),
                kivact::interaction::ListItem::new(Category::Compositing, "Compositing 4"),
                kivact::interaction::ListItem::new(Category::Feedback, "Feedback 3"),
            ],
            kivact::state::binding(move || category.get(), move |next| category.set(next)),
        )
        .style(style().margin_left(10)),
        spacer(),
        row(colors).style(style().gap(7).padding_x(8).align_items(Align::Center)),
        kivact::ui::button("Compact density")
            .hide_label(true)
            .icon(kivact::view::icon("density"))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .with_node(|node| node.selected(compact))
            .on_press(move || compact.update(|v| *v = !*v)),
        kivact::ui::button("Operator Search")
            .hide_label(true)
            .icon(kivact::view::icon("search"))
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
                .icon(kivact::view::icon("undo"))
                .on_press(move || status.set("Undo · Last UI operation".into())),
            kivact::ui::button("Redo")
                .variant(kivact::ui::ButtonVariant::Secondary)
                .hide_label(true)
                .icon(kivact::view::icon("redo"))
                .on_press(move || status.set("Redo · UI operation".into())),
        ))
        .style(style().gap(2)),
        separator(),
        tool_palette_size(false, true)
            .test_id("creative-toolbar-tools")
            .style(style().align_self(Align::Center)),
        separator(),
        kivact::ui::toggle_button("", signal(true)).icon(kivact::view::icon("magnet")),
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
        transport_readout(state),
        spacer(),
        performance_meters(),
        separator(),
        kivact::ui::button("Render  F12")
            .icon(kivact::view::icon("aperture"))
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
            .background(ui::ColorName::Island)
            .border(1, ui::ColorName::Hair),
    )
}

fn sidebar(
    category: Signal<Category>,
    filter: Signal<String>,
    current: Signal<usize>,
    rects: std::rc::Rc<Vec<Value<Option<Rect>>>>,
    main_scroll: NodeRef,
    main_offset: Value<Point>,
) -> Node {
    let navigation = CatalogNavigation {
        category,
        filter,
        current,
        rects,
        main_scroll,
        main_offset,
    };
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
    .map(|(g, title)| sidebar_group(g, title, navigation.clone()))
    .collect::<Vec<_>>();
    column((
        row((
            kivact::view::icon("sidebar"),
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
            .background(ui::ColorName::Island)
            .border(1, ui::ColorName::Hair),
    )
}

fn hero(compact: Signal<bool>) -> impl IntoView {
    row((column((
        text("● COMPONENT CATALOG · 2026 EDITION").style(style().font_family("Geist Mono").font_size(10.5).padding_x(10).padding_y(4).radius(99).align_self(Align::Start))
            .style( style().background(ui::ColorName::Hair).color(ui::ColorName::Muted)),
        text("Professional Creative Tool — Native UI Kit").role(Role::Heading).style(style().font_size(30).weight(650).letter_spacing(-0.9).margin_top(14).margin_bottom(10)),
        text("DAW・モーショングラフィックス・3DCG・カラーグレーディング系アプリケーションの文法で組んだネイティブ UI コンポーネント群の 2026 年版。浮遊するアイランド型レイアウト、ベベルの代わりのヘアライン、グラスのオーバーレイ、アクセント色のティントで選択を表現します。数値フィールドはドラッグで値変更・クリックで直接入力・Ctrl で微調整・Shift で粗調整・Backspace で既定値。")
            .style(style().max_width(660).line_height(1.7)).style( style().color(ui::ColorName::Muted)),
    )).style(style().grow(1)),
        column((text("SECTIONS      22 editors"), text(move || format!("ROW HEIGHT    {} px", if compact.get() { 20 } else { 24 })), text("BASE FONT     12.5 px Geist"), text("ICONS         Tabler (MIT)"), text("DEPENDENCIES  kivact")))
            .style(style().font_family("Geist Mono").font_size(10.5).gap(6).padding(14).radius(10).align_self(Align::End))
            .style(move || { let t = &use_theme().get(); style().background(t.ext::<ui::Tokens>().canvas.with_alpha(0.75)).border(1, t.ext::<ui::Tokens>().hair) }),
    )).style(style().min_height(242).padding_x(28).padding_top(30).padding_bottom(26).gap(24).radius(12).shrink(0))
        .style(move || { let t = &use_theme().get(); let p = t.ext::<ui::Tokens>(); style().background((radial_gradient().radii(520, 260).at(pct(92), pct(10)).colors([p.accent.with_alpha(0.3), rgba(0, 0.0)]), p.island)).border(1, p.hair) })
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

#[derive(Clone)]
struct CatalogNavigation {
    category: Signal<Category>,
    filter: Signal<String>,
    current: Signal<usize>,
    rects: std::rc::Rc<Vec<Value<Option<Rect>>>>,
    main_scroll: NodeRef,
    main_offset: Value<Point>,
}

fn sidebar_group(g: usize, title: &'static str, navigation: CatalogNavigation) -> Node {
    let collapsed = signal(false);
    let CatalogNavigation {
        category,
        filter,
        current,
        rects,
        main_scroll,
        main_offset,
    } = navigation;
    let rows = sidebar_rows(
        g,
        collapsed,
        CatalogNavigation {
            category,
            filter,
            current,
            rects,
            main_scroll,
            main_offset,
        },
    );
    column((
        row((
            kivact::view::icon("chevron-down")
                .style(move || style().rotate(if collapsed.get() { -90.0 } else { 0.0 })),
            kivact::view::icon(
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
                .color(ui::ColorName::Dim)
                .focus(style().ring(1, ui::ColorName::Accent)),
        )
        .on_press(move || collapsed.update(|v| *v = !*v)),
        rows,
    ))
    .style(style().shrink(0))
    .hidden(move || {
        let selected = category.get();
        selected != Category::All && selected != category_for_group(g + 1)
    })
}

fn sidebar_rows(g: usize, collapsed: Signal<bool>, navigation: CatalogNavigation) -> Vec<Node> {
    let CatalogNavigation {
        category,
        filter,
        current,
        rects,
        main_scroll,
        main_offset,
    } = navigation;
    SECTIONS
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
                kivact::view::icon(section_icon(i)),
                text(title).style(style().grow(1).single_line()),
                text(*code)
                    .style(style().font_family("Geist Mono").font_size(9).single_line())
                    .style(style().color(ui::ColorName::Dim)),
            ))
            .role(Role::TreeItem)
            .name(title)
            .focusable(true)
            .with_node(|node| node.selected(move || current.get() == i))
            .test_id(format!("creative-nav-{}", SOURCE_IDS[i]))
            .hidden(move || collapsed.get() || !query.contains(&filter.get().to_lowercase()))
            .on_press(move || {
                current.set(i);
                category.set(Category::All);
                if let Some(rect) = rect.get() {
                    main_scroll.scroll_to(
                        Point::new(0.0, (main_offset.get().y + rect.y - 100.0).max(0.0)),
                        kivact::view::ScrollBehavior::Smooth(kivact::motion::Easing::EaseOut),
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
                    .color(ui::ColorName::Muted)
                    .hover(
                        style()
                            .background(ColorToken::Hover)
                            .color(ui::ColorName::TextHigh),
                    )
                    .selected(
                        style()
                            .background(ui::ColorName::Selection)
                            .color(ui::ColorName::TextHigh),
                    )
                    .focus(style().ring(1, ui::ColorName::Accent)),
            )
        })
        .collect::<Vec<_>>()
}

fn choose_shading(
    index: usize,
    shading: Signal<usize>,
    viewport_grid: Signal<bool>,
    status: Signal<String>,
) {
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
}

fn catalog_theme(accent: Signal<ui::Accent>, compact: Signal<bool>) -> kivact::style::Theme {
    let theme = ui::theme_with(
        kivact::style::ThemeMode::Dark,
        ui::ThemeOptions {
            accent: accent.get(),
            density: if compact.get() {
                ui::Density::Compact
            } else {
                ui::Density::Comfortable
            },
        },
    );
    let mut palette = (*theme.ext::<ui::Tokens>()).clone();
    palette.body_font = "Geist".into();
    palette.mono_font = "Geist Mono".into();
    let text = kivact::style::theme::Typography {
        family: Some(palette.body_font.clone()),
        ..theme.text().clone()
    };
    theme.with_text(text).with(palette)
}

fn catalog_section(
    i: usize,
    body: kivact::view::View,
    reference: NodeRef,
    category: Signal<Category>,
    filter: Signal<String>,
    status: Signal<String>,
) -> Node {
    let (title, _, group, subtitle) = SECTIONS[i];
    let collapsed = signal(false);
    let query_title = title.to_lowercase();
    column((
        section_heading(i, title, subtitle, collapsed, status),
        node().hidden(collapsed).child(body),
    ))
    .with_node(|node| node.node_ref(reference))
    .test_id(format!("creative-section-{}", i + 1))
    .hidden(move || {
        let selected = category.get();
        (selected != Category::All && selected != category_for_group(group))
            || !query_title.contains(&filter.get().to_lowercase())
    })
    .style(
        style()
            .background(ui::ColorName::Island)
            .radius(12)
            .border(1, ui::ColorName::Hair)
            .min_width(0)
            .shrink(0),
    )
}

fn section_heading(
    i: usize,
    title: &'static str,
    subtitle: &'static str,
    collapsed: Signal<bool>,
    status: Signal<String>,
) -> Node {
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
                let t = &use_theme().get();
                let p = t.ext::<ui::Tokens>();
                style()
                    .background(p.accent.with_alpha(0.14))
                    .color(p.accent)
            }),
        text(title)
            .role(Role::Heading)
            .style(style().font_size(14).weight(600)),
        text(subtitle)
            .style(style().font_size(11.5).grow(1).min_width(0))
            .style(style().color(ui::ColorName::Dim)),
        kivact::ui::button("Copy component link")
            .hide_label(true)
            .icon(kivact::view::icon("link"))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .on_press(move || status.set(format!("Component {:02} · {title}", i + 1))),
        kivact::ui::button(format!("Collapse {title}"))
            .hide_label(true)
            .icon(kivact::view::icon("chevron-down"))
            .variant(kivact::ui::ButtonVariant::Ghost)
            .style(move || style().rotate(if collapsed.get() { -90.0 } else { 0.0 }))
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
    .style(style().border_bottom(1, ui::ColorName::Hair))
}

fn catalog_pointer(
    e: kivact::view::PointerEvent,
    pointer: Signal<Point>,
    pie: Signal<ui::PieState>,
    pie_entries: &[ui::PieEntry],
) -> Handled {
    if matches!(
        e.phase(),
        PointerPhase::Move | PointerPhase::Enter | PointerPhase::Down(_)
    ) {
        pointer.set(e.window_pos());
        if matches!(pie.get(), ui::PieState::Open { .. }) {
            let mut next = pie.get();
            next.pointer(e.window_pos(), pie_entries);
            pie.set(next);
        }
    }
    Handled::No
}

fn catalog_key_up(
    e: kivact::view::KeyEvent,
    pie: Signal<ui::PieState>,
    choose_pie: impl Fn(usize),
) -> Handled {
    if e.key() == Key::Char('z')
        && matches!(
            pie.get(),
            ui::PieState::Open {
                mode: ui::PieMode::Held,
                ..
            }
        )
    {
        if let ui::PieState::Open {
            hot: Some(index), ..
        } = pie.get()
        {
            choose_pie(index);
        }
        pie.set(ui::PieState::default());
        return Handled::Yes;
    }
    Handled::No
}

fn catalog_key(
    e: kivact::view::KeyEvent,
    pie: Signal<ui::PieState>,
    pointer: Signal<Point>,
    zones: (NodeRef, NodeRef),
    search: Signal<bool>,
    status: Signal<String>,
    compact: Signal<bool>,
) -> Handled {
    let (pie_zone, viewport_ref) = zones;
    match e.key() {
        Key::Char('z')
            if !e.repeat()
                && !e.modifiers().ctrl
                && !e.modifiers().alt
                && matches!(pie.get(), ui::PieState::Closed)
                && (pie_zone.hit_test(pointer.get()) || viewport_ref.hit_test(pointer.get())) =>
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
    }
}

fn catalog_frame(
    controls: CatalogControls,
    refs: std::rc::Rc<Vec<NodeRef>>,
    rects: std::rc::Rc<Vec<Value<Option<Rect>>>>,
    pie_entries: std::rc::Rc<Vec<ui::PieEntry>>,
) -> Node {
    let choose_pie = move |index| {
        choose_shading(
            index,
            controls.shading,
            controls.viewport_grid,
            controls.status,
        )
    };
    let bodies = catalog_bodies(controls);
    let sections = bodies
        .into_iter()
        .enumerate()
        .map(|(i, body)| {
            catalog_section(
                i,
                body,
                refs[i],
                controls.category,
                controls.filter,
                controls.status,
            )
        })
        .collect::<Vec<_>>();
    let main =
        scroll(column((hero(controls.compact), sections)).style(style().gap(8).padding_right(2)))
            .with_node(|node| node.node_ref(controls.main_scroll))
            .test_id("creative-main")
            .style(style().grow(1).min_width(0).min_height(0))
            .style(
                style()
                    .scrollbar_color(ui::ColorName::HairStrong, rgba(0, 0.0))
                    .scrollbar_width(kivact::style::ScrollbarWidth::Thin),
            );
    column((
        topbar(
            controls.accent,
            controls.compact,
            controls.category,
            controls.sidebar_open,
            controls.search,
            controls.status,
        ),
        toolbar(controls.transport, controls.status),
        catalog_workspace(controls, rects.clone(), main),
        catalog_status(controls.status, controls.transport),
        operator_search(controls.search, controls.status, refs.clone()),
        kivact::ui::toast_region(kivact::interaction::toaster()),
        ui::pie_menu(pie_entries.as_ref().clone(), controls.pie).on_activate(choose_pie),
    ))
    .test_id("creative-catalog")
    .style(style().size(pct(100), pct(100)).min_height(0))
    .style(
        style()
            .background(ui::ColorName::Canvas)
            .color(ui::ColorName::Text)
            .font_family("Geist, 'Noto Sans JP', sans-serif")
            .font_size(12.5)
            .line_height(1.4),
    )
    .on_pointer(move |e| catalog_pointer(e, controls.pointer, controls.pie, &pie_entries))
    .on_key_up(move |e| catalog_key_up(e, controls.pie, choose_pie))
    .on_key(move |e| {
        catalog_key(
            e,
            controls.pie,
            controls.pointer,
            (controls.pie_zone, controls.viewport_ref),
            controls.search,
            controls.status,
            controls.compact,
        )
    })
}

/// Existing application handles shared by catalog chrome and chapter factories.
#[derive(Clone, Copy)]
struct CatalogControls {
    accent: Signal<ui::Accent>,
    // xtask-allow(state-fields): independent: flipped by the density button and Ctrl+Alt+D only; either row density shows with any chapter, sidebar and search overlay state
    compact: Signal<bool>,
    category: Signal<Category>,
    filter: Signal<String>,
    // xtask-allow(state-fields): independent: the chapter chosen in the sidebar list; every chapter is a reachable view in either density and with the sidebar or search overlay in any state
    current: Signal<usize>,
    // xtask-allow(state-fields): independent: the top bar's sidebar toggle; the sidebar hides or shows over any chapter and density, with the search overlay up or down
    sidebar_open: Signal<bool>,
    status: Signal<String>,
    transport: Signal<TransportState>,
    // xtask-allow(state-fields): independent: opened by the search button, F3 and the feedback chapter's button, and closed by the overlay; it covers any chapter, density and sidebar state
    search: Signal<bool>,
    pie: Signal<ui::PieState>,
    pointer: Signal<Point>,
    pie_zone: NodeRef,
    viewport_ref: NodeRef,
    // xtask-allow(state-fields): independent: the viewport shading mode chosen in the viewport toolbar or the pie menu; every mode draws with the viewport grid on or off
    shading: Signal<usize>,
    // xtask-allow(state-fields): independent: the viewport grid flipped from the viewport toolbar or the pie menu; the grid draws over every shading mode
    viewport_grid: Signal<bool>,
    main_scroll: NodeRef,
    main_offset: Value<Point>,
}
impl CatalogControls {
    fn new() -> Self {
        let accent = signal(ui::Accent::Iris);
        // xtask-allow(state-fields): independent: flipped by the density button and Ctrl+Alt+D only; either row density shows with any chapter, sidebar and search overlay state
        let compact = signal(false);
        let category = signal(Category::All);
        let filter = signal(String::new());
        // xtask-allow(state-fields): independent: the chapter chosen in the sidebar list; every chapter is a reachable view in either density and with the sidebar or search overlay in any state
        let current = signal(0_usize);
        // xtask-allow(state-fields): independent: the top bar's sidebar toggle; the sidebar hides or shows over any chapter and density, with the search overlay up or down
        let sidebar_open = signal(true);
        let status = signal("Ready — Space: Play · F3: Search · F12: Render".to_string());
        let transport = signal(TransportState::default());
        super::audio_values::playback(transport);
        // xtask-allow(state-fields): independent: opened by the search button, F3 and the feedback chapter's button, and closed by the overlay; it covers any chapter, density and sidebar state
        let search = signal(false);
        let pie = signal(ui::PieState::default());
        let pointer = signal(Point::new(0.0, 0.0));
        let pie_zone = node_ref();
        let viewport_ref = node_ref();
        // xtask-allow(state-fields): independent: the viewport shading mode chosen in the viewport toolbar or the pie menu; every mode draws with the viewport grid on or off
        let shading = signal(1_usize);
        // xtask-allow(state-fields): independent: the viewport grid flipped from the viewport toolbar or the pie menu; the grid draws over every shading mode
        let viewport_grid = signal(true);
        let main_scroll = node_ref();
        let main_offset = main_scroll.scroll_offset();
        Self {
            accent,
            compact,
            category,
            filter,
            current,
            sidebar_open,
            status,
            transport,
            search,
            pie,
            pointer,
            pie_zone,
            viewport_ref,
            shading,
            viewport_grid,
            main_scroll,
            main_offset,
        }
    }
}

fn catalog_bodies(controls: CatalogControls) -> Vec<kivact::view::View> {
    vec![
        buttons(controls.status).into_view(),
        numeric(controls.transport).into_view(),
        selection().into_view(),
        indicators().into_view(),
        transport_section(controls.transport).into_view(),
        knobs().into_view(),
        meters(controls.transport).into_view(),
        mixer(controls.transport).into_view(),
        properties().into_view(),
        docking().into_view(),
        outliner().into_view(),
        media().into_view(),
        layers(controls.transport).into_view(),
        timeline(controls.transport, controls.status).into_view(),
        animation(controls.transport).into_view(),
        nodes().into_view(),
        grading().into_view(),
        picker().into_view(),
        viewport(
            controls.shading,
            controls.viewport_grid,
            controls.viewport_ref,
            controls.pointer,
        )
        .into_view(),
        menus(
            controls.status,
            controls.pie,
            controls.pointer,
            controls.pie_zone,
            controls.search,
        )
        .into_view(),
        dialogs(controls.status).into_view(),
        jobs(controls.status).into_view(),
    ]
}

fn catalog_workspace(
    controls: CatalogControls,
    rects: std::rc::Rc<Vec<Value<Option<Rect>>>>,
    main: impl IntoView,
) -> Node {
    row((
        sidebar(
            controls.category,
            controls.filter,
            controls.current,
            rects.clone(),
            controls.main_scroll,
            controls.main_offset,
        )
        .hidden(move || !controls.sidebar_open.get()),
        main,
    ))
    .style(move || {
        style()
            .grow(1)
            .basis(0)
            .min_height(0)
            .min_width(0)
            .padding(if controls.compact.get() { 4 } else { 8 })
            .gap(if controls.compact.get() { 4 } else { 8 })
            .align_items(Align::Stretch)
    })
}
