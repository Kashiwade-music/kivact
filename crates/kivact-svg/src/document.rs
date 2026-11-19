//! Role: Writes one recorded drawing as the text of an SVG 1.1 document.

use super::emit::Ended;
use super::emitter::Emitter;
use super::encode::number;
use crate::{Resources, SvgError};
use kivact_types::DrawList;

/// The SVG 1.1 document of `scene`, whose logical size is `width` × `height`.
/// The caller checks that the size is finite and positive. The document has
/// no background, and its fonts are named, not embedded.
pub fn write(
    scene: &DrawList,
    width: f64,
    height: f64,
    resources: &mut impl Resources,
) -> Result<String, SvgError> {
    let mut emitter = Emitter::new(resources);
    match emitter.commands(&mut scene.cmds.iter())? {
        Ended::End => Ok(document(&emitter, width, height)),
        Ended::Pop => Err(SvgError::Malformed(
            "the drawing pops a layer, clip or transform it did not push".to_owned(),
        )),
    }
}

/// The root element, then the definitions and the body.
fn document(emitter: &Emitter<'_>, width: f64, height: f64) -> String {
    let (w, h) = (number(width), number(height));
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n"
    ));
    if !emitter.defs.is_empty() {
        out.push_str("<defs>\n");
        out.push_str(&emitter.defs);
        out.push_str("</defs>\n");
    }
    out.push_str(&emitter.body);
    out.push_str("</svg>\n");
    out
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the whole document and of the nesting of its groups.

    use super::write;
    use crate::{Resources, SvgError};
    use kivact_types::{
        BackfaceVisibility, BlendMode, ColorInterpolation, Composite, DecorationLines, DrawCmd,
        DrawList, DrawingLayer, Fill, FillImage, FillLayer, FontFeature, FontKerning,
        FontVariation, Gradient, GradientKind, GradientStop, Mix, Pixels, Point, Rect, Rgba,
        SpaceColor, Stroke, TextDecoration, TextDecorationStyle, TextRun, TransformStyle,
    };

    /// A host with no images and no text: nothing a test below draws asks it.
    struct NoResources;

    impl Resources for NoResources {
        fn fallback_family(&self) -> Option<&str> {
            None
        }

        fn baseline(&self, _run: &TextRun) -> f64 {
            0.0
        }

        fn rasterize(
            &mut self,
            _cmd: DrawCmd,
            _width: u32,
            _height: u32,
        ) -> Result<Pixels, SvgError> {
            Err(SvgError::Resource("this test draws no image".to_owned()))
        }
    }

    fn scene(cmds: Vec<DrawCmd>) -> DrawList {
        DrawList {
            cmds,
            regions: Vec::new(),
        }
    }

    fn square() -> Rect {
        Rect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        }
    }

    fn filled() -> DrawCmd {
        DrawCmd::Rect {
            rect: square(),
            fill: Fill::Solid(Rgba {
                r: 1.0,
                g: 0.2,
                b: 0.0,
                a: 1.0,
            }),
        }
    }

    fn write_list(cmds: Vec<DrawCmd>) -> Result<String, SvgError> {
        write(&scene(cmds), 64.0, 32.0, &mut NoResources)
    }

    #[test]
    fn a_drawing_is_written_as_a_document_of_its_size() {
        let document = write_list(vec![filled()]).expect("a filled rectangle writes");

        assert!(document.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"));
        assert!(document.contains("width=\"64\" height=\"32\" viewBox=\"0 0 64 32\""));
        assert!(
            document.contains("<rect x=\"1\" y=\"2\" width=\"3\" height=\"4\" fill=\"#ff3300\"/>")
        );
        assert!(document.ends_with("</svg>\n"));
    }

    #[test]
    fn a_clip_group_defines_its_clip_path_once_and_closes_its_group() {
        let document = write_list(vec![
            DrawCmd::PushClip(square()),
            filled(),
            DrawCmd::PopClip,
        ])
        .expect("a clipped rectangle writes");

        assert!(document.contains("<defs>\n<clipPath id=\"clip1\">"));
        assert!(document.contains("<g clip-path=\"url(#clip1)\">\n<rect "));
        assert!(document.ends_with("</g>\n</svg>\n"));
    }

    #[test]
    fn nested_groups_close_in_the_order_they_opened() {
        let document = write_list(vec![
            DrawCmd::PushClip(square()),
            DrawCmd::PushTransform(kivact_types::Transform::IDENTITY),
            filled(),
            DrawCmd::PopTransform,
            DrawCmd::PopClip,
        ])
        .expect("nested groups write");

        let body = document
            .split_once("</defs>\n")
            .map_or(document.as_str(), |(_, rest)| rest);
        let groups: Vec<&str> = body
            .lines()
            .filter(|l| l.starts_with("<g") || *l == "</g>")
            .collect();
        assert_eq!(groups.len(), 4);
        assert!(groups[0].starts_with("<g clip-path="));
        assert!(groups[1].starts_with("<g transform="));
        assert_eq!(groups[2], "</g>");
        assert_eq!(groups[3], "</g>");
    }

    #[test]
    fn a_pop_without_a_push_is_malformed() {
        let error = write_list(vec![filled(), DrawCmd::PopLayer])
            .expect_err("a pop with no open group is refused");

        assert!(matches!(error, SvgError::Malformed(_)));
    }

    #[test]
    fn a_pop_after_its_group_closed_is_malformed() {
        let error = write_list(vec![
            DrawCmd::PushClip(square()),
            DrawCmd::PopClip,
            DrawCmd::PopClip,
        ])
        .expect_err("the second pop has no group");

        assert!(matches!(error, SvgError::Malformed(_)));
    }

    #[test]
    fn a_group_left_open_at_the_end_is_malformed() {
        let error = write_list(vec![DrawCmd::PushClip(square()), filled()])
            .expect_err("the group never closes");

        assert!(matches!(error, SvgError::Malformed(_)));
    }

    #[test]
    fn an_inner_group_left_open_is_malformed_even_when_the_outer_closes() {
        let error = write_list(vec![
            DrawCmd::PushClip(square()),
            DrawCmd::PushTransform(kivact_types::Transform::IDENTITY),
            DrawCmd::PopClip,
        ])
        .expect_err("a pop closes the innermost group, so the clip group stays open");

        assert!(matches!(error, SvgError::Malformed(_)));
    }

    /// A vertical linear gradient from red to blue, in oklab.
    fn linear_gradient_fill() -> Fill {
        let stop = |r: f64, b: f64| GradientStop {
            color: SpaceColor::from_rgba(Rgba {
                r,
                g: 0.0,
                b,
                a: 1.0,
            }),
            at: None,
        };
        let gradient = Gradient {
            kind: GradientKind::Linear { angle_deg: 90.0 },
            stops: vec![stop(1.0, 0.0), stop(0.0, 1.0)],
            hints: vec![None],
            repeating: false,
            interpolation: ColorInterpolation::DEFAULT,
        };
        Fill::Layers(vec![FillLayer::cover(FillImage::Gradient(gradient))])
    }

    /// A layer over the whole drawing with no clip, opacity or effect but `mix`.
    fn layer(mix: Mix) -> DrawingLayer {
        DrawingLayer {
            clip: None,
            area: square(),
            opacity: 1.0,
            mix,
            filter: Vec::new(),
            masks: Vec::new(),
            transform: None,
            transform_style: TransformStyle::default(),
            backface: BackfaceVisibility::default(),
        }
    }

    /// A run of `text` at the origin with no paint, font feature or decoration.
    fn text_run(text: &str) -> TextRun {
        TextRun {
            text: text.to_owned(),
            origin: Point::new(0.0, 0.0),
            font_size: 16.0,
            weight: 400,
            color: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            fill: None,
            stroke: None,
            family: None,
            italic: false,
            line_height: None,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            kerning: FontKerning::Auto,
            features: Vec::new(),
            variations: Vec::new(),
            font_width: 100.0,
            decoration: TextDecoration::default(),
        }
    }

    #[test]
    fn a_gradient_interpolates_in_oklab_like_the_window() {
        let document = write_list(vec![DrawCmd::Rect {
            rect: square(),
            fill: linear_gradient_fill(),
        }])
        .expect("a linear gradient writes");

        assert!(document.matches("<stop ").count() > 2);
    }

    #[test]
    fn a_blended_layer_writes_its_css_blend_mode() {
        let document = write_list(vec![
            DrawCmd::PushLayer(Box::new(layer(Mix::Blend(BlendMode::Multiply)))),
            filled(),
            DrawCmd::PopLayer,
        ])
        .expect("a blended layer writes");

        assert!(document.contains("mix-blend-mode:multiply"));
    }

    #[test]
    fn a_composited_layer_is_unsupported() {
        let error = write_list(vec![
            DrawCmd::PushLayer(Box::new(layer(Mix::Composite(Composite::DestinationOut)))),
            filled(),
            DrawCmd::PopLayer,
        ])
        .expect_err("a composite operation has no SVG 1.1 form");

        assert!(matches!(error, SvgError::Unsupported(_)));
    }

    #[test]
    fn text_writes_its_gradient_fill_and_stroke() {
        let mut run = text_run("Hi");
        run.fill = Some(linear_gradient_fill());
        run.stroke = Some((
            Stroke::new(2.0),
            Fill::Solid(Rgba {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            }),
        ));
        let document = write_list(vec![DrawCmd::Text(run)]).expect("gradient text writes");

        assert!(document.contains("fill=\"url(#"));
        assert!(document.contains("stroke-width=\"2\""));
        assert!(document.contains("gradientUnits=\"objectBoundingBox\""));
    }

    #[test]
    fn text_writes_features_variations_and_stretch() {
        let mut run = text_run("12:34");
        run.features = vec![FontFeature::new(*b"tnum", 1)];
        run.variations = vec![FontVariation {
            tag: *b"wght",
            value: 650.0,
        }];
        run.font_width = 75.0;
        let document = write_list(vec![DrawCmd::Text(run)]).expect("styled text writes");

        assert!(document.contains("font-feature-settings:'tnum' 1"));
        assert!(document.contains("font-variation-settings:'wght' 650"));
        assert!(document.contains("font-stretch=\"75%\""));
    }

    #[test]
    fn text_writes_its_decoration() {
        let mut run = text_run("Hi");
        run.decoration.lines = DecorationLines::UNDERLINE;
        run.decoration.style = TextDecorationStyle::Wavy;
        let document = write_list(vec![DrawCmd::Text(run)]).expect("decorated text writes");

        assert!(document.contains("text-decoration-line:underline"));
        assert!(document.contains("text-decoration-style:wavy"));
    }
}
