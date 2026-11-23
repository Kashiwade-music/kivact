//! Role: Present an attached or detached field validation message.
use crate::color as c;
use crate::radius as r;
use crate::space as s;
use crate::type_scale as t;

use kivact::Prop;
use kivact::component;
use kivact::custom::Caller;
use kivact::state::Value;
use kivact::style::Align;
use kivact::style::pct;
use kivact::style::style;
use kivact::ui::theme::Variant;
use kivact::view::LiveRegion;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

/// Meaning and color of a field validation message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusKind {
    /// Invalid-value message, announced assertively when announcements are enabled.
    Error,
    /// Warning message, announced politely when announcements are enabled.
    Warning,
    /// Successful-validation message, announced politely when enabled.
    Success,
}

/// Placement of the validation-message surface.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Prop)]
pub enum FieldStatusVariant {
    /// Message visually attached to its field, the default.
    #[default]
    Attached,
    /// Standalone message surface.
    Detached,
}

impl Variant for FieldStatusVariant {}

/// Validation text and icon with optional live-region announcements.
///
/// The placement is the element's `.variant(FieldStatusVariant::..)`; attached by default.
#[component]
pub fn field_status(
    kind: StatusKind,
    message: Value<String>,
    #[prop(default = 16.0)] overlap: Value<f64>,
    #[prop(default = true)] announce: Value<bool>,
    #[caller] caller: Caller,
) -> impl Element {
    let variants = caller.variants();
    let mut root = node();
    if announce.get() {
        root = root.live_region(if kind == StatusKind::Error {
            LiveRegion::Assertive
        } else {
            LiveRegion::Polite
        });
    }
    let glyph = match kind {
        StatusKind::Error => "circle-x",
        StatusKind::Warning => "triangle-alert",
        StatusKind::Success => "circle-check",
    };
    root.child(
        row((
            icon(glyph).style(style().size(16, 16).flex_shrink(0)),
            text(message),
        ))
        .style(move || {
            let (background, color) = match kind {
                StatusKind::Error => (c::ERROR_MUTED, c::ON_ERROR_MUTED),
                StatusKind::Warning => (c::WARNING_MUTED, c::ON_WARNING_MUTED),
                StatusKind::Success => (c::SUCCESS_MUTED, c::ON_SUCCESS_MUTED),
            };
            let base = style()
                .width(pct(100))
                .gap(s::SPACE_S1)
                .align_items(Align::Start)
                .font_size(t::SIZE_SM)
                .line_height(20.0 / t::SIZE_SM)
                .color(color);
            let overlap = overlap.get().max(0.0);
            match variants.get::<FieldStatusVariant>().unwrap_or_default() {
                FieldStatusVariant::Attached => base
                    .background(background)
                    .border_radius(r::RADIUS_ELEMENT)
                    .margin_top(-overlap)
                    .padding_top(overlap + 8.0)
                    .padding_bottom(s::SPACE_S2)
                    .padding_x(s::SPACE_S2),
                FieldStatusVariant::Detached => base.padding_y(s::SPACE_S1),
            }
        }),
    )
}
