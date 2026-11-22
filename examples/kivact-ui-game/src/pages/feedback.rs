//! Role: The Feedback page of the game catalog: alerts, toasts, loading states.
//! Empty states.

use crate::support::common::{cards, line, list, specimen, wide};
use crate::support::dialogs::overlay_layer;
use crate::support::state::{Demo, Overlay};
use catalog_shell::{Example, Page};
use kivact::interaction::{Tone, use_toaster};
use kivact::prelude::*;
use kivact::view::{Node, rich_text, span};
use kivact_ui_game as g;
use std::time::Duration;

/// The page: its one demo, with its explanation.
pub(crate) fn page() -> Page {
    Page::new(
        "feedback",
        "Feedback",
        "Console",
        include_str!("feedback.rs"),
    )
    .description("Alerts, toasts, loading states and empty states")
    .example(Example::new(
        "feedback",
        "Alerts, toasts and states",
        "Banners, inline alerts, toasts, loading skeletons and an empty state.",
        demo,
    ))
}

// example: feedback
fn demo() -> impl IntoView {
    let scene = Demo::new();
    column((feedback(scene), overlay_layer(scene)))
}
// end example

// xtask-allow(state): independent: info, success, warning and danger are the open bindings of four separate inline alerts, each cleared only by its own dismiss button and never reopened
pub(crate) fn feedback(scene: Demo) -> Node {
    let info = signal(true);
    let success = signal(true);
    let warning = signal(true);
    let danger = signal(true);
    cards(
        (
            wide(system_banners()),
            inline_alerts(info, success, warning, danger),
            toast_actions(),
            loading_skeletons(),
            empty_content(scene),
        ),
        460.,
    )
}

fn system_banners() -> impl kivact::view::Element {
    specimen(
        "FBK-01",
        "System banners",
        kivact::ui::alert("NOTICE")
            .variant(kivact::ui::AlertVariant::Banner)
            .tone(kivact::interaction::Tone::Warning)
            .actions(
                row((
                    g::mono("T-41:20:00").style(style().color(rgb(0x9fb7d2))),
                    kivact::ui::button("Details")
                        .variant(kivact::ui::Slant)
                        .test_id("game-banner-details")
                        .size(kivact::ui::ControlSize::Small)
                        .style(style().background(rgb(0xffc928)).color(rgb(0x1a1a1a)))
                        .on_press(move || {
                            use_toaster().push(
                                Toast::new("Planned maintenance")
                                    .body("10/05 02:00–04:00 JST · the API becomes read-only.")
                                    .tone(Tone::Warning)
                                    .duration(Some(Duration::from_secs(6))),
                            );
                        }),
                ))
                .style(style().gap(14).align_items(Align::Center)),
            )
            .with_node(|node| {
                node.role(Role::Status)
                    .live_region(kivact::view::LiveRegion::Polite)
            })
            .content(move || {
                rich_text(vec![
                    span("Planned maintenance:")
                        .font_weight(700)
                        .color(rgb(0xffc928)),
                    span(" The API becomes read-only 10/05 (Sun) 02:00–04:00 JST."),
                ])
            }),
    )
}

fn inline_alerts(
    info: kivact::state::Signal<bool>,
    success: kivact::state::Signal<bool>,
    warning: kivact::state::Signal<bool>,
    danger: kivact::state::Signal<bool>,
) -> impl kivact::view::Element {
    specimen(
        "FBK-02",
        "Inline alerts",
        list((
            kivact::view::show(info, move || {
                kivact::ui::alert("A new version is available")
                    .description("Audit log export was added in v2.5.0.")
                    .tone(kivact::interaction::Tone::Info)
                    .icon(kivact::view::icon("info-circle"))
                    .open(info)
                    .dismissible(true)
                    .dismiss_label(kivact::i18n::message("game.alert.close", "Close alert"))
                    .dismiss_content(text("×"))
                    .with_node(|node| {
                        node.role(Role::Status)
                            .live_region(kivact::view::LiveRegion::Polite)
                    })
            }),
            kivact::view::show(success, move || {
                kivact::ui::alert("Backup complete")
                    .description("2.4 GB · 3 min 12 s")
                    .tone(kivact::interaction::Tone::Success)
                    .icon(kivact::view::icon("circle-check"))
                    .open(success)
                    .dismissible(true)
                    .dismiss_label(kivact::i18n::message("game.alert.close", "Close alert"))
                    .dismiss_content(text("×"))
                    .with_node(|node| {
                        node.role(Role::Status)
                            .live_region(kivact::view::LiveRegion::Polite)
                    })
            }),
            kivact::view::show(warning, move || {
                kivact::ui::alert("The certificate expires soon")
                    .description("*.ops.example.internal — 9 days left")
                    .tone(kivact::interaction::Tone::Warning)
                    .icon(kivact::view::icon("alert-triangle"))
                    .actions({
                        row((
                            kivact::ui::button("Update")
                                .variant(kivact::ui::Slant)
                                .size(kivact::ui::ControlSize::Small)
                                .variant(kivact::ui::ButtonVariant::Ghost)
                                .on_press(move || {
                                    use_toaster().push(
                                        Toast::new("Update started")
                                            .body("Certificate renewal started.")
                                            .tone(Tone::Info)
                                            .duration(Some(Duration::from_secs(4))),
                                    );
                                }),
                            kivact::ui::button("Later")
                                .variant(kivact::ui::Slant)
                                .size(kivact::ui::ControlSize::Small)
                                .variant(kivact::ui::ButtonVariant::Ghost),
                        ))
                        .style(style().gap(10).margin_top(6))
                    })
                    .open(warning)
                    .dismissible(true)
                    .dismiss_label(kivact::i18n::message("game.alert.close", "Close alert"))
                    .dismiss_content(text("×"))
            }),
            kivact::view::show(danger, move || {
                kivact::ui::alert("Payment link failed")
                    .description("ERR_UPSTREAM_502 · stopped after 3 retries")
                    .tone(kivact::interaction::Tone::Danger)
                    .icon(kivact::view::icon("alert-octagon"))
                    .open(danger)
                    .dismissible(true)
                    .dismiss_label(kivact::i18n::message("game.alert.close", "Close alert"))
                    .dismiss_content(text("×"))
            }),
        )),
    )
}

fn toast_actions() -> impl kivact::view::Element {
    specimen(
        "FBK-03",
        "Toast",
        list((
            line(
                [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger]
                    .into_iter()
                    .map(|tone| {
                        let label = match tone {
                            Tone::Success => "Success",
                            Tone::Warning => "Warning",
                            Tone::Danger => "Error",
                            Tone::Info | Tone::Neutral | Tone::Accent => "Info",
                        };
                        kivact::ui::button(label)
                            .variant(kivact::ui::Slant)
                            .variant(kivact::ui::ButtonVariant::Secondary)
                            .size(kivact::ui::ControlSize::Small)
                            .on_press(move || {
                                use_toaster().push(
                                    Toast::new(label)
                                        .body("Hovering pauses the auto-dismiss timer.")
                                        .tone(tone)
                                        .duration(Some(Duration::from_millis(4200))),
                                );
                            })
                    })
                    .collect::<Vec<_>>(),
            ),
            text("Hovering pauses the auto-dismiss timer. Added to the aria-live=polite region.")
                .style(style().font_size(11.5).color(color::TEXT_MUTED)),
        )),
    )
}

fn loading_skeletons() -> impl kivact::view::Element {
    specimen(
        "FBK-04",
        "Skeleton",
        list((
            kivact::ui::skeleton()
                .variant(g::SkeletonVariant::Rows)
                .lines(2)
                .with_node(|node| node.role(Role::Status))
                .name(kivact::i18n::message("spinner.loading", "Loading")),
            node().style(
                style()
                    .height(64)
                    .width(pct(100))
                    .background(color::SURFACE_ALT),
            ),
        )),
    )
}

fn empty_content(scene: Demo) -> impl kivact::view::Element {
    specimen(
        "FBK-05",
        "Empty state",
        kivact::ui::empty_state("NO RECORDS")
            .icon(())
            .description("No operations match. Clear the filter or create a new operation.")
            .actions(line((
                kivact::ui::button("Clear filter")
                    .variant(kivact::ui::Slant)
                    .size(kivact::ui::ControlSize::Small)
                    .variant(kivact::ui::ButtonVariant::Ghost),
                kivact::ui::button("New registration")
                    .variant(kivact::ui::Slant)
                    .icon(kivact::view::icon("plus"))
                    .size(kivact::ui::ControlSize::Small)
                    .on_press(move || scene.show(Overlay::Create)),
            ))),
    )
}
