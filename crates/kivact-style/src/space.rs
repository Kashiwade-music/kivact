//! Role: The core spacing variables of a theme (`space::MD`), in logical px.

use kivact_types::{Length, ThemeVar};

/// Declares each length variable (`X_VAR`, the `static` of `f64` px) and its
/// value form (`X`, the `const` `Length` a style takes).
macro_rules! length_vars {
    ($($(#[$doc:meta])* $var:ident / $value:ident = $name:literal, $fallback:expr;)*) => {$(
        $(#[$doc])*
        pub static $var: ThemeVar<f64> = ThemeVar::new($name, $fallback);
        $(#[$doc])*
        pub const $value: Length = Length::var(&$var);
    )*};
}
pub(crate) use length_vars;

length_vars! {
    /// The extra-small space.
    XS_VAR / XS = "space.xs", 2.0;
    /// The small space.
    SM_VAR / SM = "space.sm", 4.0;
    /// The medium space.
    MD_VAR / MD = "space.md", 8.0;
    /// The large space.
    LG_VAR / LG = "space.lg", 12.0;
    /// The extra-large space.
    XL_VAR / XL = "space.xl", 16.0;
}
