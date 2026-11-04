//! Role: Present an attached or detached field validation message.
use crate::theme_access::tokens;
use kivact::state::IntoValue;
use kivact::state::Value;
use kivact::view::Node;
use kivact::view::NodeBuilder;
use kivact::view::icon;
use kivact::view::node;
use kivact::view::row;
use kivact::view::text;

use kivact::state::IntoText;
use kivact::style::Align;
use kivact::style::pct;
use kivact::style::style;
use kivact::view::IntoView;
use kivact::view::LiveRegion;
use kivact::view::View;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusKind {
    Error,
    Warning,
    Success,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FieldStatusVariant {
    #[default]
    Attached,
    Detached,
}

pub struct FieldStatus {
    root: Node,
    kind: StatusKind,
    text: Value<String>,
    variant: Value<FieldStatusVariant>,
    overlap: Value<f64>,
    announce: Value<bool>,
}

pub fn field_status(kind: StatusKind, text: impl IntoText) -> FieldStatus {
    FieldStatus {
        root: node(),
        kind,
        text: text.into_text_value(),
        variant: (FieldStatusVariant::Attached).into_value(),
        overlap: (16.0).into_value(),
        announce: (true).into_value(),
    }
}

impl FieldStatus {
    pub fn variant(mut self, variant: impl IntoValue<FieldStatusVariant>) -> Self {
        self.variant = variant.into_value();
        self
    }

    pub fn overlap(mut self, overlap: impl IntoValue<f64>) -> Self {
        let overlap = overlap.into_value();
        self.overlap = (move || overlap.get().max(0.0)).into_value();
        self
    }

    pub fn announce(mut self, announce: impl IntoValue<bool>) -> Self {
        self.announce = announce.into_value();
        self
    }
}

impl NodeBuilder for FieldStatus {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for FieldStatus {
    fn into_view(self) -> View {
        let Self {
            mut root,
            kind,
            text: label,
            variant,
            overlap,
            announce,
        } = self;
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
                icon(glyph).style(style().size(16, 16).shrink(0)),
                text(label),
            ))
            .style(move || {
                let t = tokens();
                let c = &t.colors;
                let (background, color) = match kind {
                    StatusKind::Error => (c.error_muted, c.on_error_muted),
                    StatusKind::Warning => (c.warning_muted, c.on_warning_muted),
                    StatusKind::Success => (c.success_muted, c.on_success_muted),
                };
                let base = style()
                    .width(pct(100))
                    .gap(4)
                    .align_items(Align::Start)
                    .font_size(12)
                    .line_height(20.0 / 12.0)
                    .color(color);
                match variant.get() {
                    FieldStatusVariant::Attached => base
                        .background(background)
                        .radius(t.radius.element)
                        .margin_top(-overlap.get())
                        .padding_top(overlap.get() + 8.0)
                        .padding_bottom(8)
                        .padding_x(8),
                    FieldStatusVariant::Detached => base.padding_y(4),
                }
            }),
        )
        .into_view()
    }
}

kivact::state::constant_value!(FieldStatusVariant);
