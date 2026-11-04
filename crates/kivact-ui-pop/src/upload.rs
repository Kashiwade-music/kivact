//! Role: File-transfer rows accept application-owned metadata and progress.

use crate::Color;
use crate::spinner;
use kivact::i18n::message;
use kivact::state::{IntoText, IntoValue};
use kivact::style::{Align, ColorToken, Justify, style};
use kivact::view::IntoView;
use kivact::view::{NodeBuilder, column, icon, node, row, text};
use kivact::view::{Role, dynamic};

/// A file row with an actual byte count and a percentage supplied by its owner.
///
/// The component performs no file I/O or network transfer. Its progress and
/// completion indicator update when the caller's value changes. Unknown metadata
/// is displayed explicitly instead of inventing a size.
#[track_caller]
pub fn upload_row(
    name: impl IntoText,
    bytes: impl IntoValue<Option<u64>>,
    percentage: impl IntoValue<f64>,
) -> impl kivact::view::Component {
    let name = name.into_text_value();
    let bytes = bytes.into_value();
    let percentage = percentage.into_value();
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
                .radius(6)
                .background(ColorToken::SurfaceAlt)
                .color(ColorToken::TextMuted),
        ),
        column((
            text(name).style(style().font_size(13).weight(500)),
            kivact::ui::progress(move || Some(normalized() / 100.0))
                .name(name)
                .style(style().height(4)),
        ))
        .style(style().grow(1).min_width(0).gap(6)),
        text(move || match bytes.get() {
            Some(size) if size > 1024 * 1024 => format!("{:.1}MB", size as f64 / 1048576.0),
            Some(size) => format!("{}KB", ((size as f64 / 1024.0).round() as u64).max(1)),
            None => unknown_size.get(),
        })
        .style(style().font_size(12).color(ColorToken::TextMuted)),
        dynamic(
            move || normalized() >= 100.0,
            move |complete| {
                if complete {
                    icon("circle-check")
                        .with_node(|root| root.role(Role::Status))
                        .name(done)
                        .style(style().color(Color::Mint))
                        .into_view()
                } else {
                    spinner()
                        .style(style().size(16, 16))
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
            .border(1, ColorToken::Border)
            .radius(8),
    )
}
