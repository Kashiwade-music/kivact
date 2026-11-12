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
    use kivact_types::{DrawCmd, DrawList, Fill, Pixels, Rect, Rgba, TextRun};

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
}
