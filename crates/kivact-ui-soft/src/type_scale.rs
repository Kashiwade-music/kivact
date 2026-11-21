//! Role: The font families, weights, sizes and typography roles of Astryx Neutral (crate-private).

/// A typography role's size, line height, weight and family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TypeStyle {
    /// Font size in logical pixels.
    pub size_px: f64,
    /// Line height in logical pixels.
    pub line_height_px: f64,
    /// Numeric font weight.
    pub weight: u16,
    /// Font-family fallback list used to shape this role.
    pub family: &'static str,
}

pub(crate) const FAMILY_BODY: &str = "Figtree, sans-serif";
pub(crate) const FAMILY_HEADING: &str = "Figtree, sans-serif";
pub(crate) const FAMILY_CODE: &str =
    "ui-monospace, SF Mono, Monaco, Consolas, Liberation Mono, Courier New, monospace";
pub(crate) const WEIGHT_NORMAL: u16 = 400;
pub(crate) const WEIGHT_MEDIUM: u16 = 500;
pub(crate) const WEIGHT_SEMIBOLD: u16 = 600;
pub(crate) const WEIGHT_BOLD: u16 = 700;
pub(crate) const SIZE_4XS: f64 = 6.0;
pub(crate) const SIZE_3XS: f64 = 7.0;
pub(crate) const SIZE_2XS: f64 = 8.0;
pub(crate) const SIZE_XS: f64 = 10.0;
pub(crate) const SIZE_SM: f64 = 12.0;
pub(crate) const SIZE_MD: f64 = 14.0;
pub(crate) const SIZE_LG: f64 = 17.0;
pub(crate) const SIZE_XL: f64 = 20.0;
pub(crate) const SIZE_2XL: f64 = 24.0;
pub(crate) const SIZE_3XL: f64 = 29.0;
pub(crate) const SIZE_4XL: f64 = 35.0;

pub(crate) const DISPLAY1: TypeStyle = TypeStyle {
    size_px: 42.0,
    line_height_px: 52.0,
    weight: 400,
    family: FAMILY_BODY,
};
pub(crate) const DISPLAY2: TypeStyle = TypeStyle {
    size_px: 35.0,
    line_height_px: 44.0,
    weight: 400,
    family: FAMILY_BODY,
};
pub(crate) const DISPLAY3: TypeStyle = TypeStyle {
    size_px: 29.0,
    line_height_px: 40.0,
    weight: 400,
    family: FAMILY_BODY,
};
pub(crate) const HEADING1: TypeStyle = TypeStyle {
    size_px: 24.0,
    line_height_px: 32.0,
    weight: 600,
    family: FAMILY_HEADING,
};
pub(crate) const HEADING2: TypeStyle = TypeStyle {
    size_px: 20.0,
    line_height_px: 28.0,
    weight: 600,
    family: FAMILY_HEADING,
};
pub(crate) const HEADING3: TypeStyle = TypeStyle {
    size_px: 17.0,
    line_height_px: 24.0,
    weight: 700,
    family: FAMILY_HEADING,
};
pub(crate) const HEADING4: TypeStyle = TypeStyle {
    size_px: 14.0,
    line_height_px: 20.0,
    weight: 700,
    family: FAMILY_HEADING,
};
pub(crate) const HEADING5: TypeStyle = TypeStyle {
    size_px: 12.0,
    line_height_px: 20.0,
    weight: 600,
    family: FAMILY_HEADING,
};
pub(crate) const HEADING6: TypeStyle = TypeStyle {
    size_px: 10.0,
    line_height_px: 16.0,
    weight: 600,
    family: FAMILY_HEADING,
};
pub(crate) const LARGE: TypeStyle = TypeStyle {
    size_px: 17.0,
    line_height_px: 24.0,
    weight: 600,
    family: FAMILY_BODY,
};
pub(crate) const BODY: TypeStyle = TypeStyle {
    size_px: 14.0,
    line_height_px: 20.0,
    weight: 400,
    family: FAMILY_BODY,
};
pub(crate) const LABEL: TypeStyle = TypeStyle {
    size_px: 14.0,
    line_height_px: 20.0,
    weight: 500,
    family: FAMILY_BODY,
};
pub(crate) const CODE: TypeStyle = TypeStyle {
    size_px: 14.0,
    line_height_px: 20.0,
    weight: 400,
    family: FAMILY_CODE,
};
pub(crate) const SUPPORTING: TypeStyle = TypeStyle {
    size_px: 12.0,
    line_height_px: 20.0,
    weight: 400,
    family: FAMILY_BODY,
};
