//! Role: Audio time formatting and level calculations for catalog views.

use kivact::motion::{FrameRate, frame_loop};
use kivact::state::IntoValue;
use kivact::state::Signal;
use kivact::state::effect;
use kivact::state::{Binding, binding};
use kivact_ui_creative as ui;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TransportState {
    pub frame: f64,
    pub fps: f64,
    pub playing: bool,
    pub recording: bool,
    pub cycling: bool,
    pub loop_start: f64,
    pub loop_end: f64,
}
impl Default for TransportState {
    fn default() -> Self {
        Self {
            frame: 0.0,
            fps: 24.0,
            playing: false,
            recording: false,
            cycling: true,
            loop_start: 0.0,
            loop_end: 192.0,
        }
    }
}
pub(super) fn controls(state: Signal<TransportState>) -> Binding<ui::TransportState> {
    binding(
        (move || {
            let s = state.get();
            ui::TransportState {
                playing: s.playing,
                recording: s.recording,
                cycling: s.cycling,
            }
        })
        .into_value(),
        move |s: ui::TransportState| {
            state.update(|v| {
                v.playing = s.playing;
                v.recording = s.recording;
                v.cycling = s.cycling;
            })
        },
    )
}
pub(super) fn playback(state: Signal<TransportState>) {
    let clock = frame_loop(move |frame| {
        state.update(|s| {
            if !s.playing {
                return;
            }
            s.frame += frame.delta.as_secs_f64() * s.fps.max(1.0);
            if s.cycling && s.loop_end > s.loop_start && s.frame >= s.loop_end {
                s.frame = s.loop_start + (s.frame - s.loop_start) % (s.loop_end - s.loop_start);
            }
        })
    });
    effect(move || {
        if state.get().playing {
            clock.start(FrameRate::Hz(24.0));
        } else {
            clock.stop();
        }
    });
}

pub(super) fn smpte(frame: f64) -> String {
    let f = frame.max(0.0) as u64;
    format!(
        "{:02}:{:02}:{:02}:{:02}",
        f / 86400,
        f / 1440 % 60,
        f / 24 % 60,
        f % 24
    )
}

pub(super) fn level(state: Signal<TransportState>, channel: usize) -> impl Fn() -> f64 + Copy {
    move || {
        let s = state.get();
        let phase = s.frame / (13.0 + channel as f64) + channel as f64;
        (phase.sin().abs() * 0.32 + 0.58).min(1.0)
    }
}
