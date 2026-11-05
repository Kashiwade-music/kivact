//! Role: Apply Soft typography and metric and description appearances.
use crate::{TextKind, Tokens};
use kivact::style::{Style, Theme};
use kivact::ui::{HeadingState, Look, Variant};

/// Display typography independent of the heading's semantic level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadingDisplay {
    D1,
    D2,
    D3,
}
impl Variant for HeadingDisplay {}

pub(super) fn heading(t: &Theme, look: &Look<HeadingState>) -> Style {
    let kind = match look.variant::<HeadingDisplay>() {
        Some(HeadingDisplay::D1) => TextKind::Display1,
        Some(HeadingDisplay::D2) => TextKind::Display2,
        Some(HeadingDisplay::D3) => TextKind::Display3,
        None => TextKind::Body,
    };
    crate::text::type_style(
        &t.ext::<Tokens>(),
        kind,
        Some(u32::from(look.level)),
        None,
        None,
        None,
    )
}
