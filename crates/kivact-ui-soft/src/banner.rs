//! Role: Provide the decorative tone icon of a canonical persistent alert.
use crate::icon::icon_view;
use kivact::component;
use kivact::interaction::Tone;

/// Decorative tone icon for a standard alert's explicit icon slot.
#[component]
pub fn banner_icon(tone: Tone) -> impl Element {
    icon_view(match tone {
        Tone::Warning => "triangle-alert",
        Tone::Danger => "circle-x",
        Tone::Success => "circle-check",
        _ => "info",
    })
}
