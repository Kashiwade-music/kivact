//! Role: The Lyra alert: a bordered callout with icon, title, text and action.

use kivact::state::IntoValue;
use kivact::view::{Node, NodeBuilder, column, node, row, text};

use kivact::state::IntoText;
use kivact::style::{Align, Position, style};
use kivact::view::{IntoView, Role, View};

/// The colors of an [`Alert`] (alert.tsx `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum AlertVariant {
    /// Card colors.
    #[default]
    Default,
    /// Destructive text and icon.
    Destructive,
}

/// A callout (alert.tsx) with `Role::Alert`. Created by [`alert`].
pub struct Alert {
    root: Node,
    title: kivact::state::Value<String>,
    variant: AlertVariant,
    tone: kivact::state::Value<kivact::interaction::Tone>,
    icon: Option<View>,
    description: Option<kivact::state::Value<String>>,
    action: Option<kivact::view::View>,
}

/// An alert titled `title`.
///
/// ```no_run
/// use kivact::view::{icon};
/// use kivact::prelude::*;
/// use kivact_ui_basic as basic;
///
/// fn heads_up() -> impl IntoView {
///     basic::alert("Heads up!")
///         .icon(icon("info"))
///         .description("You can add components to your app using the CLI.")
/// }
/// ```
#[track_caller]
pub fn alert(title: impl IntoText) -> Alert {
    let title = title.into_text_value();
    Alert {
        root: node().with_node(|root| root.role(Role::Alert)).name(title),
        title,
        variant: AlertVariant::Default,
        tone: kivact::interaction::Tone::Neutral.into_value(),
        icon: None,
        description: None,
        action: None,
    }
}

impl Alert {
    /// Semantic accent; neutral by default. The destructive visual variant takes precedence.
    pub fn tone(mut self, tone: impl kivact::state::IntoValue<kivact::interaction::Tone>) -> Self {
        let tone = tone.into_value();
        self.tone = tone.into_value();
        self
    }

    /// Choose the colors.
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Show `icon` (a `kivact::view::icon`) to the left of the text.
    pub fn icon(mut self, icon: impl IntoView) -> Self {
        self.icon = Some(icon.into_view());
        self
    }

    /// Show `text` under the title.
    pub fn description(mut self, text: impl IntoText) -> Self {
        self.description = Some(text.into_text_value());
        self
    }

    /// Show `action` (such as a small button) at the right, centered on the
    /// alert's height. shadcn puts it at the top right, 5 px from both edges.
    pub fn action(mut self, action: impl IntoView) -> Self {
        self.action = Some(action.into_view());
        self
    }
}

impl NodeBuilder for Alert {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Alert {
    fn into_view(self) -> View {
        let variant = self.variant;
        let tone = self.tone;
        let has_action = self.action.is_some();
        // The title inherits the alert's color (the variant's).
        let body = column((
            text(self.title).style(style().weight(500)),
            self.description.map(|description| {
                text(description).style(move || {
                    let p = kivact::style::use_tokens::<crate::Tokens>();
                    style().line_height(1.625).color(match variant {
                        AlertVariant::Default => p.muted_foreground,
                        AlertVariant::Destructive => p.destructive.with_alpha(0.9),
                    })
                })
            }),
        ))
        .style(style().gap(2).grow(1).min_width(0));
        let action = self.action.map(|action| {
            row(action).style(
                style()
                    .position(Position::Absolute)
                    .top(0)
                    .bottom(0)
                    .right(5)
                    .align_items(Align::Center),
            )
        });
        self.root
            .base_style(move || {
                let p = kivact::style::use_tokens::<crate::Tokens>();
                style()
                    .direction(kivact::style::Direction::Row)
                    .align_items(Align::Start)
                    .position(Position::Relative)
                    .width(kivact::style::pct(100))
                    .gap(8)
                    .padding_left(10)
                    .padding_y(8)
                    .padding_right(if has_action { 72.0 } else { 10.0 })
                    .border(1, p.border)
                    .radius(0)
                    .background(p.card)
                    .font_size(12)
                    .color(match variant {
                        AlertVariant::Default => match tone.get() {
                            kivact::interaction::Tone::Neutral => p.card_foreground,
                            kivact::interaction::Tone::Info | kivact::interaction::Tone::Accent => {
                                kivact::style::ColorToken::Accent.into()
                            }
                            kivact::interaction::Tone::Success => {
                                kivact::style::ColorToken::Success.into()
                            }
                            kivact::interaction::Tone::Warning => {
                                kivact::style::ColorToken::Warning.into()
                            }
                            kivact::interaction::Tone::Danger => p.destructive,
                        },
                        AlertVariant::Destructive => p.destructive,
                    })
            })
            .child(self.icon)
            .child(body)
            .child(action)
            .into_view()
    }
}
