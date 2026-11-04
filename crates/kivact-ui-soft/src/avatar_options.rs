//! Role: Share Avatar sizes and silhouettes across the avatar family.
use crate::tokens;

/// The diameter of an avatar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarSize {
    Xsm,
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
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

/// The silhouette of an avatar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AvatarShape {
    #[default]
    Circle,
    Rounded,
    Square,
}

impl AvatarShape {
    pub(crate) fn radius(self) -> f64 {
        let r = &tokens().radius;
        match self {
            Self::Circle => r.full,
            Self::Rounded => r.element,
            Self::Square => r.none,
        }
    }
}
