//! Role: Share Avatar sizes and silhouettes across the avatar family.

/// The diameter of an avatar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    /// 20-pixel diameter.
    Xsm,
    /// 24-pixel diameter.
    Sm,
    /// 36-pixel diameter, the default.
    #[default]
    Md,
    /// 48-pixel diameter.
    Lg,
    /// 128-pixel diameter.
    Xl,
    /// Requested diameter in pixels, snapped to the nearest supported size.
    Px(u16),
}

impl AvatarSize {
    pub(crate) fn side(self) -> f64 {
        const SIZES: [u16; 14] = [16, 20, 24, 32, 36, 40, 48, 60, 64, 72, 96, 128, 144, 180];
        let size = match self {
            Self::Xsm => 20,
            Self::Sm => 24,
            Self::Md => 36,
            Self::Lg => 48,
            Self::Xl => 128,
            Self::Px(size) => SIZES
                .into_iter()
                .min_by_key(|side| side.abs_diff(size))
                .unwrap_or(36),
        };
        f64::from(size)
    }
}

impl kivact::ui::Variant for AvatarSize {}
kivact::state::constant_value!(AvatarSize);
