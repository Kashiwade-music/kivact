//! Role: Style canonical empty-state content with Neutral spacing.
use kivact::state::{IntoValue, Value};
use kivact::style::{Align, Direction, Justify, TextAlign, Theme, pct, style};
use kivact::ui::{EmptyStateRecipes, EmptyStateState, Look};
/// Reactive visual density of an empty message, without announcement state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmptyStateVariant {
    compact: Value<bool>,
}
impl EmptyStateVariant {
    /// Stack actions and use the compact spacing scale.
    pub fn compact(compact: impl IntoValue<bool>) -> Self {
        Self {
            compact: compact.into_value(),
        }
    }
}
impl kivact::ui::Variant for EmptyStateVariant {}
fn compact(s: &Look<EmptyStateState>) -> bool {
    s.variant::<EmptyStateVariant>()
        .is_some_and(|variant| variant.compact.get())
}
pub(crate) fn empty_state(r: EmptyStateRecipes) -> EmptyStateRecipes {
    r.root(|_| {
        |theme: &Theme, s: &Look<EmptyStateState>| {
            let t = theme.ext::<crate::Tokens>();
            let (gap, x, y) = if compact(s) {
                (t.spacing.s2, t.spacing.s4, t.spacing.s4)
            } else {
                (t.spacing.s4, t.spacing.s6, t.spacing.s8)
            };
            style()
                .direction(Direction::Column)
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .width(pct(100))
                .min_width(0)
                .gap(gap)
                .padding_x(x)
                .padding_y(y)
        }
    })
    .title(|_| {
        |theme: &Theme, s: &Look<EmptyStateState>| {
            let t = theme.ext::<crate::Tokens>();
            let font = t.typography.large;
            style()
                .max_width(360)
                .min_width(0)
                .text_align(TextAlign::Center)
                .font_size(if compact(s) {
                    t.typography.size_md
                } else {
                    font.size_px
                })
                .line_height(font.line_height_px / font.size_px)
                .weight(font.weight)
                .color(t.colors.text_primary)
        }
    })
    .description(|_| {
        |theme: &Theme, s: &Look<EmptyStateState>| {
            let t = theme.ext::<crate::Tokens>();
            let font = t.typography.body;
            style()
                .max_width(360)
                .min_width(0)
                .text_align(TextAlign::Center)
                .margin_top(-if compact(s) {
                    t.spacing.s2
                } else {
                    t.spacing.s4
                })
                .font_size(if compact(s) {
                    t.typography.size_sm
                } else {
                    font.size_px
                })
                .line_height(font.line_height_px / font.size_px)
                .color(t.colors.text_secondary)
        }
    })
    .icon(|_| {
        |_: &Theme, _: &Look<EmptyStateState>| {
            style()
                .align_items(Align::Center)
                .justify_content(Justify::Center)
        }
    })
    .actions(|_| {
        |theme: &Theme, s: &Look<EmptyStateState>| {
            let t = theme.ext::<crate::Tokens>();
            style()
                .direction(if compact(s) {
                    Direction::Column
                } else if crate::stack::rtl() {
                    Direction::RowReverse
                } else {
                    Direction::Row
                })
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .gap(t.spacing.s2)
                .margin_top(t.spacing.s1)
        }
    })
}
