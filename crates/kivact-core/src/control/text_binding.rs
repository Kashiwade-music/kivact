//! Role: This module binds rich text and styled spans in one render effect.

use std::cell::Cell;
use std::panic::Location;

use kivact_engine::bridge::PropValue;
use kivact_reactive::EffectKind;
use kivact_reactive::Prop as InputProp;
use kivact_wire::NodeId;

use crate::view::descriptor::Mcx;
use crate::view::span::{self, Span};
use crate::view::state::ViewState;

/// Write the joined text and the spans of a rich text from one render
/// effect (theme colors resolved; a theme switch writes them again),
/// reporting dropped values once.
pub(super) fn bind_rich(
    cx: &Mcx<'_>,
    id: NodeId,
    spans: InputProp<Vec<Span>>,
    at: &'static Location<'static>,
) {
    let weak = cx.weak();
    let theme = cx.theme();
    let owner = cx.owner;
    let reported = Cell::new(false);
    cx.effect(EffectKind::Render, at, move || {
        let theme = theme.map(|t| t.get()).unwrap_or_default();
        let (text, spans, dropped) = spans.with(|v| span::resolve(v, &theme));
        let Some(rt) = weak.upgrade() else {
            return;
        };
        {
            let writer = rt.running_effect();
            let mut tree = ViewState::tree_mut(&rt);
            tree.update(id, PropValue::Content(text), writer);
            tree.update(id, PropValue::TextSpans(spans), writer);
        }
        if dropped > 0 && !reported.replace(true) {
            rt.report(
                Some(owner),
                kivact_reactive::UiError::new(
                    kivact_reactive::ErrorKind::Unsupported,
                    format!(
                        "{dropped} font feature tag(s) that are not four ASCII characters; dropped"
                    ),
                    Some(at),
                ),
            );
        }
    });
}
