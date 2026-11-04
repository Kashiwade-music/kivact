//! Role: Decide color controls.
use super::super::color_to_hsv;
use super::super::geometry::Channel;
use super::data::{ColorControls, SwatchSelection};
use kivact::state::Step;
use kivact::style::Color;
pub(in crate::color_wheel) struct Machine;
pub(in crate::color_wheel) enum Event {
    ColorChanged(Color),
    AlphaChanged(bool),
    PaletteChanged(usize),
    Channel(Channel),
    Swatch(SwatchSelection),
    Hue(f64),
}
impl kivact::state::Machine for Machine {
    type State = ColorControls;
    type Event = Event;
    type Effect = ();
    fn transition(&self, state: &ColorControls, event: Event) -> Step<ColorControls, Event, ()> {
        let mut next = *state;
        match event {
            Event::ColorChanged(color) => {
                let (h, s, v) = color_to_hsv(color);
                if s > 0.00001 && v > 0.00001 {
                    next.hue = h;
                }
            }
            Event::AlphaChanged(false) if next.channel == Channel::Alpha => {
                next.channel = Channel::Hue
            }
            Event::PaletteChanged(len) => {
                if let SwatchSelection::Selected(index) = next.swatch {
                    next.swatch = if len == 0 {
                        SwatchSelection::None
                    } else {
                        SwatchSelection::Selected(index.min(len - 1))
                    };
                }
            }
            Event::Channel(channel) => next.channel = channel,
            Event::Swatch(swatch) => next.swatch = swatch,
            Event::Hue(hue) => next.hue = hue,
            _ => {}
        }
        Step::to(next)
    }
}
