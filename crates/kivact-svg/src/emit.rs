//! Role: Turns each recorded draw command into SVG markup, in order.
//!
//! A push opens a `<g>`; the commands up to its matching pop are its body, so
//! the markup nests as the recorded stack does. The nesting is the recursion
//! of `commands`: the call that reads a push writes its group and returns at
//! the pop that closes it. A pop with no group, or a list that ends inside a
//! group, is `Malformed`. Definitions (clip paths and gradients) go to one
//! `<defs>` block, numbered in the order they are written.

use super::emitter::Emitter;
use super::encode::{hex, number, opacity_attribute};
use super::fill::paint_attributes;
use super::shape::{outline_of, path_data};
use crate::SvgError;
use kivact_types::{
    BlendMode, DrawCmd, DrawingLayer, DrawingShape, Fill, FillRule, Point, Rect, Rgba, ShapePaint,
    Stroke, Transform,
};

/// How a run of commands ended.
pub(super) enum Ended {
    /// At a pop, which closes the group that the run belongs to.
    Pop,
    /// At the end of the list.
    End,
}

/// What one command does to the nesting of the markup.
enum Step {
    /// The command is complete markup by itself.
    Done,
    /// The command opens a group with these attributes.
    Open(String),
    /// The command closes the innermost open group.
    Close,
}

impl Emitter<'_> {
    /// Writes `cmds` up to the end of the list, or up to the pop that closes
    /// the group they belong to, whichever comes first.
    pub(super) fn commands(
        &mut self,
        cmds: &mut std::slice::Iter<'_, DrawCmd>,
    ) -> Result<Ended, SvgError> {
        while let Some(cmd) = cmds.next() {
            match self.step(cmd)? {
                Step::Done => {}
                Step::Open(attributes) => self.group(&attributes, cmds)?,
                Step::Close => return Ok(Ended::Pop),
            }
        }
        Ok(Ended::End)
    }

    /// Writes a `<g>` with `attributes`, then the commands up to its pop.
    fn group(
        &mut self,
        attributes: &str,
        cmds: &mut std::slice::Iter<'_, DrawCmd>,
    ) -> Result<(), SvgError> {
        self.body.push_str(&format!("<g{attributes}>\n"));
        match self.commands(cmds)? {
            Ended::Pop => {
                self.body.push_str("</g>\n");
                Ok(())
            }
            Ended::End => Err(SvgError::Malformed(
                "the drawing leaves a layer, clip or transform open".to_owned(),
            )),
        }
    }

    /// Writes one command and reports how it changes the nesting.
    fn step(&mut self, cmd: &DrawCmd) -> Result<Step, SvgError> {
        match cmd {
            DrawCmd::Rect { rect, fill } => self.rect(*rect, None, fill).map(|()| Step::Done),
            DrawCmd::RoundedRect { rect, radius, fill } => {
                self.rect(*rect, Some(*radius), fill).map(|()| Step::Done)
            }
            DrawCmd::StrokeRect {
                rect,
                width,
                color,
                radius,
            } => {
                self.stroke_rect(*rect, *width, *color, *radius);
                Ok(Step::Done)
            }
            DrawCmd::Line {
                from,
                to,
                width,
                color,
            } => {
                self.line(*from, *to, *width, *color);
                Ok(Step::Done)
            }
            DrawCmd::Shape(shape) => self.shape(shape).map(|()| Step::Done),
            DrawCmd::Text(run) => self.text(run).map(|()| Step::Done),
            DrawCmd::Paragraph { .. } => Err(SvgError::unsupported(
                "a shaped paragraph (its text is not kept by the shaping)",
            )),
            DrawCmd::Pixels {
                image,
                rect,
                source,
                opacity,
                pixelated,
            } => self
                .pixels(image, *rect, *source, *opacity, *pixelated)
                .map(|()| Step::Done),
            DrawCmd::Image { source, rect } => self
                .drawn_image(
                    DrawCmd::Image {
                        source: source.clone(),
                        rect: at_origin(*rect),
                    },
                    *rect,
                )
                .map(|()| Step::Done),
            DrawCmd::ImagePart {
                source,
                rect,
                crop,
                opacity,
                pixelated,
            } => self
                .drawn_image(
                    DrawCmd::ImagePart {
                        source: source.clone(),
                        rect: at_origin(*rect),
                        crop: *crop,
                        opacity: *opacity,
                        pixelated: *pixelated,
                    },
                    *rect,
                )
                .map(|()| Step::Done),
            DrawCmd::External { .. } => Err(SvgError::unsupported("an external GPU frame")),
            DrawCmd::Drawable { .. } => Err(SvgError::unsupported("a drawable's subtree")),
            DrawCmd::BorderImage { .. } => Err(SvgError::unsupported("a border image")),
            DrawCmd::PushLayer(layer) => self.layer_attributes(layer).map(Step::Open),
            DrawCmd::PopLayer | DrawCmd::PopTransform | DrawCmd::PopClip => Ok(Step::Close),
            DrawCmd::PushTransform(transform) => {
                Ok(Step::Open(format!(" transform=\"{}\"", matrix(*transform))))
            }
            DrawCmd::PushClip(rect) => Ok(Step::Open(self.rect_clip(*rect))),
        }
    }

    /// A clip to `rect`, defined once as a clip path; the attributes of its group.
    fn rect_clip(&mut self, rect: Rect) -> String {
        let id = self.id("clip");
        self.defs.push_str(&format!(
            "<clipPath id=\"{id}\"><rect {}/></clipPath>\n",
            rect_attributes(rect, None)
        ));
        format!(" clip-path=\"url(#{id})\"")
    }

    /// A clip to a drawn outline, defined once as a clip path; the attributes of its group.
    fn outline_clip(&mut self, clip: &DrawingShape) -> Result<String, SvgError> {
        let (cmds, rule, _) = outline_of(clip);
        let d = path_data(&cmds)?;
        let id = self.id("clip");
        self.defs.push_str(&format!(
            "<clipPath id=\"{id}\"><path d=\"{d}\" clip-rule=\"{}\"/></clipPath>\n",
            rule_name(rule)
        ));
        Ok(format!(" clip-path=\"url(#{id})\""))
    }

    /// The attributes of the group of a layer, which the SVG form can express
    /// only as opacity and a clip.
    fn layer_attributes(&mut self, layer: &DrawingLayer) -> Result<String, SvgError> {
        if layer.blend != BlendMode::Normal {
            return Err(SvgError::unsupported("a blend mode on a layer"));
        }
        if !layer.filter.is_empty() {
            return Err(SvgError::unsupported("a filter on a layer"));
        }
        if !layer.masks.is_empty() {
            return Err(SvgError::unsupported("a mask on a layer"));
        }
        if layer.transform.is_some() {
            return Err(SvgError::unsupported("a 3D transform on a layer"));
        }
        let mut attributes = String::new();
        if layer.opacity != 1.0 {
            attributes.push_str(&format!(" opacity=\"{}\"", number(layer.opacity)));
        }
        if let Some(clip) = &layer.clip {
            attributes.push_str(&self.outline_clip(clip)?);
        }
        Ok(attributes)
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

    fn rect(&mut self, rect: Rect, radius: Option<f64>, fill: &Fill) -> Result<(), SvgError> {
        let paint = self.paint("fill", fill, rect)?;
        self.body.push_str(&format!(
            "<rect {} {paint}/>\n",
            rect_attributes(rect, radius)
        ));
        Ok(())
    }

    fn shape(&mut self, shape: &ShapePaint) -> Result<(), SvgError> {
        let (cmds, rule, bounds) = outline_of(&shape.shape);
        let d = path_data(&cmds)?;
        let attributes = match &shape.stroke {
            None => {
                let paint = self.paint("fill", &shape.paint, bounds)?;
                format!("fill-rule=\"{}\" {paint}", rule_name(rule))
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

    fn paint(&mut self, kind: &str, fill: &Fill, bounds: Rect) -> Result<String, SvgError> {
        paint_attributes(self, kind, fill, bounds)
    }
}

/// The SVG name of a fill rule.
fn rule_name(rule: FillRule) -> &'static str {
    match rule {
        FillRule::NonZero => "nonzero",
        FillRule::EvenOdd => "evenodd",
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
fn stroke_attributes(stroke: &Stroke) -> String {
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
