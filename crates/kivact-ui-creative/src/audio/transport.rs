//! Role: The transport bar: its TransportState and play, stop and position view.

use kivact::control::{Node, NodeBuilder, row};
use kivact::i18n::message;
use kivact::style::{Align, style};
use kivact::ui::ButtonVariant;
use kivact::view::HandlerResult;
use kivact::view::{IntoValue, IntoView, Role, View, callback};
use kivact::{Binding, Callback};

/// Playback button state; the application owns its clock.
/// Built with [`TransportState::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct TransportState {
    pub playing: bool,
    pub recording: bool,
    pub cycling: bool,
}

impl TransportState {
    /// A transport that is `playing`, `recording` and `cycling` or not.
    pub const fn new(playing: bool, recording: bool, cycling: bool) -> Self {
        Self {
            playing,
            recording,
            cycling,
        }
    }
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
pub struct Transport {
    root: Node,
    binding: Binding<TransportState>,
    to_start: Option<Callback<()>>,
}

/// Create controls for the application's playback state.
#[track_caller]
pub fn transport(binding: impl Into<Binding<TransportState>>) -> Transport {
    Transport {
        root: row(())
            .role(Role::Toolbar)
            .name(message("creative.transport.label", "Transport"))
            .style(
                style()
                    .align_items(Align::Center)
                    .gap(2)
                    .padding(3)
                    .radius(9)
                    .background(crate::Color::Canvas),
            ),
        binding: binding.into(),
        to_start: None,
    }
}

impl Transport {
    /// Report a request to return to the application's start position.
    pub fn on_to_start<R: HandlerResult>(mut self, f: impl Fn() -> R + 'static) -> Self {
        self.to_start = Some(callback(move |_| f()));
        self
    }
}

impl NodeBuilder for Transport {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Transport {
    fn into_view(self) -> View {
        transport_view(self.root, self.binding, self.to_start).into_view()
    }
}

fn transport_view(
    root: Node,
    binding: Binding<TransportState>,
    to_start: Option<Callback<()>>,
) -> Node {
    let state = binding.into_value();
    let modify = move |f: fn(&mut TransportState)| {
        let mut next = state.get_untracked();
        f(&mut next);
        binding.set(next);
    };
    root.child((
        to_start.map(|callback| {
            kivact::ui::button(message("creative.transport.start", "To Start"))
                .hide_label(true)
                .icon(kivact::control::icon("start"))
                .variant(ButtonVariant::Ghost)
                .on_press(move || callback.call(()))
        }),
        kivact::ui::button(message("creative.transport.stop", "Stop"))
            .hide_label(true)
            .icon(kivact::control::icon("stop"))
            .variant(ButtonVariant::Ghost)
            .on_press(move || modify(|s| s.playing = false)),
        kivact::ui::button(message("creative.transport.play", "Play"))
            .hide_label(true)
            .icon(kivact::control::icon("play"))
            .variant(ButtonVariant::Ghost)
            .selected(move || state.get().playing)
            .on_press(move || modify(|s| s.playing = !s.playing)),
        kivact::ui::button(message("creative.transport.record", "Record"))
            .hide_label(true)
            .icon(kivact::control::icon("record"))
            .variant(ButtonVariant::Ghost)
            .selected(move || state.get().recording)
            .style(style().color(crate::Color::X))
            .on_press(move || modify(|s| s.recording = !s.recording)),
        kivact::ui::button(message("creative.transport.cycle", "Cycle"))
            .hide_label(true)
            .icon(kivact::control::icon("repeat"))
            .variant(ButtonVariant::Ghost)
            .selected(move || state.get().cycling)
            .on_press(move || modify(|s| s.cycling = !s.cycling)),
    ))
}
