//! Role: File-transfer rows accept application-owned metadata and progress.

use crate::color;
use kivact::component;
use kivact::i18n::message;
use kivact::state::Value;
use kivact::style::color as core;
use kivact::style::{Align, Justify, style};
use kivact::ui::spinner;
use kivact::view::{Element, Role, column, dynamic, icon, node, row, text};

/// A file row with an actual byte count and a percentage supplied by its owner.
/// The name goes on the title text and the progress bar, the parts inside the row.
///
/// The component performs no file I/O or network transfer. Its progress and
/// completion indicator update when the caller's value changes. Unknown metadata
/// is displayed explicitly instead of inventing a size.
#[component]
pub fn upload_row(
    name: Value<String>,
    bytes: Value<Option<u64>>,
    percentage: Value<f64>,
) -> impl Element {
    let unknown_size = message("pop.upload.unknown_size", "Unknown size");
    let done = message("pop.upload.done", "Upload complete");
    let uploading = message("pop.upload.uploading", "Uploading");
    let normalized = move || {
        let value = percentage.get();
        if value.is_finite() {
            value.clamp(0.0, 100.0)
        } else {
            0.0
        }
    };
    row((
        node().child(icon("file-text")).style(
            style()
                .size(32, 32)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .border_radius(6)
                .background(core::SURFACE_ALT)
                .color(core::TEXT_MUTED),
        ),
        column((
            text(name).style(style().font_size(13).font_weight(500)),
            kivact::ui::progress(move || Some(normalized() / 100.0))
                .name(name)
                .style(style().height(4)),
        ))
        .style(style().flex_grow(1).min_width(0).gap(6)),
        text(move || match bytes.get() {
            Some(size) if size > 1024 * 1024 => format!("{:.1}MB", size as f64 / 1048576.0),
            Some(size) => format!("{}KB", ((size as f64 / 1024.0).round() as u64).max(1)),
            None => unknown_size.get(),
        })
        .style(style().font_size(12).color(core::TEXT_MUTED)),
        dynamic(
            move || normalized() >= 100.0,
            move |complete| {
                if complete {
                    icon("circle-check")
                        .with_node(|root| root.role(Role::Status))
                        .name(done)
                        .style(style().color(color::MINT))
                        .into_view()
                } else {
                    spinner()
                        .size(kivact::ui::ControlSize::ExtraSmall)
                        .with_node(|root| root.role(Role::Status))
                        .name(uploading)
                        .into_view()
                }
            },
        ),
    ))
    .with_node(|root| root.role(Role::Group))
    .name(name)
    .style(
        style()
            .gap(10)
            .padding_x(12)
            .padding_y(8)
            .align_items(Align::Center)
            .border(1, core::BORDER)
            .border_radius(8),
    )
}
