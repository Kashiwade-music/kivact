//! Role: Dense data displays with hard-edged status geometry.
use kivact::draw::Path;
use kivact::geometry::Point;
use kivact::style::{Color, ColorToken, Theme, pct, rgba, style};
use kivact::view::{IntoView, Role, View};
use kivact::view::{NodeBuilder, column, node};

use crate::Tokens;
use kivact::interaction::Tone;

/// The colors a game theme gives a semantic [`Tone`].
pub(crate) trait ToneExt {
    /// The tone's solid color.
    fn color(self, t: &Theme) -> Color;
    /// The tone's weak tint, for backgrounds.
    fn weak(self, t: &Theme) -> Color;
}

impl ToneExt for Tone {
    fn color(self, t: &Theme) -> Color {
        match self {
            Self::Info => t.colors().accent,
            Self::Success => t.colors().success,
            Self::Warning => t.colors().warning,
            Self::Danger => t.colors().danger,
            Self::Neutral => t.ext::<Tokens>().text_2,
            Self::Accent => t.ext::<Tokens>().yellow,
        }
    }
    fn weak(self, t: &Theme) -> Color {
        let dark = t.ext::<Tokens>().mode == kivact::style::ThemeMode::Dark;
        match self {
            Self::Info => t.ext::<Tokens>().primary_weak,
            Self::Success => {
                if dark {
                    rgba(0x2cc48a, 0.14)
                } else {
                    kivact::style::rgb(0xddf5eb)
                }
            }
            Self::Warning => {
                if dark {
                    rgba(0xf5a524, 0.14)
                } else {
                    kivact::style::rgb(0xfff1d6)
                }
            }
            Self::Danger => {
                if dark {
                    rgba(0xff5a6e, 0.14)
                } else {
                    kivact::style::rgb(0xfde4e8)
                }
            }
            Self::Neutral => t.ext::<Tokens>().surface_3,
            Self::Accent => t.ext::<Tokens>().yellow,
        }
    }
}

/// One entry of an [`activity`] timeline.
pub struct ActivityEntry {
    time: String,
    message: View,
    tone: Tone,
}
impl ActivityEntry {
    /// An entry at `time` marked in `tone`. `message` is a plain string or any
    /// application-owned view, such as rich text spans.
    pub fn new(time: impl Into<String>, message: impl IntoView, tone: Tone) -> Self {
        Self {
            time: time.into(),
            message: message.into_view(),
            tone,
        }
    }
}

/// Diamond-marked activity timeline.
/// Entries own non-cloneable views and are consumed once as composition.
#[track_caller]
pub fn activity(entries: Vec<ActivityEntry>) -> impl kivact::view::Component {
    column(
        entries
            .into_iter()
            .map(
                |ActivityEntry {
                     time,
                     message,
                     tone,
                 }| {
                    column((
                        crate::mono(time).style(style().color(ColorToken::TextMuted)),
                        node().style(style().font_size(12.5)).child(message),
                    ))
                    .style(style().padding_left(26).padding_bottom(14).gap(2))
                    .draw(move |d, s| {
                        d.line(
                            Point::new(5., 7.),
                            Point::new(5., s.height),
                            1.,
                            d.theme().ext::<Tokens>().line_strong,
                        );
                        d.fill_path(
                            &Path::new()
                                .move_to(Point::new(5., 2.))
                                .line_to(Point::new(10., 7.))
                                .line_to(Point::new(5., 12.))
                                .line_to(Point::new(0., 7.))
                                .close(),
                            tone.color(d.theme()),
                        );
                    })
                },
            )
            .collect::<Vec<_>>(),
    )
    .with_node(|root| root.role(Role::List))
    .style(style().width(pct(100)))
}
