//! Role: Render the catalog application mission model and demo cards.
use kivact::draw::{Shape, linear_gradient};
use kivact::prelude::*;
use kivact::style::{Direction, Position};
use kivact_ui_pop::{self as ui, Hue, Tokens};
// xtask-allow(state-fields): Immutable catalog mission facts; locked, progress and action labels are independent data, not local lifecycle flags.
/// All mission content is public data; fetching, rewards and locking belong to the application.
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
pub(super) fn mission_card(mission: Mission, action: impl Into<Callback<()>>) -> impl Component {
    kivact::ui::card((
        mission_banner(&mission),
        mission_details(&mission),
        mission_footer(&mission, action.into()),
    ))
    .style(
        style()
            .padding(0)
            .clip(true)
            .min_height(348)
            .hover(style().shadow(kivact::draw::Shadow::new(0, 4, 6, ColorToken::Shadow))),
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
        text(label).style(style().grow(1)),
        text(value).style(style().weight(600)),
    ))
    .style(style().font_size(12).color(ColorToken::TextMuted).gap(8))
}

fn mission_banner(mission: &Mission) -> Node {
    let hue = mission.hue;
    let locked = mission.locked;
    node()
        .style(move || {
            let t = &use_theme().get();
            let p = t.ext::<Tokens>();
            style()
                .height(104)
                .background(linear_gradient(120).colors([
                    if hue == Hue::Gray {
                        p.navy
                    } else if hue == Hue::Blue {
                        p.blue_deep
                    } else {
                        p.accent(hue)
                    },
                    if hue == Hue::Pink {
                        kivact::style::rgb(0xffb3cf)
                    } else if hue == Hue::Gray {
                        kivact::style::rgb(0x3b4f7a)
                    } else {
                        p.cyan
                    },
                ]))
                .color(Color::WHITE)
                .filter(kivact::draw::Filter::new().saturate(if locked { 0.6 } else { 1.0 }))
                .direction(Direction::Row)
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
                    .italic(true)
                    .weight(800)
                    .font_size(13)
                    .color(Color::WHITE),
            ),
        )
        .child(icon(mission.banner_icon).style(style().size(40, 40)))
        .child(
            kivact::ui::badge(mission.rank.clone())
                .variant(ui::BadgeHue::new(mission.rank_tone))
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
        text(mission.title.clone()).style(style().font_size(15).weight(600)),
        text(mission.description.clone()).style(
            style()
                .font_size(13)
                .color(ColorToken::TextMuted)
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
                        .radius(6)
                        .background(ColorToken::SurfaceAlt),
                )
            })
            .collect::<Vec<_>>())
        .style(style().gap(6).wrap(true).margin_top(8)),
    ))
    .style(
        style()
            .padding_x(20)
            .padding_top(8)
            .padding_bottom(16)
            .gap(6)
            .grow(1)
            .filter(kivact::draw::Filter::new().saturate(if locked { 0.6 } else { 1.0 })),
    )
}

fn mission_footer(mission: &Mission, action: Callback<()>) -> Node {
    let locked = mission.locked;
    let action_label = or_message(mission.action_label.clone(), "mission.open", "Open");
    row((
        row((
            icon(if locked { "lock" } else { "clock" })
                .style(style().size(14, 14).color(ColorToken::TextMuted)),
            text(mission.footer.clone()).style(style().font_size(12).color(ColorToken::TextMuted)),
        ))
        .style(style().gap(4).grow(1).align_items(Align::Center)),
        button(action_label)
            .variant(mission.action_tone)
            .variant(kivact_ui_pop::Slant)
            .size(kivact::ui::ControlSize::Small)
            .disabled(locked)
            .on_press(action),
    ))
    .style(
        style()
            .padding_x(20)
            .padding_top(12)
            .padding_bottom(16)
            .border_top(1, ColorToken::Border)
            .align_items(Align::Center),
    )
}
