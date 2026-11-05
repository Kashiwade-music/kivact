//! Role: Media pool and asset browser for creative application.
use kivact::state::IntoBinding;

use super::common::{col, keyed_options, line, note};
use kivact::geometry::Point;
use kivact::geometry::Rect;
use kivact::interaction::ListItem;
use kivact::interaction::{
    ListGesture, ListOptions, ListRangeMode, ListRangeOptions, Selection, use_list,
};
use kivact::prelude::{
    IntoView, NodeBuilder, ScrollAxes, for_each, grid, node, rgb, row, scroll, show, signal,
    spacer, text,
};
use kivact::state::IntoValue;
use kivact::style::hsl;
use kivact::style::use_theme;
use kivact::style::{Align, style};
use kivact::style::{Overflow, Track};
use kivact::view::{A11yAction, Handled};
use kivact::view::{ButtonPress, MouseButton, PointerPhase};
use kivact_ui_creative as ui;

pub(super) fn media() -> impl IntoView {
    let query = signal(String::new());
    let mode = signal(Some("tiles".to_owned()));
    let selected = signal(vec![1u64]);
    let sort = signal(Some((1, kivact::interaction::SortOrder::Ascending)));
    let rows = signal(fixtures());
    let tile_size = signal(120.0);
    let widths = signal(vec![
        18.0_f64, 190.0, 86.0, 98.0, 96.0, 62.0, 130.0, 70.0, 84.0, 128.0,
    ]);
    let shown = (move || {
        rows.get()
            .into_iter()
            .filter(|r| {
                r.cells
                    .iter()
                    .any(|s| s.to_lowercase().contains(&query.get().to_lowercase()))
            })
            .collect::<Vec<_>>()
    })
    .into_value();
    let selection_control = use_list(
        move || {
            shown.with(|rows| {
                rows.iter()
                    .map(|row| {
                        ListItem::new(row.id, row.cells.first().cloned().unwrap_or_default())
                    })
                    .collect::<Vec<_>>()
            })
        },
        Selection::Multiple(selected.into_binding()),
        ListOptions {
            range: ListRangeOptions {
                shift: ListRangeMode::Add,
                ..ListRangeOptions::default()
            },
            ..ListOptions::default()
        },
    );
    let pick = move |id: u64, shift: bool, ctrl: bool| {
        selection_control.select(
            id,
            if shift {
                ListGesture::Extend {
                    mode: ListRangeMode::Add,
                }
            } else if ctrl {
                ListGesture::Toggle
            } else {
                ListGesture::Replace
            },
        );
    };
    col((
        media_toolbar(query, mode, selected, tile_size, shown),
        media_table(query, mode, selected, sort, rows, widths),
        media_tiles(mode, selected, tile_size, shown, pick),
    ))
}

fn fixtures() -> Vec<super::common::TextRow> {
    let names = [
        "A001_C003_0912",
        "A001_C007_0912",
        "A002_C001_0913",
        "B001_C012_0913",
        "Drone_Wide_01",
        "Drone_Wide_02",
        "Interview_CamA",
        "Interview_CamB",
        "Plate_BG_v03",
        "Sky_Replace_v01",
        "Smoke_Elements",
        "Lens_Flare_Pack",
        "Music_Bed_Final",
        "VO_Take_07",
        "Room_Tone",
        "Logo_Sting",
        "Shot_010_Comp_v012",
        "Title_Card",
    ];
    let types = [
        "Video",
        "Video",
        "Video",
        "Video",
        "Video",
        "Video",
        "Video",
        "Video",
        "Image Seq",
        "Image Seq",
        "Image Seq",
        "Video",
        "Audio",
        "Audio",
        "Audio",
        "Video",
        "Timeline",
        "Still",
    ];
    let labels = [
        0xb53838, 0xe4d84c, 0xa9cbc7, 0xe5bcc9, 0xa9a9ca, 0xe7c19e, 0xb3c7b3, 0x7c8cff, 0x4aa44c,
        0x8e2c9a, 0xe8920d, 0x7f452a, 0xf46dd6, 0x3ad3e0, 0xa89677, 0x1e401e,
    ];
    let mut seed = 42u32;
    let mut random = move || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        f64::from(seed) / 4294967296.0
    };
    names
        .into_iter()
        .enumerate()
        .map(|(i, name)| fixture_row(i, name, types[i], &labels, &mut random))
        .collect()
}

fn media_tiles(
    mode: kivact::state::Signal<Option<String>>,
    selected: kivact::state::Signal<Vec<u64>>,
    tile_size: kivact::state::Signal<f64>,
    shown: kivact::state::Value<Vec<super::common::TextRow>>,
    pick: impl Fn(u64, bool, bool) + Copy + 'static,
) -> impl IntoView {
    show(
        move || mode.with(|key| key.as_deref() == Some("list")),
        move || {
            let tiles = for_each(
                shown,
                |r| r.id,
                move |item| media_tile(item, selected, pick),
            );
            scroll(grid(tiles).style(move || {
                style()
                    .grid_template_columns(Track::repeat_auto_fill(Track::minmax(
                        tile_size.get() as f64,
                        Track::fr(1.0),
                    )))
                    .gap(10)
                    .padding(12)
            }))
            .style(style().height(360))
        },
    )
}

fn media_table(
    query: kivact::state::Signal<String>,
    mode: kivact::state::Signal<Option<String>>,
    selected: kivact::state::Signal<Vec<u64>>,
    sort: kivact::state::Signal<Option<(usize, kivact::interaction::SortOrder)>>,
    rows: kivact::state::Signal<Vec<super::common::TextRow>>,
    widths: kivact::state::Signal<Vec<f64>>,
) -> impl IntoView {
    show(
        move || mode.with(|key| key.as_deref() == Some("tiles")),
        move || {
            scroll(
                super::common::text_table(
                    &[
                        ("", 0.0),
                        ("Clip Name", 0.0),
                        ("Type", 0.0),
                        ("Duration", 0.0),
                        ("Resolution", 0.0),
                        ("FPS", 0.0),
                        ("Codec", 0.0),
                        ("Audio Ch", 0.0),
                        ("Rating", 0.0),
                        ("Date Created", 0.0),
                    ],
                    rows,
                    selected,
                    sort,
                )
                .filter(query)
                .row_label(|row| row.cells.get(1).cloned().unwrap_or_default())
                .resizable(widths)
                .style(move || style().width(widths.with(|v| v.iter().sum::<f64>()) + 3.0))
                .row_height(30.0_f64)
                .name("Media Pool table")
                .cell_controls(|item, c| {
                    if c == 0 {
                        return node()
                            .style(style().size(8, 8).radius(4))
                            .style(move || {
                                style().background(item.get().color.unwrap_or(rgb(0xa9a9ca)))
                            })
                            .into_view();
                    }
                    if c == 1 {
                        return row((
                            kivact::view::icon("aperture"),
                            text(move || item.get().cells[1].clone()),
                        ))
                        .style(style().gap(5).align_items(Align::Center))
                        .into_view();
                    }
                    text(move || item.get().cells.get(c).cloned().unwrap_or_default())
                        .style(style().font_family("Geist Mono").font_size(11))
                        .style(move || {
                            let t = &use_theme().get();
                            style().color(if c == 8 {
                                t.ext::<ui::Tokens>().accent
                            } else {
                                t.ext::<ui::Tokens>().muted
                            })
                        })
                        .into_view()
                }),
            )
            .axes(ScrollAxes::Both)
            .style(style().height(360))
        },
    )
}

fn media_toolbar(
    query: kivact::state::Signal<String>,
    mode: kivact::state::Signal<Option<String>>,
    selected: kivact::state::Signal<Vec<u64>>,
    tile_size: kivact::state::Signal<f64>,
    shown: kivact::state::Value<Vec<super::common::TextRow>>,
) -> kivact::view::Node {
    line((
        kivact::ui::radio_group(
            "Media view",
            keyed_options([("tiles", "☷"), ("list", "⊞")]),
            mode,
        )
        .orientation(kivact::view::Orientation::Horizontal)
        .variant(kivact::ui::RadioGroupVariant::Segmented)
        .test_id("creative-media-mode"),
        kivact::ui::text_input("Search Media Pool", query)
            .placeholder("Search clips...")
            .style(style().width(200)),
        spacer(),
        ui::value_slider("Size", tile_size)
            .range(80.0..=260.0)
            .step(1.0)
            .precision(0)
            .reset_to(120.0)
            .style(style().width(140)),
        note(move || {
            format!(
                "{} of {} selected",
                selected.with(Vec::len),
                shown.with(Vec::len)
            )
        })
        .style(style().font_family("Geist Mono").font_size(10.5)),
    ))
    .style(style().height(40).padding_x(10))
}

fn media_tile(
    item: kivact::state::Value<super::common::TextRow>,
    selected: kivact::state::Signal<Vec<u64>>,
    pick: impl Fn(u64, bool, bool) + Copy + 'static,
) -> impl IntoView {
    let id = item.get().id;
    let scrub = signal(None::<f64>);
    let preview = media_preview(id, scrub, pick);
    ui::media_tile(
        move || item.get().cells[1].clone(),
        move || format!("{} · {}", item.get().cells[4], item.get().cells[6]),
        preview,
    )
    .with_node(|node| node.selected(move || selected.with(|s| s.contains(&id))))
    .with_node(|node| node.a11y_action(A11yAction::Select, move |_| pick(id, false, false)))
    .on_pointer(move |event| {
        if matches!(
            event.phase(),
            PointerPhase::Down(ButtonPress {
                button: MouseButton::Primary,
                ..
            })
        ) {
            pick(
                id,
                event.modifiers().shift,
                event.modifiers().ctrl || event.modifiers().meta,
            );
            Handled::Yes
        } else {
            Handled::No
        }
    })
}

fn media_preview(
    id: u64,
    scrub: kivact::state::Signal<Option<f64>>,
    pick: impl Fn(u64, bool, bool) + Copy + 'static,
) -> kivact::view::Node {
    node()
        .style(
            style()
                .aspect_ratio(16.0 / 9.0)
                .radius(7)
                .overflow(Overflow::Hidden),
        )
        .draw(move |d, s| {
            let hue = (id * 37) as f64;
            d.fill_rect(s.to_rect(), hsl(hue, 0.35, 0.2));
            for n in 0..12 {
                d.fill_rect(
                    Rect::new(
                        n as f64 * s.width / 12.0,
                        0.0,
                        s.width / 12.0 + 1.0,
                        s.height,
                    ),
                    hsl(hue + n as f64 * 4.0, 0.35, 0.38 - n as f64 * 0.016),
                );
            }
            d.rounded_rect(
                Rect::new(
                    s.width * 0.3,
                    s.height * 0.15,
                    s.width * 0.4,
                    s.height * 0.5,
                ),
                s.width * 0.2,
                hsl(hue, 0.7, 0.7).with_alpha(0.4),
            );
            if let Some(x) = scrub.get() {
                d.line(
                    Point::new(x, 0.0),
                    Point::new(x, s.height),
                    2.0,
                    d.theme().ext::<ui::Tokens>().accent,
                );
            }
        })
        .on_pointer(move |e| {
            match e.phase() {
                PointerPhase::Enter | PointerPhase::Move => scrub.set(Some(e.pos().x)),
                PointerPhase::Down(ButtonPress {
                    button: MouseButton::Primary,
                    ..
                }) => pick(
                    id,
                    e.modifiers().shift,
                    e.modifiers().ctrl || e.modifiers().meta,
                ),
                PointerPhase::Leave => scrub.set(None),
                _ => {}
            };
            Handled::No
        })
}

fn fixture_row(
    i: usize,
    name: &str,
    kind: &str,
    labels: &[u32],
    random: &mut impl FnMut() -> f64,
) -> super::common::TextRow {
    let duration = if kind == "Still" {
        1.0
    } else {
        (48.0 + random() * 1400.0).round()
    };
    let _hue = (random() * 360.0).round();
    let resolution = if kind == "Audio" {
        "—"
    } else {
        ["3840×2160", "4096×2160", "1920×1080", "6144×3240"][(random() * 4.0).floor() as usize]
    };
    let fps = if kind == "Audio" {
        None
    } else {
        Some([23.976_f64, 24.0, 25.0, 29.97][(random() * 4.0).floor() as usize])
    };
    let codecs: &[&str] = match kind {
        "Video" => &["ProRes 4444 XQ", "ARRIRAW", "BRAW 8:1", "H.265 10-bit"],
        "Image Seq" => &["OpenEXR (DWAA)", "DPX 10-bit"],
        "Audio" => &["PCM 24-bit"],
        "Still" => &["PNG 16-bit"],
        _ => &["—"],
    };
    let codec = codecs[(random() * codecs.len() as f64).floor() as usize];
    let channels = if kind == "Image Seq" || kind == "Still" {
        0
    } else if kind == "Audio" {
        2
    } else {
        [2, 4, 8][(random() * 3.0).floor() as usize]
    };
    let rating = (random() * 6.0).floor() as usize;
    let date = format!(
        "2026-09-{:02} {:02}:{:02}",
        10 + (random() * 18.0).floor() as usize,
        8 + (random() * 10.0).floor() as usize,
        (random() * 60.0).floor() as usize
    );
    let color = labels[(random() * labels.len() as f64).floor() as usize];
    let rate = fps.unwrap_or(24.0).round() as u32;
    let frame = duration as u32;
    super::common::TextRow::new(
        i as u64 + 1,
        [
            String::new(),
            name.into(),
            kind.into(),
            format!(
                "{:02}:{:02}:{:02}:{:02}",
                frame / rate / 3600,
                frame / rate / 60 % 60,
                frame / rate % 60,
                frame % rate
            ),
            resolution.into(),
            fps.map_or_else(|| "—".into(), |f| format!("{f:.3}")),
            codec.into(),
            if channels == 0 {
                "—".into()
            } else {
                channels.to_string()
            },
            format!("{}{}", "★".repeat(rating), "☆".repeat(5 - rating)),
            date,
        ],
    )
    .color(rgb(color))
}
