//! Role: The Data Display page of the game catalog: KPI tiles, status badges, progress, avatars, activity and property lists.

use crate::support::common::{cards, display_family, line, list, specimen};
use crate::support::dialogs::overlay_layer;
use crate::support::feature::feature_card;
use crate::support::state::{Demo, Overlay};
use catalog_shell::{Example, Page};
use kivact::platform;
use kivact::prelude::*;
use kivact::view::{Node, rich_text, span};
use kivact_ui_game as g;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new("data-display", "Data Display", "Console", include_str!("data_display.rs"))
        .description("KPI tiles, status badges, progress, avatars, activity and property lists")
        .example(Example::new("tiles", "Tiles, badges and lists", "KPI tiles, status badges, progress bars and gauges, avatars, an activity feed and a property list.", demo))
}

// example: tiles
fn demo() -> impl IntoView {
    let scene = Demo::new();
    column((display(scene), overlay_layer(scene)))
}
// end example

fn kpi_heading(icon: &'static str, label: &str) -> Node {
    row((
        kivact::view::icon(icon).style(style().size(13, 13)),
        text(label.to_owned()).style(style().font_size(11).font_weight(600)),
    ))
    .style(
        style()
            .gap(6)
            .align_items(Align::Center)
            .color(g::color::TEXT_3),
    )
}

pub(crate) fn display(scene: Demo) -> Node {
    list((
        kpi_tiles(),
        cards(
            (
                status_badges(),
                progress_bars(),
                progress_gauges(),
                avatars(),
                activity_feed(),
                property_list(),
                feature_cards(scene),
            ),
            330.,
        ),
    ))
}

fn watermark(number: &str) -> kivact::view::Node {
    // A decoration that bleeds out of its card on purpose; the layout check
    // of the catalog ignores it by this id.
    text(number.to_owned())
        .style(
            style()
                .position(Position::Absolute)
                .right(-4)
                .top(-10)
                .font_family(display_family())
                .font_size(64)
                .font_weight(800)
                .font_style(kivact::style::FontStyle::Italic)
                .opacity(0.035),
        )
        .test_id(format!("deco-number-{number}"))
}

pub(crate) fn progress_row(label: &str, value: f64, tone: kivact::interaction::Tone) -> Node {
    row((
        text(label.to_owned()).style(style().width(96).font_size(12)),
        kivact::ui::progress(Some(value / 100.0))
            .name(label.to_owned())
            .variant(tone)
            .style(style().flex_grow(1)),
        g::mono(format!("{value:.0}%"))
            .style(style().width(44).text_align(kivact::style::TextAlign::End)),
    ))
    .style(style().align_items(Align::Center).gap(10))
}

fn format_count(value: f64, decimals: usize) -> String {
    let plain = format!("{:.*}", decimals.min(9), value);
    let (whole, fraction) = plain
        .split_once('.')
        .map_or((plain.as_str(), None), |(a, b)| (a, Some(b)));
    let mut grouped = String::new();
    for (index, character) in whole.chars().enumerate() {
        if index > 0
            && (whole.len() - index).is_multiple_of(3)
            && character.is_ascii_digit()
            && whole.as_bytes()[index - 1].is_ascii_digit()
        {
            grouped.push(',');
        }
        grouped.push(character);
    }
    if let Some(fraction) = fraction {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    grouped
}

fn kpi_tiles() -> impl kivact::view::Element {
    specimen(
        "DSP-01",
        "KPI tiles",
        cards(
            (
                completed_kpi(),
                uptime_kpi(),
                duration_kpi(),
                incident_kpi(),
            ),
            220.,
        )
        .style(style().gap(12)),
    )
}

fn status_badges() -> impl kivact::view::Element {
    specimen(
        "DSP-02",
        "Badges / status",
        line((
            kivact::ui::badge("RUNNING").tone(kivact::interaction::Tone::Info),
            kivact::ui::badge("PASSED").tone(kivact::interaction::Tone::Success),
            kivact::ui::badge("PENDING").tone(kivact::interaction::Tone::Warning),
            kivact::ui::badge("FAILED").tone(kivact::interaction::Tone::Danger),
            kivact::ui::badge("ARCHIVED")
                .tone(kivact::interaction::Tone::Neutral)
                .variant(g::BadgeVariant::Plain),
            kivact::ui::badge("V2.4.0")
                .tone(kivact::interaction::Tone::Info)
                .variant(g::BadgeVariant::Solid),
            kivact::ui::badge("BETA")
                .tone(kivact::interaction::Tone::Accent)
                .variant(g::BadgeVariant::Plain)
                .style(style().color(rgb(0x13233a))),
            kivact::ui::badge("PROD")
                .tone(kivact::interaction::Tone::Info)
                .variant(g::BadgeVariant::Outline),
            row((
                text("Inbox"),
                kivact::ui::badge("24")
                    .tone(kivact::interaction::Tone::Danger)
                    .variant(g::BadgeVariant::Solid)
                    .style(style().padding_x(5)),
            ))
            .style(style().gap(6)),
        )),
    )
}

fn progress_bars() -> impl kivact::view::Element {
    specimen(
        "DSP-03",
        "Progress",
        list((
            row((
                text("Sync").style(style().width(96).font_size(12)),
                kivact::ui::progress(Some(0.68))
                    .name("Sync")
                    .variant(kivact::interaction::Tone::Info)
                    .variant(kivact::ui::Striped)
                    .moving(true)
                    .style(style().flex_grow(1)),
                g::mono("68%").style(style().width(44).text_align(TextAlign::End)),
            ))
            .style(style().gap(10).align_items(Align::Center)),
            progress_row("Verify", 100., kivact::interaction::Tone::Success),
            progress_row("Storage", 86., kivact::interaction::Tone::Warning),
            row((
                text("Analyzing").style(style().width(96).font_size(12)),
                kivact::ui::progress(None::<f64>)
                    .name("Analyzing")
                    .variant(kivact::interaction::Tone::Info)
                    .size(kivact::ui::ControlSize::Small)
                    .style(style().flex_grow(1)),
                g::mono("—").style(style().width(44).text_align(TextAlign::End)),
            ))
            .style(style().gap(10).align_items(Align::Center)),
        )),
    )
}

fn progress_gauges() -> impl kivact::view::Element {
    specimen(
        "DSP-04",
        "Gauge / ring",
        row((
            list((
                line((
                    capacity_gauge("Capacity", 7, kivact::interaction::Tone::Info),
                    text("7/10").style(
                        style()
                            .font_family(display_family())
                            .font_size(22)
                            .font_style(kivact::style::FontStyle::Italic)
                            .font_weight(700),
                    ),
                )),
                line((
                    capacity_gauge("Low capacity", 2, kivact::interaction::Tone::Danger),
                    text("2/10").style(
                        style()
                            .font_family(display_family())
                            .font_size(22)
                            .font_style(kivact::style::FontStyle::Italic)
                            .font_weight(700),
                    ),
                )),
                g::mono("CAPACITY UNITS").style(style().color(color::TEXT_MUTED)),
            )),
            {
                let value: kivact::state::Value<f64> = kivact::state::IntoValue::into_value(76.);
                // The label lies over the ring in a stack, not as a child of the
                // #[component] root (its root takes no children).
                stack((
                    kivact::ui::progress(move || Some(value.get() / 100.0))
                        .name("SLA")
                        .variant(kivact::ui::ProgressVariant::Ring)
                        .variant(kivact::ui::ProgressMotion {
                            determinate_ms: 900,
                            determinate_easing: kivact::motion::Easing::CubicBezier(
                                0.2, 0.7, 0.2, 1.,
                            ),
                            ..Default::default()
                        })
                        .variant(kivact::interaction::Tone::Info),
                    row((g::mono("SLA").style(style().font_size(9).color(color::TEXT_MUTED)),))
                        .style(
                            style()
                                .justify_content(Justify::Center)
                                .align_items(Align::Center),
                        ),
                ))
            },
        ))
        .style(
            style()
                .justify_content(Justify::SpaceBetween)
                .width(pct(100))
                .align_items(Align::Center),
        ),
    )
}

fn avatars() -> impl kivact::view::Element {
    specimen(
        "DSP-05",
        "Avatars",
        list((
            line((
                kivact::ui::avatar("KM")
                    .fallback_content(|| text("KM"))
                    .variant(g::AvatarVariant {
                        side: 40.,
                        color: rgb(0x1283f0),
                    })
                    .status(Some(kivact::interaction::Tone::Success)),
                kivact::ui::avatar("SA")
                    .fallback_content(|| text("SA"))
                    .variant(g::AvatarVariant {
                        side: 32.,
                        color: rgb(0x0c1f36),
                    })
                    .status(Some(kivact::interaction::Tone::Warning)),
                kivact::ui::avatar("TN")
                    .fallback_content(|| text("TN"))
                    .variant(g::AvatarVariant {
                        side: 24.,
                        color: rgb(0x12a06e),
                    })
                    .status(Some(kivact::interaction::Tone::Neutral)),
                kivact::ui::avatar_group(
                    vec![
                        ("HI", rgb(0x1283f0)),
                        ("YU", rgb(0xd98300)),
                        ("RK", rgb(0xe0364d)),
                    ]
                    .into_iter()
                    .map(|(name, color)| {
                        kivact::ui::avatar(name)
                            .fallback_content(move || text(name))
                            .variant(g::AvatarVariant { side: 32., color })
                            .style(style().border(2, color::SURFACE))
                    })
                    .collect::<Vec<_>>(),
                )
                .total(8usize)
                .max(3usize),
            )),
            row((
                kivact::ui::avatar("MS")
                    .fallback_content(|| text("MS"))
                    .variant(g::AvatarVariant {
                        side: 32.,
                        color: rgb(0x0a62c2),
                    })
                    .status(None),
                column((
                    text("Sou Mashiba").style(style().font_weight(700)),
                    g::mono("ADMIN · Maintenance")
                        .style(style().font_size(10).color(color::TEXT_MUTED)),
                )),
            ))
            .style(style().gap(10).align_items(Align::Center)),
        )),
    )
}

fn activity_feed() -> impl kivact::view::Element {
    specimen(
        "DSP-06",
        "Activity",
        g::activity(vec![
            g::ActivityEntry::new(
                "09:40 · NOW",
                rich_text(vec![
                    span("Deploy #418").font_weight(700),
                    span(" started in production"),
                ]),
                kivact::interaction::Tone::Info,
            ),
            g::ActivityEntry::new(
                "09:12",
                rich_text(vec![
                    span("Verify").font_weight(700),
                    span(" passed all 214 checks"),
                ]),
                kivact::interaction::Tone::Success,
            ),
            g::ActivityEntry::new(
                "08:55",
                rich_text(vec![
                    span("worker-2").font_weight(700),
                    span(" timed out (auto-recovered)"),
                ]),
                kivact::interaction::Tone::Danger,
            ),
            g::ActivityEntry::new(
                "08:30",
                rich_text(vec![
                    span("Mashiba").font_weight(700),
                    span(" approved OPS-2041"),
                ]),
                kivact::interaction::Tone::Neutral,
            ),
        ]),
    )
}

fn property_list() -> impl kivact::view::Element {
    specimen(
        "DSP-07",
        "Property list",
        kivact::ui::description_list(
            (vec![
                (
                    "ID".into(),
                    row((
                        g::mono("ops_01HZX4K9"),
                        spacer(),
                        g::tool_button("Copy ID").on_press(|| {
                            spawn(platform::copy_text("ops_01HZX4K9"));
                        }),
                    ))
                    .style(style().align_items(Align::Center))
                    .into_view(),
                ),
                (
                    "STATUS".into(),
                    kivact::ui::badge("ACTIVE")
                        .tone(kivact::interaction::Tone::Success)
                        .into_view(),
                ),
                ("OWNER".into(), text("Maintenance / Mashiba").into_view()),
                ("CREATED".into(), g::mono("2026-09-28 14:02").into_view()),
            ])
            .into_iter()
            .map(|(term, detail): (String, _)| kivact::ui::description(term, detail))
            .collect::<Vec<_>>(),
        ),
    )
}

fn feature_cards(scene: Demo) -> impl kivact::view::Element {
    specimen(
        "DSP-08",
        "Feature card",
        feature_card(
            "Continuous monitoring",
            "Collects the sensors of all sites every 30 seconds and notifies on-call when a threshold is exceeded.",
            kivact::ui::button("Edit rules")
                .variant(kivact::ui::Slant)
                .size(kivact::ui::ControlSize::Small)
                .variant(kivact::ui::ButtonVariant::Secondary)
                .on_press(move || scene.show(Overlay::Drawer)),
        )
        .child(
            kivact::ui::badge("ON")
                .tone(kivact::interaction::Tone::Success)
                .variant(g::BadgeVariant::Plain)
                .style(style().position(Position::Absolute).right(16).top(16)),
        ),
    )
}

fn completed_kpi() -> Node {
    node()
        .child(
            g::kpi("Completed operations", 1284.)
                .change("↗ 12.4%")
                .tone(kivact::interaction::Tone::Info)
                .heading(kpi_heading("target", "Completed operations"))
                .trend(vec![0.24, 0.36, 0.3, 0.66, 0.56, 0.75, 0.58, 0.88])
                .format(move |value| format_count(value, 0)),
        )
        .child(watermark("01"))
}

fn uptime_kpi() -> Node {
    node()
        .child(
            g::kpi("Uptime", 99.97)
                .unit("%")
                .change("↗ 0.02pt")
                .tone(kivact::interaction::Tone::Success)
                .heading(kpi_heading("activity", "Uptime"))
                .trend(vec![0.24, 0.36, 0.3, 0.66, 0.56, 0.75, 0.58, 0.88])
                .format(move |value| format_count(value, 2)),
        )
        .child(watermark("02"))
}

fn duration_kpi() -> Node {
    node()
        .child(
            g::kpi("Average processing time", 4.2)
                .unit("h")
                .change("↘ +0.6h")
                .tone(kivact::interaction::Tone::Warning)
                .heading(kpi_heading("clock", "Average processing time"))
                .trend(vec![0.24, 0.36, 0.3, 0.66, 0.56, 0.75, 0.58, 0.88])
                .format(move |value| format_count(value, 1)),
        )
        .child(watermark("03"))
}

fn incident_kpi() -> Node {
    node()
        .child(
            g::kpi("Open incidents", 7.)
                .change("↗ +2")
                .tone(kivact::interaction::Tone::Danger)
                .heading(kpi_heading("alert-octagon", "Open incidents"))
                .trend(vec![0.24, 0.36, 0.3, 0.66, 0.56, 0.75, 0.58, 0.88])
                .format(move |value| format_count(value, 0)),
        )
        .child(watermark("04"))
}

fn capacity_gauge(
    label: &'static str,
    count: usize,
    tone: kivact::interaction::Tone,
) -> impl IntoView {
    let total: kivact::state::Value<usize> = kivact::state::IntoValue::into_value(10);
    let value: kivact::state::Value<usize> = kivact::state::IntoValue::into_value(count);
    kivact::ui::progress(move || {
        Some(if total.get() == 0 {
            0.0
        } else {
            value.get() as f64 / total.get() as f64
        })
    })
    .name(label)
    .variant(kivact::ui::ProgressVariant::Gauge)
    .variant(g::GaugeSegments(total.get()))
    .variant(tone)
}
