//! Role: Lowering of local scene text commands.
use super::state::State;
use crate::lower::context::Context;
use kivact_types::{DrawCmd, Fill, Point};
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

    /// A text run, shaped here.
    fn paint_run(&self, run: &kivact_types::TextRun, state: &mut State<'_>) {
        let tint = &state.current_tint;
        // Glyph fills other than one color are lowered by the text paint
        // of a later change; such a run is counted as dropped.
        let color = match &run.fill {
            None => run.color,
            Some(Fill::Solid(color)) => *color,
            Some(Fill::Layers(_)) => {
                self.drop_command();
                return;
            }
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
                    Some((stroke, Fill::Solid(color))) => Some(kivact_types::render::GlyphStroke {
                        stroke: stroke.clone(),
                        paint: kivact_types::render::Paint::Solid(tint.color(*color)),
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
                    Some(Fill::Solid(color)) => Some(tint.color(*color)),
                    Some(_) => {
                        // A non-solid fill is not drawn; it is counted.
                        self.drop_command();
                        None
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
        let Fill::Solid(color) = fill else {
            self.drop_command();
            return;
        };
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
        let color = state.current_tint.color(*color);
        let colors = crate::lower::glyphs::Colors {
            glyphs: Some(color),
            decorations: Some(color),
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
}
