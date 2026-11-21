//! Role: The overview page of the Creative catalog: the kit's status, accent and transport chrome.

use super::chrome::{
    accent_controls, application_menus, catalog_status, performance_meters, transport_readout,
};
use super::common::transport;
use catalog_shell::{Example, Page};
use kivact::prelude::*;
use kivact_ui_creative as ui;

pub(crate) fn page() -> Page {
    Page::new(
        "overview",
        "Overview",
        "Overview",
        include_str!("overview.rs"),
    )
    .description("The kit's chrome: accent, menus, transport readout and status.")
    .example(Example::new("overview", "Overview", "", demo))
}

// example: overview
fn demo() -> impl IntoView {
    let mode = use_theme().get().mode();
    let accent = signal(ui::Accent::Iris);
    let status = signal("Ready: Space plays, F3 searches, F12 renders".to_string());
    let state = transport();
    provide_theme(
        move || {
            ui::theme_with(
                mode,
                ui::ThemeOptions {
                    accent: accent.get(),
                    ..Default::default()
                },
            )
        },
        move || {
            column((
                text("Creative")
                    .role(Role::Heading)
                    .style(style().font_size(24).font_weight(650)),
                row(accent_controls(accent)),
                row(application_menus(status)),
                transport_readout(state),
                performance_meters(),
                catalog_status(status, state),
            ))
            .style(style().gap(12).width(pct(100)))
        },
    )
}
// end example
