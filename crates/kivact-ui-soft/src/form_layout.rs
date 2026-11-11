//! Role: Arrange fields with Neutral form spacing.
use crate::stack::rtl;

use kivact::view::Element;
use kivact::view::Node;
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

/// Placement of fields and their labels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FormDirection {
    /// Stack fields vertically, the default.
    #[default]
    Vertical,
    /// Place fields in equal-width columns.
    Horizontal,
    /// Align labels and fields in separate columns above 480 pixels; stack below.
    HorizontalLabels,
}

/// Requiredness hint inherited by form fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Optionality {
    /// Mark fields as optional.
    Optional,
    /// Mark fields as required.
    Required,
}

/// Shared field placement and optionality inside a form layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FormLayoutCx {
    /// Fixed arrangement of the form fields.
    pub direction: FormDirection,
    /// Inherited optionality hint, or no shared hint.
    pub default_optionality: Option<Optionality>,
}

/// Form spacing and inherited label/optionality context.
pub struct FormLayout {
    root: Node,
    content: View,
    context: FormLayoutCx,
}

/// Mount form content once with vertical spacing and no optionality override.
pub fn form_layout(content: impl IntoView) -> FormLayout {
    FormLayout {
        root: node(),
        content: content.into_view(),
        context: FormLayoutCx::default(),
    }
}

impl FormLayout {
    /// Choose the field arrangement; vertical by default.
    pub fn direction(mut self, direction: FormDirection) -> Self {
        self.context.direction = direction;
        self
    }
    /// Provide the optionality hint inherited by nested form fields.
    pub fn default_optionality(mut self, optionality: Optionality) -> Self {
        self.context.default_optionality = Some(optionality);
        self
    }
}

impl Element for FormLayout {
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
                let spacing = kivact::style::use_tokens::<crate::Tokens>().spacing;
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
                    let t = kivact::style::use_tokens::<crate::Tokens>();
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
