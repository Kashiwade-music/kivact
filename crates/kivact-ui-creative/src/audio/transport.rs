//! Role: The transport bar: its TransportState and play, stop and position view.

use crate::color::{CANVAS, X};
use kivact::component;
use kivact::i18n::message;
use kivact::state::{Callback, IntoValue, Signal};
use kivact::style::{Align, style};
use kivact::ui::ButtonVariant;
use kivact::view::{Element, Role, row};

/// Playback button state; the application owns its clock.
#[derive(Clone, Debug, PartialEq)]
pub struct TransportState {
    pub playing: bool,
    pub recording: bool,
    pub cycling: bool,
}

impl Default for TransportState {
    fn default() -> Self {
        Self {
            playing: false,
            recording: false,
            cycling: true,
        }
    }
}

/// Playback controls without a clock. The application handles seeking.
///
/// `on_to_start` reports a request to return to the application's start
/// position; the start button is shown only when it is set.
#[component]
pub fn transport(
    binding: Signal<TransportState>,
    on_to_start: Option<Callback<()>>,
) -> impl Element {
    let state = binding.into_value();
    let modify = move |f: fn(&mut TransportState)| {
        let mut next = state.get_untracked();
        f(&mut next);
        binding.set(next);
    };
    row((
        to_start_button(on_to_start),
        kivact::ui::button(message("creative.transport.stop", "Stop"))
            .hide_label(true)
            .icon(kivact::view::icon("stop"))
            .variant(ButtonVariant::Ghost)
            .on_press(move || modify(|s| s.playing = false)),
        kivact::ui::button(message("creative.transport.play", "Play"))
            .hide_label(true)
            .icon(kivact::view::icon("play"))
            .variant(ButtonVariant::Ghost)
            .with_node(move |node| node.selected(move || state.get().playing))
            .on_press(move || modify(|s| s.playing = !s.playing)),
        kivact::ui::button(message("creative.transport.record", "Record"))
            .hide_label(true)
            .icon(kivact::view::icon("record"))
            .variant(ButtonVariant::Ghost)
            .with_node(move |node| node.selected(move || state.get().recording))
            .with_node(move |node| node.style(style().color(X)))
            .on_press(move || modify(|s| s.recording = !s.recording)),
        kivact::ui::button(message("creative.transport.cycle", "Cycle"))
            .hide_label(true)
            .icon(kivact::view::icon("repeat"))
            .variant(ButtonVariant::Ghost)
            .with_node(move |node| node.selected(move || state.get().cycling))
            .on_press(move || modify(|s| s.cycling = !s.cycling)),
    ))
    .role(Role::Toolbar)
    .name(message("creative.transport.label", "Transport"))
    .style(
        style()
            .align_items(Align::Center)
            .gap(2)
            .padding(3)
            .border_radius(9)
            .background(CANVAS),
    )
}

/// The optional start button, first in the transport.
fn to_start_button(on_to_start: Option<Callback<()>>) -> Option<impl Element> {
    on_to_start.map(|callback| {
        kivact::ui::button(message("creative.transport.start", "To Start"))
            .hide_label(true)
            .icon(kivact::view::icon("start"))
            .variant(ButtonVariant::Ghost)
            .on_press(move || callback.call(()))
    })
}
