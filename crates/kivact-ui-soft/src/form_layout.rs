//! Role: Arrange fields with Neutral form spacing.
use crate::metrics;
use crate::space as s;
use crate::space::Space;
use crate::stack::rtl;

use kivact::component;
use kivact::state::provide;
use kivact::state::try_use_context;
use kivact::style::Align;
use kivact::style::FlexDirection;
use kivact::style::GridAutoFlow;
use kivact::style::Track;
use kivact::style::auto;
use kivact::style::style;
use kivact::view::Element;
use kivact::view::IntoView;
use kivact::view::View;
use kivact::view::node;

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
#[component]
pub fn form_layout(
    content: View,
    #[prop(default = FormDirection::Vertical)] direction: FormDirection,
    default_optionality: Option<Optionality>,
) -> impl Element {
    let context = FormLayoutCx {
        direction,
        default_optionality,
    };
    node()
        .with_node(move |node| {
            node.base_style(move || {
                let base = style().gap(s::SPACE_S4);
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
                            .row_gap(s::SPACE_S3)
                            .align_items(Align::Start)
                    }
                    _ => base
                        .flex_direction(FlexDirection::Column)
                        .align_items(Align::Stretch),
                }
            })
        })
        .child(provide(context, move || content))
}

/// Place the label and field in consecutive tracks in a horizontal-labels layout.
pub fn form_row(label: impl IntoView, field: impl IntoView) -> impl IntoView {
    (
        node()
            .with_node(move |node| {
                node.base_style(move || {
                    let cx = try_use_context::<FormLayoutCx>().unwrap_or_default();
                    let base = style();
                    if cx.direction == FormDirection::HorizontalLabels
                        && kivact::platform::window_size().get().width > 480.0
                    {
                        let base = base.padding_top(metrics::BORDER_WIDTH + Space::S1.px());
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
