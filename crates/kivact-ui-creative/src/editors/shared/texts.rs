//! Role: Texts that drawing code uses to name its regions, read once in setup.
//!
//! The texts that drawing code names its regions with (a node's socket, a
//! timeline's playhead). A drawing closure cannot create a message, so
//! they are read once in setup and used while drawing; the drawing runs
//! again when the App switches its messages.

use kivact::Value;
use kivact::i18n::{interpolate, message};

/// The editors' texts (`editor.*`, `viewport.*`, `arrangement.*`).
#[derive(Clone, Copy)]
pub(crate) struct Texts {
    playhead: Value<String>,
    selected: Value<String>,
    unselected: Value<String>,
    on: Value<String>,
    off: Value<String>,
    socket_in: Value<String>,
    socket_out: Value<String>,
    trim_start: Value<String>,
    trim_end: Value<String>,
    port_link: Value<String>,
}

impl Texts {
    /// Read the texts. Setup only.
    #[track_caller]
    pub(crate) fn new() -> Self {
        Self {
            playhead: message("creative.editor.playhead", "Playhead"),
            selected: message("creative.editor.selected", "selected"),
            unselected: message("creative.editor.unselected", "unselected"),
            on: message("creative.editor.on", "On"),
            off: message("creative.editor.off", "Off"),
            socket_in: message("creative.editor.socket_in", "{item} input {socket}"),
            socket_out: message("creative.editor.socket_out", "{item} output {socket}"),
            trim_start: message("creative.editor.trim_start", "{item} start trim"),
            trim_end: message("creative.editor.trim_end", "{item} end trim"),
            port_link: message(
                "creative.editor.port_link",
                "Link {from} output {output} to {to} input {input}",
            ),
        }
    }

    pub(crate) fn playhead(&self) -> String {
        self.playhead.get()
    }

    pub(crate) fn selection(&self, selected: bool) -> String {
        if selected {
            self.selected.get()
        } else {
            self.unselected.get()
        }
    }

    pub(crate) fn toggle(&self, on: bool) -> String {
        if on { self.on.get() } else { self.off.get() }
    }

    pub(crate) fn socket(&self, outgoing: bool, item: &str, socket: &str) -> String {
        let template = if outgoing {
            self.socket_out
        } else {
            self.socket_in
        };
        interpolate(&template.get(), &[("item", item), ("socket", socket)])
    }

    pub(crate) fn trim(&self, end: bool, item: &str) -> String {
        let template = if end { self.trim_end } else { self.trim_start };
        interpolate(&template.get(), &[("item", item)])
    }

    pub(crate) fn port_link(&self, from: &str, output: usize, to: &str, input: usize) -> String {
        interpolate(
            &self.port_link.get(),
            &[
                ("from", from),
                ("output", &output.to_string()),
                ("to", to),
                ("input", &input.to_string()),
            ],
        )
    }
}
