//! Role: Style canonical native file drop targets as Neutral fields or large targets.
use crate::Tokens;
use kivact::custom::PartState;
use kivact::interaction::{DropzoneState, Validity};
use kivact::style::{Align, Direction, Justify, Theme, pct, style};
use kivact::ui::{DropzoneRecipes, Look};
/// Visual chrome of a standard drop target, independent of file validation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DropzoneVariant {
    /// Use the compact field treatment for the drop target.
    Field,
    #[default]
    /// Use the larger drop-area treatment.
    Area,
}
impl kivact::ui::Variant for DropzoneVariant {}
fn field(s: &Look<DropzoneState>) -> bool {
    s.variant::<DropzoneVariant>() == Some(&DropzoneVariant::Field)
}
pub(crate) fn dropzone(r: DropzoneRecipes) -> DropzoneRecipes {
    r.root(|_| {
        |theme: &Theme, s: &Look<DropzoneState>| {
            let t = theme.ext::<Tokens>();
            let invalid = matches!(
                &**s,
                DropzoneState::Failed(_)
                    | DropzoneState::DragOver {
                        valid: Validity::Invalid(_)
                    }
            );
            let base = crate::recipes_field::frame(&t, &PartState::default(), invalid)
                .direction(if field(s) {
                    Direction::Row
                } else {
                    Direction::Column
                })
                .align_items(Align::Center)
                .justify_content(Justify::Center)
                .min_width(0)
                .width(pct(100))
                .min_height(if field(s) { 32.0 } else { 96.0 })
                .gap(t.spacing.s2)
                .padding_x(t.spacing.s2)
                .padding_y(if field(s) { 0.0 } else { t.spacing.s3 });
            match &**s {
                DropzoneState::DragOver { .. } => base
                    .border_color(t.colors.accent)
                    .shadow(t.shadows.input_focus_ring),
                DropzoneState::Picking => base.opacity(t.sizes.disabled_opacity),
                _ => base,
            }
        }
    })
    .label(|_| {
        |theme: &Theme, _: &Look<DropzoneState>| {
            let t = theme.ext::<Tokens>();
            crate::recipes_field::text(
                style().min_width(0).single_line(),
                t.typography.body,
                t.colors.text_primary,
            )
        }
    })
    .hint(|_| {
        |theme: &Theme, s: &Look<DropzoneState>| {
            let t = theme.ext::<Tokens>();
            crate::recipes_field::text(
                style().min_width(0).single_line(),
                if field(s) {
                    t.typography.body
                } else {
                    t.typography.supporting
                },
                if field(s) {
                    t.colors.text_primary
                } else {
                    t.colors.text_secondary
                },
            )
        }
    })
    .status(|_| {
        |theme: &Theme, _: &Look<DropzoneState>| {
            let t = theme.ext::<Tokens>();
            crate::recipes_field::text(style(), t.typography.supporting, t.colors.text_error)
        }
    })
    .icon(|_| {
        |theme: &Theme, _: &Look<DropzoneState>| {
            let t = theme.ext::<Tokens>();
            style()
                .size(t.sizes.icon_sm, t.sizes.icon_sm)
                .shrink(0)
                .color(t.colors.icon_secondary)
        }
    })
}
