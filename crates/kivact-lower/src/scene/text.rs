//! Role: Lowering of local scene text commands.
use super::state::State;
use crate::context::Context;
use kivact_types::{Point, SceneCmd};
impl Context<'_> {
    pub(super) fn paint_text(&self, cmd: &SceneCmd, state: &mut State<'_>) -> bool {
        let origin = state.origin;
        let point = |p: Point| Point {
            x: origin.x + p.x,
            y: origin.y + p.y,
        };
        let tint = &state.current_tint;
        match cmd {
            SceneCmd::Text(run) => {
                let mut style = crate::glyphs::run_style(state.engine, run, state.inherited);
                style.color = tint.color(run.color);
                style.decoration.color = run.decoration.color.map(|c| tint.color(c));
                let origin = point(run.origin);
                if !(style.font_size.is_finite() && origin.x.is_finite() && origin.y.is_finite())
                    || style.font_size < 0.0
                {
                    self.drop_command();
                    return true;
                }
                if style.font_size == 0.0 {
                    // Valid, and draws nothing.
                    return true;
                }
                match state.engine.shape_glyphs(run.text.as_str(), &style, None) {
                    Ok(shaped) => {
                        let colors = crate::glyphs::Colors {
                            glyphs: None,
                            decorations: None,
                        };
                        crate::glyphs::push_text(
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
                true
            }
            SceneCmd::Paragraph {
                shaped,
                origin: at,
                color,
            } => {
                let at = point(*at);
                if !(at.x.is_finite() && at.y.is_finite()) {
                    self.drop_command();
                    return true;
                }
                // The shaping carries no color of its own: the glyphs and
                // the decoration lines take the paragraph's.
                let color = tint.color(*color);
                let colors = crate::glyphs::Colors {
                    glyphs: Some(color),
                    decorations: Some(color),
                };
                crate::glyphs::push_text(
                    state.out,
                    shaped,
                    at,
                    &colors,
                    &[],
                    tint,
                    self.raster.get(),
                );
                true
            }

            _ => false,
        }
    }
}
