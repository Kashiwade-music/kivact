//! Role: Dense data displays with hard-edged status geometry.
use crate::color::LINE_STRONG;
use crate::surface::mono;
use crate::tone::ToneExt;
use kivact::component;
use kivact::draw::Path;
use kivact::geometry::Point;
use kivact::interaction::Tone;
use kivact::state::IntoValue;
use kivact::style::color::TEXT_MUTED;
use kivact::style::{pct, style};
use kivact::view::{Element, IntoView, Role, View, column, node};

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

/// Diamond-marked activity timeline. Items own non-cloneable views and are
/// consumed once as composition.
#[component]
pub fn activity(items: Vec<ActivityEntry>) -> impl Element {
    column(
        items
            .into_iter()
            .map(
                |ActivityEntry {
                     time,
                     message,
                     tone,
                 }| {
                    column((
                        mono(time.into_value()).style(style().color(TEXT_MUTED)),
                        node().style(style().font_size(12.5)).child(message),
                    ))
                    .style(style().padding_left(26).padding_bottom(14).gap(2))
                    .draw(move |d, s| {
                        d.line(
                            Point::new(5., 7.),
                            Point::new(5., s.height),
                            1.,
                            LINE_STRONG,
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
