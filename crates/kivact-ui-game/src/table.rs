//! Role: The console's table cell looks for the standard data table's columns.
use kivact::i18n::message;
use kivact::style::{Align, Color, ColorToken, pct, style};
use kivact::view::{IntoView, View};
use kivact::view::{NodeBuilder, column, row, text};

use kivact::interaction::Tone;

/// The console's cell looks, for a [`DataColumn::cell`](kivact::ui::DataColumn::cell)
/// to return.
#[derive(Clone, Debug, PartialEq)]
pub enum GameCellVariant {
    Identifier(String),
    Detail {
        title: String,
        description: String,
    },
    User {
        name: String,
        initials: String,
        color: Color,
    },
    Badge {
        label: String,
        tone: Tone,
    },
    Progress {
        value: f64,
        tone: Tone,
    },
}

impl IntoView for GameCellVariant {
    fn into_view(self) -> View {
        match self {
            Self::Identifier(v) => crate::mono(v)
                .style(style().font_size(11.5).color(ColorToken::Accent))
                .into_view(),
            Self::Detail { title, description } => column((
                text(title).style(style().font_size(12.5).weight(700)),
                text(description).style(style().font_size(10.5).color(ColorToken::TextMuted)),
            ))
            .style(style().gap(1))
            .into_view(),
            Self::User {
                name,
                initials,
                color,
            } => row((
                kivact::ui::avatar(initials.clone())
                    .fallback_content(move || text(initials.clone()))
                    .variant(crate::AvatarVariant {
                        side: 24.,
                        color: color,
                    }),
                text(name).style(style().font_size(12.5)),
            ))
            .style(style().gap(10).align_items(Align::Center))
            .into_view(),
            Self::Badge { label, tone } => kivact::ui::badge(label).tone(tone).into_view(),
            Self::Progress { value, tone } => row((
                kivact::ui::progress(Some(value / 100.0))
                    .name(message("game.table.progress", "Progress"))
                    .variant(tone)
                    .size(kivact::ui::ControlSize::Small)
                    .style(style().grow(1)),
                crate::mono(format!("{value:.0}%")),
            ))
            .style(style().gap(8).align_items(Align::Center).width(pct(100)))
            .into_view(),
        }
    }
}
