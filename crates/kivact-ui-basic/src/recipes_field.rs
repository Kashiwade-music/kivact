//! Role: Theme appearance for standard field labels, hints and errors.
use kivact::style::{Direction, pct, style};
use kivact::ui::FieldRecipes;

pub(crate) fn field(table: FieldRecipes) -> FieldRecipes {
    table
        .root(|_| {
            |_, _| {
                style()
                    .direction(Direction::Column)
                    .width(pct(100))
                    .min_width(0)
                    .gap(8)
            }
        })
        .label(|_| {
            |theme, state| {
                style()
                    .font_size(12)
                    .weight(500)
                    .line_height(1.)
                    .color(if state.invalid {
                        theme.get::<crate::tokens::Tokens>().destructive
                    } else {
                        theme.colors().text
                    })
            }
        })
        .hint(|_| |theme, _| style().font_size(12).color(theme.colors().text_muted))
        .error(|_| {
            |theme, _| {
                style()
                    .font_size(12)
                    .color(theme.get::<crate::tokens::Tokens>().destructive)
            }
        })
}
