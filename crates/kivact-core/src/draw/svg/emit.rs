//! Role: Turns each recorded draw command into SVG markup, collecting the shared definitions.
//!
//! Every push opens a `<g>` and every pop closes one, so the markup nests
//! exactly as the recorded stack does. Definitions (clip paths and
//! gradients) go to one `<defs>` block, numbered in the order they are
//! written.

use super::encode::number;
use super::fill::paint_attributes;
use crate::{Result, UiError};
use kivact_runtime::ErrorKind;
use kivact_types::{
    BlendMode, ComputedTextStyle, DrawCmd, DrawingLayer, Fill, Rect, Rgba, ShapePaint, Stroke,
    Transform,
};
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

    pub(super) fn command(&mut self, cmd: &DrawCmd) -> Result<()> {
        match cmd {
            DrawCmd::Rect { rect, fill } => self.rect(*rect, None, fill),
            DrawCmd::RoundedRect { rect, radius, fill } => self.rect(*rect, Some(*radius), fill),
            DrawCmd::StrokeRect {
                rect,
                width,
                color,
                radius,
            } => {
                let rect_attrs = rect_attributes(*rect, Some(*radius));
                self.body.push_str(&format!(
                    "<rect {rect_attrs} fill=\"none\" stroke=\"{}\"{} stroke-width=\"{}\"/>\n",
                    hex(*color),
                    opacity_attribute("stroke", *color),
                    number(*width)
                ));
                Ok(())
            }
            DrawCmd::Line {
                from,
                to,
                width,
                color,
            } => {
                self.body.push_str(&format!(
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\"{} stroke-width=\"{}\" stroke-linecap=\"butt\"/>\n",
                    number(from.x),
                    number(from.y),
                    number(to.x),
                    number(to.y),
                    hex(*color),
                    opacity_attribute("stroke", *color),
                    number(*width)
                ));
                Ok(())
            }
            DrawCmd::Shape(shape) => self.shape(shape),
            DrawCmd::Text(run) => self.text(run),
            DrawCmd::Paragraph { .. } => {
                Err(self.unsupported("a shaped paragraph (its text is not kept by the shaping)"))
            }
            DrawCmd::Pixels {
                image,
                rect,
                source,
                opacity,
                pixelated,
            } => self.pixels(image, *rect, *source, *opacity, *pixelated),
            DrawCmd::Image { source, rect } => self.drawn_image(
                DrawCmd::Image {
                    source: source.clone(),
                    rect: Rect {
                        x: 0.0,
                        y: 0.0,
                        width: rect.width,
                        height: rect.height,
                    },
                },
                *rect,
            ),
            DrawCmd::ImagePart {
                source,
                rect,
                crop,
                opacity,
                pixelated,
            } => self.drawn_image(
                DrawCmd::ImagePart {
                    source: source.clone(),
                    rect: Rect {
                        x: 0.0,
                        y: 0.0,
                        width: rect.width,
                        height: rect.height,
                    },
                    crop: *crop,
                    opacity: *opacity,
                    pixelated: *pixelated,
                },
                *rect,
            ),
            DrawCmd::External { .. } => Err(self.unsupported("an external GPU frame")),
            DrawCmd::PushLayer(layer) => self.push_layer(layer),
            DrawCmd::PopLayer | DrawCmd::PopTransform | DrawCmd::PopClip => self.pop(),
            DrawCmd::PushTransform(transform) => {
                self.push(&format!(" transform=\"{}\"", matrix(*transform)));
                Ok(())
            }
            DrawCmd::PushClip(rect) => {
                let id = self.id("clip");
                self.defs.push_str(&format!(
                    "<clipPath id=\"{id}\"><rect {}/></clipPath>\n",
                    rect_attributes(*rect, None)
                ));
                self.push(&format!(" clip-path=\"url(#{id})\""));
                Ok(())
            }
        }
    }

    /// Open a `<g>` with `attributes` (already formatted, with a leading space).
    fn push(&mut self, attributes: &str) {
        self.body.push_str(&format!("<g{attributes}>\n"));
        self.depth += 1;
    }

    fn pop(&mut self) -> Result<()> {
        if self.depth == 0 {
            return Err(UiError::new(
                ErrorKind::Message,
                "the drawing pops a layer, clip or transform it did not push",
                Some(self.at),
            ));
        }
        self.depth -= 1;
        self.body.push_str("</g>\n");
        Ok(())
    }

    fn push_layer(&mut self, layer: &DrawingLayer) -> Result<()> {
        if layer.blend != BlendMode::Normal {
            return Err(self.unsupported("a blend mode on a layer"));
        }
        if !layer.filter.is_empty() {
            return Err(self.unsupported("a filter on a layer"));
        }
        if layer.mask.is_some() {
            return Err(self.unsupported("a mask on a layer"));
        }
        let mut attributes = String::new();
        if layer.opacity != 1.0 {
            attributes.push_str(&format!(" opacity=\"{}\"", number(layer.opacity)));
        }
        if let Some(clip) = &layer.clip {
            let (cmds, rule, _) = self.outline_of(clip)?;
            let d = super::shape::path_data(&cmds, self.at)?;
            let id = self.id("clip");
            let clip_rule = match rule {
                kivact_types::FillRule::NonZero => "nonzero",
                kivact_types::FillRule::EvenOdd => "evenodd",
            };
            self.defs.push_str(&format!(
                "<clipPath id=\"{id}\"><path d=\"{d}\" clip-rule=\"{clip_rule}\"/></clipPath>\n"
            ));
            attributes.push_str(&format!(" clip-path=\"url(#{id})\""));
        }
        self.push(&attributes);
        Ok(())
    }

    fn rect(&mut self, rect: Rect, radius: Option<f64>, fill: &Fill) -> Result<()> {
        let paint = self.paint("fill", fill, rect)?;
        self.body.push_str(&format!(
            "<rect {} {paint}/>\n",
            rect_attributes(rect, radius)
        ));
        Ok(())
    }

    fn shape(&mut self, shape: &ShapePaint) -> Result<()> {
        let (cmds, rule, bounds) = self.outline_of(&shape.shape)?;
        let d = super::shape::path_data(&cmds, self.at)?;
        let rule_name = match rule {
            kivact_types::FillRule::NonZero => "nonzero",
            kivact_types::FillRule::EvenOdd => "evenodd",
        };
        let attributes = match &shape.stroke {
            None => {
                let paint = self.paint("fill", &shape.paint, bounds)?;
                format!("fill-rule=\"{rule_name}\" {paint}")
            }
            Some(stroke) => {
                let paint = self.paint("stroke", &shape.paint, bounds)?;
                format!("fill=\"none\" {paint} {}", stroke_attributes(stroke))
            }
        };
        self.body
            .push_str(&format!("<path d=\"{d}\" {attributes}/>\n"));
        Ok(())
    }

    fn paint(&mut self, kind: &str, fill: &Fill, bounds: Rect) -> Result<String> {
        paint_attributes(self, kind, fill, bounds)
    }
}

/// `x y width height` and the corner radius, if any.
fn rect_attributes(rect: Rect, radius: Option<f64>) -> String {
    let mut out = format!(
        "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\"",
        number(rect.x),
        number(rect.y),
        number(rect.width),
        number(rect.height)
    );
    if let Some(radius) = radius {
        out.push_str(&format!(" rx=\"{0}\" ry=\"{0}\"", number(radius)));
    }
    out
}

/// The SVG attributes of a stroke's width, caps, joins, miter limit and dashes.
pub(super) fn stroke_attributes(stroke: &Stroke) -> String {
    let cap = match stroke.cap {
        kivact_types::LineCap::Butt => "butt",
        kivact_types::LineCap::Round => "round",
        kivact_types::LineCap::Square => "square",
    };
    let join = match stroke.join {
        kivact_types::LineJoin::Miter => "miter",
        kivact_types::LineJoin::Round => "round",
        kivact_types::LineJoin::Bevel => "bevel",
    };
    let mut out = format!(
        "stroke-width=\"{}\" stroke-linecap=\"{cap}\" stroke-linejoin=\"{join}\" stroke-miterlimit=\"{}\"",
        number(stroke.width),
        number(stroke.miter_limit)
    );
    if !stroke.dash.is_empty() {
        let dashes: Vec<String> = stroke.dash.iter().map(|v| number(*v)).collect();
        out.push_str(&format!(
            " stroke-dasharray=\"{}\" stroke-dashoffset=\"{}\"",
            dashes.join(" "),
            number(stroke.dash_offset)
        ));
    }
    out
}

/// `stroke-opacity="…"` (or `fill-opacity`) for a color that is not opaque.
pub(super) fn opacity_attribute(kind: &str, color: Rgba) -> String {
    let alpha = (color.a.clamp(0.0, 1.0) * 255.0).round() / 255.0;
    if alpha == 1.0 {
        String::new()
    } else {
        format!(" {kind}-opacity=\"{}\"", number(alpha))
    }
}

/// `#rrggbb` of a color (its alpha is an opacity attribute).
pub(super) fn hex(color: Rgba) -> String {
    let channel = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(color.r),
        channel(color.g),
        channel(color.b)
    )
}

/// `matrix(a b c d e f)` of a transform.
fn matrix(t: Transform) -> String {
    format!(
        "matrix({} {} {} {} {} {})",
        number(t.a),
        number(t.b),
        number(t.c),
        number(t.d),
        number(t.e),
        number(t.f)
    )
}
