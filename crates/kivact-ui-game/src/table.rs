//! Role: The console's table cell looks for the standard data table's columns.

use crate::surface::mono;
use crate::variants::AvatarVariant;
use kivact::component;
use kivact::i18n::message;
use kivact::interaction::Tone;
use kivact::state::IntoValue;
use kivact::style::Color;
use kivact::style::color::{ACCENT, TEXT_MUTED};
use kivact::style::{Align, pct, style};
use kivact::ui::ControlSize;
use kivact::view::{Element, IntoView, View, column, node, row, text};

/// The console's cell looks, for a data column's cell to show. Give one to
/// [`console_cell`] in the column's cell closure.
#[derive(Clone, Debug, PartialEq)]
pub enum TableCellVariant {
    /// An identifier in the mono type, in the accent.
    Identifier(String),
    /// A title over a muted description.
    Detail {
        /// The title in bold.
        title: String,
        /// The description in the muted small type.
        description: String,
    },
    /// An avatar of `color` beside the user's name.
    User {
        /// The name shown beside the avatar.
        name: String,
        /// The initials the avatar shows.
        initials: String,
        /// The fill of the avatar's facet.
        color: Color,
    },
    /// A badge of `label` in `tone`.
    Badge {
        /// The badge's label.
        label: String,
        /// The badge's tone.
        tone: Tone,
    },
    /// A thin progress bar in `tone` with its percentage.
    Progress {
        /// The percentage, from 0 to 100.
        value: f64,
        /// The bar's tone.
        tone: Tone,
    },
}

/// The table cell of the console's look: the cell's view as the node's child.
#[component]
pub fn console_cell(cell: TableCellVariant) -> impl Element {
    node().child(cell_view(cell))
}

fn cell_view(cell: TableCellVariant) -> View {
    match cell {
        TableCellVariant::Identifier(v) => mono(v.into_value())
            .style(style().font_size(11.5).color(ACCENT))
            .into_view(),
        TableCellVariant::Detail { title, description } => column((
            text(title).style(style().font_size(12.5).font_weight(700)),
            text(description).style(style().font_size(10.5).color(TEXT_MUTED)),
        ))
        .style(style().gap(1))
        .into_view(),
        TableCellVariant::User {
            name,
            initials,
            color,
        } => row((
            kivact::ui::avatar(initials.clone().into_value())
                .fallback_content(move || text(initials.clone()))
                .variant(AvatarVariant { side: 24., color }),
            text(name).style(style().font_size(12.5)),
        ))
        .style(style().gap(10).align_items(Align::Center))
        .into_view(),
        TableCellVariant::Badge { label, tone } => {
            kivact::ui::badge(label.into_value()).tone(tone).into_view()
        }
        TableCellVariant::Progress { value, tone } => row((
            kivact::ui::progress((Some(value / 100.0)).into_value())
                .name(message("game.table.progress", "Progress"))
                .variant(tone)
                .size(ControlSize::Small)
                .style(style().flex_grow(1.)),
            mono(format!("{value:.0}%").into_value()),
        ))
        .style(style().gap(8).align_items(Align::Center).width(pct(100)))
        .into_view(),
    }
}
