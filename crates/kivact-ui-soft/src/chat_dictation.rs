//! Role: Expose application-owned voice dictation controls alongside canonical composers.
use crate::color as c;
use crate::recipe_parts::ring;

use kivact::component;
use kivact::custom::Caller;
use kivact::i18n::message;
use kivact::state::{Callback, Value};
use kivact::style::{Align, FlexDirection, Justify, style};
use kivact::ui::ControlSize;
use kivact::view::{Element, Node, Role, node, row, text};

fn control_side(size: ControlSize) -> i32 {
    match size {
        ControlSize::ExtraSmall => 24,
        ControlSize::Small => 28,
        ControlSize::Medium => 36,
        ControlSize::Large => 44,
    }
}
/// Read-only microphone capability and audio samples supplied by the application.
// Read-only application-owned microphone capability and sampled audio facts; the control owns no recording transitions.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum DictationState {
    /// The application cannot offer dictation.
    #[default]
    Unsupported,
    /// The application can offer dictation and is not listening.
    Idle,
    /// The application is listening.
    Listening {
        /// Current normalized input amplitude used when band samples are absent.
        volume: f64,
        /// Normalized frequency-band amplitudes used to draw the listening waveform.
        bands: Vec<f64>,
    },
}

/// Dictation toggle displaying application-owned recording state.
///
/// `on_toggle` runs when the control is pressed; recording remains application-owned. The
/// accessible name is the element's `.name(..)`, and it is "Dictation" when none is given.
#[component]
pub fn chat_dictation_button(
    state: Value<DictationState>,
    on_toggle: Option<Callback<()>>,
    #[prop(default = ControlSize::Medium)] size: Value<ControlSize>,
    #[prop(default = true)] hide_when_unsupported: Value<bool>,
    #[caller] caller: Caller,
) -> impl Element {
    let name = caller
        .name()
        .unwrap_or_else(|| message("soft.chat_dictation_button.label", "Dictation"));
    let side = control_side(size.get());
    let listening = move || matches!(state.get(), DictationState::Listening { .. });
    let icon = text("🎙").hidden(listening);
    let waveform = row(dictation_bars(state, side))
        .hidden(move || !listening())
        .style(style().gap(2).align_items(Align::Center));
    node()
        .with_node(move |node| node.role(Role::Button))
        .focusable(true)
        .name(name)
        .checked(listening)
        .disabled(move || matches!(state.get(), DictationState::Unsupported))
        .on_press(move || {
            if let Some(toggle) = &on_toggle {
                toggle.call(());
            }
        })
        .hidden(move || {
            hide_when_unsupported.get() && matches!(state.get(), DictationState::Unsupported)
        })
        .with_node(move |node| {
            node.base_style(move || {
                style()
                    .flex_direction(FlexDirection::Row)
                    .size(side, side)
                    .border_radius(side as f64 / 2.0)
                    .align_items(Align::Center)
                    .justify_content(Justify::Center)
                    .color(c::TEXT_PRIMARY)
                    .hover(style().background(c::OVERLAY_HOVER))
                    .focus_visible(ring(c::FOCUS_RING))
            })
        })
        .child((icon, waveform))
}

fn dictation_bars(state: Value<DictationState>, side: i32) -> Vec<Node> {
    (0..5)
        .map(|index| {
            node().style(move || {
                let (band, volume) = state.with(|state| match state {
                    DictationState::Listening { volume, bands } => {
                        (bands.get(index).copied().unwrap_or(0.0), *volume)
                    }
                    DictationState::Unsupported | DictationState::Idle => (0.0, 0.0),
                });
                let level = (band.max(0.0) / 0.2).sqrt().min(1.0);
                let height = if side <= 28 { 14.0 } else { 18.0 };
                style()
                    .size(2.5, height * (0.08 + level * 0.92))
                    .border_radius(2)
                    .background(if volume >= 0.3 { c::ERROR } else { c::ACCENT })
            })
        })
        .collect::<Vec<_>>()
}
