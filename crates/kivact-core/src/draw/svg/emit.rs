//! Role: Turns each recorded draw command into SVG markup, in order.
//!
//! Every push opens a `<g>` and every pop closes one, so the markup nests
//! exactly as the recorded stack does. Definitions (clip paths and
//! gradients) go to one `<defs>` block, numbered in the order they are
//! written.

use super::emitter::Emitter;
use super::encode::{hex, number, opacity_attribute};
use super::fill::paint_attributes;
use crate::{Result, UiError};
use kivact_runtime::ErrorKind;
use kivact_types::{
    BlendMode, DrawCmd, DrawingLayer, Fill, Point, Rect, Rgba, ShapePaint, Stroke, Transform,
};

impl Emitter<'_> {
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
                self.stroke_rect(*rect, *width, *color, *radius);
                Ok(())
            }
            DrawCmd::Line {
                from,
                to,
                width,
                color,
            } => {
                self.line(*from, *to, *width, *color);
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
                    rect: at_origin(*rect),
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
                    rect: at_origin(*rect),
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
                self.push_clip(*rect);
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

    /// A clip to `rect`, as a clip path defined once and opened as a group.
    fn push_clip(&mut self, rect: Rect) {
        let id = self.id("clip");
        self.defs.push_str(&format!(
            "<clipPath id=\"{id}\"><rect {}/></clipPath>\n",
            rect_attributes(rect, None)
        ));
        self.push(&format!(" clip-path=\"url(#{id})\""));
    }

    fn stroke_rect(&mut self, rect: Rect, width: f64, color: Rgba, radius: f64) {
        self.body.push_str(&format!(
            "<rect {} fill=\"none\" stroke=\"{}\"{} stroke-width=\"{}\"/>\n",
            rect_attributes(rect, Some(radius)),
            hex(color),
            opacity_attribute("stroke", color),
            number(width)
        ));
    }

    fn line(&mut self, from: Point, to: Point, width: f64, color: Rgba) {
        self.body.push_str(&format!(
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\"{} stroke-width=\"{}\" stroke-linecap=\"butt\"/>\n",
            number(from.x),
            number(from.y),
            number(to.x),
            number(to.y),
            hex(color),
            opacity_attribute("stroke", color),
            number(width)
        ));
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

/// `rect` moved to the origin, keeping its size: where an asset is drawn.
fn at_origin(rect: Rect) -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        width: rect.width,
        height: rect.height,
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
