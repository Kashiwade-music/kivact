//! Role: The state of one SVG export: its body, definitions and open groups.

use super::encode::number;
use crate::{Result, UiError};
use kivact_runtime::ErrorKind;
use kivact_types::ComputedTextStyle;
use std::panic::Location;

/// The state of one export: the body, the definitions and the open groups.
pub(super) struct Emitter<'a> {
    pub(super) body: String,
    pub(super) defs: String,
    pub(super) next_id: u32,
    pub(super) depth: usize,
    pub(super) engine: &'a mut kivact_engine::api::OffscreenRenderer,
    pub(super) measurer: &'a kivact_engine::api::TextMeasurer,
    pub(super) inherited: &'a ComputedTextStyle,
    pub(super) at: &'static Location<'static>,
}

impl<'a> Emitter<'a> {
    pub(super) fn new(
        engine: &'a mut kivact_engine::api::OffscreenRenderer,
        measurer: &'a kivact_engine::api::TextMeasurer,
        inherited: &'a ComputedTextStyle,
        at: &'static Location<'static>,
    ) -> Self {
        Self {
            body: String::new(),
            defs: String::new(),
            next_id: 0,
            depth: 0,
            engine,
            measurer,
            inherited,
            at,
        }
    }

    /// An error for a command or value SVG cannot express, located at the call.
    pub(super) fn unsupported(&self, what: &str) -> UiError {
        UiError::new(
            ErrorKind::Unsupported,
            format!("SVG cannot express {what}"),
            Some(self.at),
        )
    }

    /// A fresh identifier for a definition (`prefix` plus a counter).
    pub(super) fn id(&mut self, prefix: &str) -> String {
        self.next_id += 1;
        format!("{prefix}{}", self.next_id)
    }

    /// The whole document: the root element, the definitions and the body.
    pub(super) fn finish(self, width: f64, height: f64) -> Result<String> {
        if self.depth != 0 {
            return Err(UiError::new(
                ErrorKind::Message,
                "the drawing leaves a layer, clip or transform open",
                Some(self.at),
            ));
        }
        let (w, h) = (number(width), number(height));
        let mut out = String::new();
        out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        out.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n"
        ));
        if !self.defs.is_empty() {
            out.push_str("<defs>\n");
            out.push_str(&self.defs);
            out.push_str("</defs>\n");
        }
        out.push_str(&self.body);
        out.push_str("</svg>\n");
        Ok(out)
    }
}
