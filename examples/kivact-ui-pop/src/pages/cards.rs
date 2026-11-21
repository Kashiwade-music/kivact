//! Role: The Mission Cards page.

use super::common::*;
use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact::state::{IntoHandler, IntoValue};
use kivact::style::{FlexDirection, Shape, linear_gradient};
use kivact::view::Node;
use kivact_ui_pop as ui;
use kivact_ui_pop::Hue;
/// The page of the Mission Cards components.
pub(crate) fn page() -> Page {
    Page::new(
        "cards",
        "Mission Cards",
        "Components",
        include_str!("cards.rs"),
    )
    .description("Mission cards that summarize a task, its progress and its next action.")
    .example(Example::new(
        "cards",
        "Mission cards",
        "A mission card shows its state, progress and action together.",
        cards_demo,
    ))
}

fn cards_demo() -> impl IntoView {
    // example: cards
    let notify = notify();
    let items = [
        super::cards::Mission {
            code: "AREA 07".into(),
            title: "Outer district patrol".into(),
            description: "Suspicious drone sighting. Check the safety of the area.".into(),
            hue: ui::Hue::Blue,
            rank: "NORMAL".into(),
            rank_tone: ui::Hue::Blue,
            banner_icon: "building-community",
            progress: 66.0,
            progress_label: None,
            progress_value: "2 / 3".into(),
            rewards: vec!["×12,000".into(), "×2".into()],
            reward_icons: vec!["coin", "ticket"],
            footer: "2 days 4 hours left".into(),
            action_label: Some("Deploy".into()),
            action_tone: kivact_ui_pop::ButtonTone::Accent,
            locked: false,
        },
        super::cards::Mission {
            code: "EVENT".into(),
            title: "Autumn moon-viewing festival".into(),
            description: "Get the event-only story and rewards.".into(),
            hue: ui::Hue::Pink,
            rank: "Limited".into(),
            rank_tone: ui::Hue::Yellow,
            banner_icon: "gift",
            progress: 43.0,
            progress_label: Some("Event points".into()),
            progress_value: "4,280 pt".into(),
            rewards: vec!["×1,200".into(), "Furniture set".into()],
            reward_icons: vec!["sparkles", "gift"],
            footer: "Until 10/17".into(),
            action_label: Some("Join".into()),
            action_tone: kivact_ui_pop::ButtonTone::Pink,
            locked: false,
        },
        super::cards::Mission {
            code: "RAID".into(),
            title: "All-out battle: the clockwork serpent".into(),
            description: "A hard endgame content for all units. Face it with your strongest team."
                .into(),
            hue: ui::Hue::Gray,
            rank: "INSANE".into(),
            rank_tone: ui::Hue::Red,
            banner_icon: "crosshair",
            progress: 90.0,
            progress_label: Some("Unlock condition".into()),
            progress_value: "Lv.80".into(),
            rewards: vec!["Title".into(), "×200,000".into()],
            reward_icons: vec!["star", "coin"],
            footer: "8 more levels".into(),
            action_label: Some("Locked".into()),
            action_tone: kivact_ui_pop::ButtonTone::Soft,
            locked: true,
        },
    ];
    section(
        "12",
        "Mission Cards",
        "Mission cards: requests, events and rewards",
        cards_grid(
            items
                .into_iter()
                .map(|mission| {
                    third(super::cards::mission_card(mission, move || {
                        notify.call((
                            ui::Hue::Yellow,
                            "Reward earned!".into(),
                            "Credits ×12,000 / activity report ×2".into(),
                        ))
                    }))
                })
                .collect::<Vec<_>>(),
        ),
    )
    // end example
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Mission {
    pub code: String,
    pub title: String,
    pub description: String,
    pub hue: Hue,
    pub rank: String,
    pub rank_tone: Hue,
    pub banner_icon: &'static str,
    pub progress: f64,
    /// The progress caption; `None` shows the message `mission.progress`.
    pub progress_label: Option<String>,
    pub progress_value: String,
    pub rewards: Vec<String>,
    pub reward_icons: Vec<&'static str>,
    pub footer: String,
    /// The action button's label; `None` shows the message `mission.open`.
    pub action_label: Option<String>,
    pub action_tone: kivact_ui_pop::ButtonTone,
    pub locked: bool,
}
#[track_caller]
pub(super) fn mission_card<M>(mission: Mission, action: impl IntoHandler<(), M>) -> impl Element {
    kivact::ui::card((
        mission_banner(&mission),
        mission_details(&mission),
        mission_footer(
            &mission,
            kivact::state::Callback::from_handler(kivact::state::IntoHandler::into_handler(action)),
        ),
    ))
    .style(
        style()
            .padding(0)
            .overflow(kivact::style::Overflow::Hidden)
            .min_height(348)
            .hover(style().shadow(kivact::style::Shadow::new(0, 4, 6, color::SHADOW))),
    )
}

/// `text` as a constant, or the message of `key` when there is none.
fn or_message(text: Option<String>, key: &'static str, default: &'static str) -> Value<String> {
    match text {
        Some(text) => text.into_value(),
        None => message(key, default),
    }
}

fn progress_labels(label: Value<String>, value: String) -> Node {
    row((
        text(label).style(style().flex_grow(1)),
        text(value).style(style().font_weight(600)),
    ))
    .style(style().font_size(12).color(color::TEXT_MUTED).gap(8))
}

fn mission_banner(mission: &Mission) -> Node {
    let hue = mission.hue;
    let locked = mission.locked;
    node()
        .style(move || {
            style()
                .height(104)
                .background(linear_gradient(120).colors([
                    if hue == Hue::Gray {
                        rgb(0x1d2b4a)
                    } else if hue == Hue::Blue {
                        rgb(0x1a66a8)
                    } else {
                        hue.accent()
                    },
                    if hue == Hue::Pink {
                        kivact::style::rgb(0xffb3cf)
                    } else if hue == Hue::Gray {
                        kivact::style::rgb(0x3b4f7a)
                    } else {
                        rgb(0x5cbde8)
                    },
                ]))
                .color(Color::WHITE)
                .filter(kivact::style::Filter::new().saturate(if locked { 0.6 } else { 1.0 }))
                .flex_direction(FlexDirection::Row)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .clip_path(
                    Shape::polygon()
                        .point(0, 0)
                        .point(pct(100), 0)
                        .point(pct(100), pct(82))
                        .point(0, pct(100)),
                )
        })
        .draw(|draw, size| {
            let stripe = kivact::style::rgba(0xffffff, 0.1);
            for i in 0..((size.width / 16.0) as usize + 10) {
                let x = i as f64 * 16.0 - 80.0;
                draw.line(
                    kivact::geometry::Point::new(x, size.height),
                    kivact::geometry::Point::new(x + size.height * 0.47, 0.0),
                    2.0,
                    stripe,
                );
            }
            let center = kivact::geometry::Point::new(size.width - 40.0, 20.0);
            let halo = kivact::draw::Path::new()
                .arc(center, 70, 0, 360, false)
                .close();
            draw.stroke_path(
                &halo,
                &kivact::draw::Stroke::new(10),
                kivact::style::rgba(0xffffff, 0.16),
            );
        })
        .child(
            text(mission.code.clone()).style(
                style()
                    .position(Position::Absolute)
                    .left(16)
                    .top(14)
                    .font_family("Barlow, sans-serif")
                    .font_style(kivact::style::FontStyle::Italic)
                    .font_weight(800)
                    .font_size(13)
                    .color(Color::WHITE),
            ),
        )
        .child(icon(mission.banner_icon).style(style().size(40, 40)))
        .child(
            kivact::ui::badge(mission.rank.clone())
                .variant(mission.rank_tone)
                .variant(ui::BadgeVariant::Slant)
                .style(style().position(Position::Absolute).right(16).top(12)),
        )
}

fn mission_details(mission: &Mission) -> Node {
    let hue = mission.hue;
    let locked = mission.locked;
    let progress_label = or_message(
        mission.progress_label.clone(),
        "mission.progress",
        "Progress",
    );
    column((
        text(mission.title.clone()).style(style().font_size(15).font_weight(600)),
        text(mission.description.clone()).style(
            style()
                .font_size(13)
                .color(color::TEXT_MUTED)
                .margin_bottom(6),
        ),
        progress_labels(progress_label, mission.progress_value.clone()),
        kivact::ui::progress(Some(mission.progress / 100.0))
            .name(progress_label)
            .variant(if hue == Hue::Gray { Hue::Blue } else { hue })
            .style(style().height(8)),
        row(mission
            .rewards
            .clone()
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                row((
                    icon(mission.reward_icons.get(i).copied().unwrap_or("gift"))
                        .style(style().size(14, 14).color(kivact::style::rgb(0xf2bf0a))),
                    text(r),
                ))
                .style(
                    style()
                        .padding_x(8)
                        .height(24)
                        .font_size(12)
                        .gap(4)
                        .align_items(Align::Center)
                        .border_radius(6)
                        .background(color::SURFACE_ALT),
                )
            })
            .collect::<Vec<_>>())
        .style(style().gap(6).flex_wrap(true).margin_top(8)),
    ))
    .style(
        style()
            .padding_x(20)
            .padding_top(8)
            .padding_bottom(16)
            .gap(6)
            .flex_grow(1)
            .filter(kivact::style::Filter::new().saturate(if locked { 0.6 } else { 1.0 })),
    )
}

fn mission_footer(mission: &Mission, action: Callback<()>) -> Node {
    let locked = mission.locked;
    let action_label = or_message(mission.action_label.clone(), "mission.open", "Open");
    row((
        row((
            icon(if locked { "lock" } else { "clock" })
                .style(style().size(14, 14).color(color::TEXT_MUTED)),
            text(mission.footer.clone()).style(style().font_size(12).color(color::TEXT_MUTED)),
        ))
        .style(style().gap(4).flex_grow(1).align_items(Align::Center)),
        button(action_label)
            .variant(mission.action_tone)
            .variant(kivact::ui::Slant)
            .size(kivact::ui::ControlSize::Small)
            .disabled(locked)
            .on_press(action),
    ))
    .style(
        style()
            .padding_x(20)
            .padding_top(12)
            .padding_bottom(16)
            .border_top(1, color::BORDER)
            .align_items(Align::Center),
    )
}
