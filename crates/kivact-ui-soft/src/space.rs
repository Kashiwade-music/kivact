//! Role: The spacing scale of Astryx Neutral as theme variables, in logical px.
use kivact::style::{Length, Theme, ThemeVar};

/// The space `s0`: 0 px.
pub(crate) static SPACE_S0_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s0", 0.0);
/// The space `s0_5`: 2 px.
pub(crate) static SPACE_S0_5_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s0_5", 2.0);
/// The space `s1`: 4 px.
pub(crate) static SPACE_S1_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s1", 4.0);
/// The space `s1_5`: 6 px.
pub(crate) static SPACE_S1_5_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s1_5", 6.0);
/// The space `s2`: 8 px.
pub(crate) static SPACE_S2_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s2", 8.0);
/// The space `s3`: 12 px.
pub(crate) static SPACE_S3_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s3", 12.0);
/// The space `s4`: 16 px.
pub(crate) static SPACE_S4_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s4", 16.0);
/// The space `s5`: 20 px.
pub(crate) static SPACE_S5_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s5", 20.0);
/// The space `s6`: 24 px.
pub(crate) static SPACE_S6_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s6", 24.0);
/// The space `s7`: 28 px.
pub(crate) static SPACE_S7_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s7", 28.0);
/// The space `s8`: 32 px.
pub(crate) static SPACE_S8_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s8", 32.0);
/// The space `s9`: 36 px.
pub(crate) static SPACE_S9_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s9", 36.0);
/// The space `s10`: 40 px.
pub(crate) static SPACE_S10_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s10", 40.0);
/// The space `s11`: 44 px.
pub(crate) static SPACE_S11_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s11", 44.0);
/// The space `s12`: 48 px.
pub(crate) static SPACE_S12_VAR: ThemeVar<f64> = ThemeVar::new("soft.space_s12", 48.0);
/// The padding of a card: 12 px.
pub(crate) static SPACE_CARD_PADDING_VAR: ThemeVar<f64> =
    ThemeVar::new("soft.space_card_padding", 12.0);
/// The padding of a section: 12 px.
pub(crate) static SPACE_SECTION_PADDING_VAR: ThemeVar<f64> =
    ThemeVar::new("soft.space_section_padding", 12.0);
/// The padding of a dialog: 16 px.
pub(crate) static SPACE_DIALOG_PADDING_VAR: ThemeVar<f64> =
    ThemeVar::new("soft.space_dialog_padding", 16.0);
/// The gutter between an overlay and the edge of the viewport: 16 px.
pub(crate) static SPACE_LAYER_VIEWPORT_GUTTER_VAR: ThemeVar<f64> =
    ThemeVar::new("soft.space_layer_viewport_gutter", 16.0);

/// The space `s0`.
pub const SPACE_S0: Length = Length::var(&SPACE_S0_VAR);
/// The space `s0_5`.
pub const SPACE_S0_5: Length = Length::var(&SPACE_S0_5_VAR);
/// The space `s1`.
pub const SPACE_S1: Length = Length::var(&SPACE_S1_VAR);
/// The space `s1_5`.
pub const SPACE_S1_5: Length = Length::var(&SPACE_S1_5_VAR);
/// The space `s2`.
pub const SPACE_S2: Length = Length::var(&SPACE_S2_VAR);
/// The space `s3`.
pub const SPACE_S3: Length = Length::var(&SPACE_S3_VAR);
/// The space `s4`.
pub const SPACE_S4: Length = Length::var(&SPACE_S4_VAR);
/// The space `s5`.
pub const SPACE_S5: Length = Length::var(&SPACE_S5_VAR);
/// The space `s6`.
pub const SPACE_S6: Length = Length::var(&SPACE_S6_VAR);
/// The space `s7`.
pub const SPACE_S7: Length = Length::var(&SPACE_S7_VAR);
/// The space `s8`.
pub const SPACE_S8: Length = Length::var(&SPACE_S8_VAR);
/// The space `s9`.
pub const SPACE_S9: Length = Length::var(&SPACE_S9_VAR);
/// The space `s10`.
pub const SPACE_S10: Length = Length::var(&SPACE_S10_VAR);
/// The space `s11`.
pub const SPACE_S11: Length = Length::var(&SPACE_S11_VAR);
/// The space `s12`.
pub const SPACE_S12: Length = Length::var(&SPACE_S12_VAR);
/// The padding of a card.
pub const SPACE_CARD_PADDING: Length = Length::var(&SPACE_CARD_PADDING_VAR);
/// The padding of a section.
pub const SPACE_SECTION_PADDING: Length = Length::var(&SPACE_SECTION_PADDING_VAR);
/// The padding of a dialog.
pub const SPACE_DIALOG_PADDING: Length = Length::var(&SPACE_DIALOG_PADDING_VAR);
/// The gutter between an overlay and the edge of the viewport.
pub const SPACE_LAYER_VIEWPORT_GUTTER: Length = Length::var(&SPACE_LAYER_VIEWPORT_GUTTER_VAR);

/// Set every space variable of the kit. The scale does not depend on the appearance.
pub(crate) fn set_space(theme: Theme) -> Theme {
    theme
        .set(&SPACE_S0_VAR, 0.0)
        .set(&SPACE_S0_5_VAR, 2.0)
        .set(&SPACE_S1_VAR, 4.0)
        .set(&SPACE_S1_5_VAR, 6.0)
        .set(&SPACE_S2_VAR, 8.0)
        .set(&SPACE_S3_VAR, 12.0)
        .set(&SPACE_S4_VAR, 16.0)
        .set(&SPACE_S5_VAR, 20.0)
        .set(&SPACE_S6_VAR, 24.0)
        .set(&SPACE_S7_VAR, 28.0)
        .set(&SPACE_S8_VAR, 32.0)
        .set(&SPACE_S9_VAR, 36.0)
        .set(&SPACE_S10_VAR, 40.0)
        .set(&SPACE_S11_VAR, 44.0)
        .set(&SPACE_S12_VAR, 48.0)
        .set(&SPACE_CARD_PADDING_VAR, 12.0)
        .set(&SPACE_SECTION_PADDING_VAR, 12.0)
        .set(&SPACE_DIALOG_PADDING_VAR, 16.0)
        .set(&SPACE_LAYER_VIEWPORT_GUTTER_VAR, 16.0)
}
