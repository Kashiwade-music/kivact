//! Role: Arrange fields with Neutral form spacing.
use crate::stack::rtl;
use crate::tokens;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::node;

use kivact::state::provide;
use kivact::style::Align;
use kivact::style::Direction;
use kivact::style::GridAutoFlow;
use kivact::style::Track;
use kivact::style::auto;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::View;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FormDirection {
    #[default]
    Vertical,
    Horizontal,
    HorizontalLabels,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Optionality {
    Optional,
    Required,
}

/// Shared field placement and optionality inside a form layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FormLayoutCx {
    pub direction: FormDirection,
    pub default_optionality: Option<Optionality>,
}

pub struct FormLayout {
    root: Node,
    content: View,
    context: FormLayoutCx,
}

pub fn form_layout(content: impl IntoView) -> FormLayout {
    FormLayout {
        root: node(),
        content: content.into_view(),
        context: FormLayoutCx::default(),
    }
}

impl FormLayout {
    pub fn direction(mut self, direction: FormDirection) -> Self {
        self.context.direction = direction;
        self
    }
    pub fn default_optionality(mut self, optionality: Optionality) -> Self {
        self.context.default_optionality = Some(optionality);
        self
    }
}

impl NodeBuilder for FormLayout {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for FormLayout {
    fn into_view(self) -> View {
        let Self {
            root,
            content,
            context,
        } = self;
        root.with_node(move |node| {
            node.base_style(move || {
                let spacing = tokens().spacing;
                let base = style().gap(spacing.s4);
                match context.direction {
                    FormDirection::Horizontal => base
                        .display(kivact::style::Display::Grid)
                        .grid_auto_flow(GridAutoFlow::Column)
                        .grid_auto_columns(Track::fr(1)),
                    FormDirection::HorizontalLabels
                        if kivact::platform::window_size().get().width > 480.0 =>
                    {
                        base.display(kivact::style::Display::Grid)
                            .grid_template_columns((Track::from(auto()), Track::fr(1)))
                            .row_gap(spacing.s3)
                            .align_items(Align::Start)
                    }
                    _ => base
                        .direction(Direction::Column)
                        .align_items(Align::Stretch),
                }
            })
        })
        .child(provide(context, move || content))
        .into_view()
    }
}

/// Place the label and field in consecutive tracks in a horizontal-labels layout.
pub fn form_row(label: impl IntoView, field: impl IntoView) -> impl IntoView {
    (
        node()
            .with_node(move |node| {
                node.base_style(move || {
                    let cx = kivact::state::try_use_context::<FormLayoutCx>()
                        .map(|value| value.get())
                        .unwrap_or_default();
                    let t = tokens();
                    let base = style();
                    if cx.direction == FormDirection::HorizontalLabels
                        && kivact::platform::window_size().get().width > 480.0
                    {
                        let base = base.padding_top(t.sizes.border_width + t.spacing.s1);
                        if rtl() {
                            base.align_self(Align::End)
                        } else {
                            base
                        }
                    } else {
                        base
                    }
                })
            })
            .child(label),
        field,
    )
}
