//! Role: Lowering of local scene text commands.
use super::state::State;
use crate::lower::context::Context;
use crate::lower::effects::Tint;
use crate::lower::fill::gradient;
use kivact_types::render::{GlyphStroke, Paint};
use kivact_types::{DrawCmd, Fill, FillImage, FillLayer, Point, Rect};
impl Context<'_> {
    pub(super) fn paint_text(&self, cmd: &DrawCmd, state: &mut State<'_>) -> bool {
        match cmd {
            DrawCmd::Text(run) => self.paint_run(run, state),
            DrawCmd::Paragraph {
                shaped,
                origin,
                fill,
                ..
            } => self.paint_paragraph(shaped, *origin, fill, state),
            _ => return false,
        }
        true
    }

    /// A text run, shaped here. Its shaping takes its color from a solid fill
    /// (or from its own color); a gradient fill paints the glyphs instead.
    fn paint_run(&self, run: &kivact_types::TextRun, state: &mut State<'_>) {
        let tint = &state.current_tint;
        let color = match &run.fill {
            Some(Fill::Solid(color)) => *color,
            None | Some(Fill::Layers(_)) => run.color,
        };
        let mut style = crate::lower::glyphs::run_style(state.engine, run, state.inherited);
        style.color = tint.color(color);
        style.decoration.color = run.decoration.color.map(|c| tint.color(c));
        let origin = Point {
            x: state.origin.x + run.origin.x,
            y: state.origin.y + run.origin.y,
        };
        if !(style.font_size.is_finite() && origin.x.is_finite() && origin.y.is_finite())
            || style.font_size < 0.0
        {
            self.drop_command();
            return;
        }
        if style.font_size == 0.0 {
            // Valid, and draws nothing.
            return;
        }
        match state.engine.shape_glyphs(run.text.as_str(), &style, None) {
            Ok(shaped) => {
                let outline = match &run.stroke {
                    Some((stroke, Fill::Solid(color))) => Some(GlyphStroke {
                        stroke: stroke.clone(),
                        paint: Paint::Solid(tint.color(*color)),
                    }),
                    Some(_) => {
                        // A non-solid outline is not drawn; it is counted.
                        self.drop_command();
                        None
                    }
                    None => None,
                };
                let glyphs = match &run.fill {
                    None => None,
                    Some(Fill::Solid(color)) => Some(Paint::Solid(tint.color(*color))),
                    Some(Fill::Layers(layers)) => {
                        let bounds = Rect {
                            x: origin.x,
                            y: origin.y,
                            width: shaped.size.width,
                            height: shaped.size.height,
                        };
                        self.text_fill(layers, bounds, tint)
                    }
                };
                let colors = crate::lower::glyphs::Colors {
                    glyphs,
                    outline,
                    ..Default::default()
                };
                crate::lower::glyphs::push_text(
                    state.out,
                    &shaped,
                    origin,
                    &colors,
                    &[],
                    tint,
                    self.raster.get(),
                );
            }
            Err(error) => self.fail(error),
        }
    }

    /// A paragraph the client shaped: its glyph runs and decoration lines.
    fn paint_paragraph(
        &self,
        shaped: &kivact_types::ShapedTextSnapshot,
        origin: Point,
        fill: &Fill,
        state: &mut State<'_>,
    ) {
        let at = Point {
            x: state.origin.x + origin.x,
            y: state.origin.y + origin.y,
        };
        if !(at.x.is_finite() && at.y.is_finite()) {
            self.drop_command();
            return;
        }
        // The shaping carries no color of its own: the glyphs and
        // the decoration lines take the paragraph's.
        let tint = &state.current_tint;
        let bounds = Rect {
            x: at.x,
            y: at.y,
            width: shaped.size.width,
            height: shaped.size.height,
        };
        let (glyphs, decorations) = match fill {
            Fill::Solid(color) => {
                let color = tint.color(*color);
                (Some(Paint::Solid(color)), Some(color))
            }
            Fill::Layers(layers) => (self.text_fill(layers, bounds, tint), None),
        };
        let colors = crate::lower::glyphs::Colors {
            glyphs,
            decorations,
            ..Default::default()
        };
        crate::lower::glyphs::push_text(
            state.out,
            shaped,
            at,
            &colors,
            &[],
            &state.current_tint,
            self.raster.get(),
        );
    }

    /// The paint of glyphs filled with `layers`: one gradient layer over
    /// `bounds`, the text's box. Any other fill is not drawn and is
    /// counted as dropped.
    fn text_fill(&self, layers: &[FillLayer], bounds: Rect, tint: &Tint) -> Option<Paint> {
        let [
            FillLayer {
                image: FillImage::Gradient(g),
                ..
            },
        ] = layers
        else {
            self.drop_command();
            return None;
        };
        let paint = gradient(g, bounds);
        if paint.is_none() {
            self.drop_command();
        }
        paint.map(|paint| tint.paint(paint))
    }
}
