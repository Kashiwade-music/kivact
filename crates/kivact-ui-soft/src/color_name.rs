//! Role: Resolve named Soft semantic colors through the active theme.
use kivact::style::{Theme, ThemeColor};

/// Named scalar colors from [`crate::Colors`], following the active theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ColorName {
    /// [`crate::Colors::bg_body`].
    BgBody,
    /// [`crate::Colors::bg_surface`].
    BgSurface,
    /// [`crate::Colors::bg_card`].
    BgCard,
    /// [`crate::Colors::bg_popover`].
    BgPopover,
    /// [`crate::Colors::bg_muted`].
    BgMuted,
    /// [`crate::Colors::bg_inverted`].
    BgInverted,
    /// [`crate::Colors::bg_error_inverted`].
    BgErrorInverted,
    /// [`crate::Colors::neutral`].
    Neutral,
    /// [`crate::Colors::accent`].
    Accent,
    /// [`crate::Colors::accent_hover`].
    AccentHover,
    /// [`crate::Colors::choice_border_hover`].
    ChoiceBorderHover,
    /// [`crate::Colors::choice_surface_hover`].
    ChoiceSurfaceHover,
    /// [`crate::Colors::switch_off_hover`].
    SwitchOffHover,
    /// [`crate::Colors::accent_muted`].
    AccentMuted,
    /// [`crate::Colors::on_accent`].
    OnAccent,
    /// [`crate::Colors::on_dark`].
    OnDark,
    /// [`crate::Colors::on_light`].
    OnLight,
    /// [`crate::Colors::text_primary`].
    TextPrimary,
    /// [`crate::Colors::text_secondary`].
    TextSecondary,
    /// [`crate::Colors::text_disabled`].
    TextDisabled,
    /// [`crate::Colors::text_accent`].
    TextAccent,
    /// [`crate::Colors::icon_primary`].
    IconPrimary,
    /// [`crate::Colors::icon_secondary`].
    IconSecondary,
    /// [`crate::Colors::icon_disabled`].
    IconDisabled,
    /// [`crate::Colors::icon_accent`].
    IconAccent,
    /// [`crate::Colors::border`].
    Border,
    /// [`crate::Colors::border_emphasized`].
    BorderEmphasized,
    /// [`crate::Colors::skeleton`].
    Skeleton,
    /// [`crate::Colors::track`].
    Track,
    /// [`crate::Colors::shadow`].
    Shadow,
    /// [`crate::Colors::tint_hover`].
    TintHover,
    /// [`crate::Colors::overlay_scrim`].
    OverlayScrim,
    /// [`crate::Colors::overlay_hover`].
    OverlayHover,
    /// [`crate::Colors::overlay_pressed`].
    OverlayPressed,
    /// [`crate::Colors::focus_ring`].
    FocusRing,
    /// [`crate::Colors::info`].
    Info,
    /// [`crate::Colors::on_info`].
    OnInfo,
    /// [`crate::Colors::info_muted`].
    InfoMuted,
    /// [`crate::Colors::on_info_muted`].
    OnInfoMuted,
    /// [`crate::Colors::text_info`].
    TextInfo,
    /// [`crate::Colors::success`].
    Success,
    /// [`crate::Colors::on_success`].
    OnSuccess,
    /// [`crate::Colors::success_muted`].
    SuccessMuted,
    /// [`crate::Colors::on_success_muted`].
    OnSuccessMuted,
    /// [`crate::Colors::text_success`].
    TextSuccess,
    /// [`crate::Colors::warning`].
    Warning,
    /// [`crate::Colors::on_warning`].
    OnWarning,
    /// [`crate::Colors::warning_muted`].
    WarningMuted,
    /// [`crate::Colors::on_warning_muted`].
    OnWarningMuted,
    /// [`crate::Colors::text_warning`].
    TextWarning,
    /// [`crate::Colors::error`].
    Error,
    /// [`crate::Colors::on_error`].
    OnError,
    /// [`crate::Colors::error_muted`].
    ErrorMuted,
    /// [`crate::Colors::on_error_muted`].
    OnErrorMuted,
    /// [`crate::Colors::text_error`].
    TextError,
}

impl ThemeColor for ColorName {
    fn resolve(self, theme: &Theme) -> kivact::style::Color {
        let tokens = theme.ext::<crate::Tokens>();
        match self {
            Self::BgBody => tokens.colors.bg_body,
            Self::BgSurface => tokens.colors.bg_surface,
            Self::BgCard => tokens.colors.bg_card,
            Self::BgPopover => tokens.colors.bg_popover,
            Self::BgMuted => tokens.colors.bg_muted,
            Self::BgInverted => tokens.colors.bg_inverted,
            Self::BgErrorInverted => tokens.colors.bg_error_inverted,
            Self::Neutral => tokens.colors.neutral,
            Self::Accent => tokens.colors.accent,
            Self::AccentHover => tokens.colors.accent_hover,
            Self::ChoiceBorderHover => tokens.colors.choice_border_hover,
            Self::ChoiceSurfaceHover => tokens.colors.choice_surface_hover,
            Self::SwitchOffHover => tokens.colors.switch_off_hover,
            Self::AccentMuted => tokens.colors.accent_muted,
            Self::OnAccent => tokens.colors.on_accent,
            Self::OnDark => tokens.colors.on_dark,
            Self::OnLight => tokens.colors.on_light,
            Self::TextPrimary => tokens.colors.text_primary,
            Self::TextSecondary => tokens.colors.text_secondary,
            Self::TextDisabled => tokens.colors.text_disabled,
            Self::TextAccent => tokens.colors.text_accent,
            Self::IconPrimary => tokens.colors.icon_primary,
            Self::IconSecondary => tokens.colors.icon_secondary,
            Self::IconDisabled => tokens.colors.icon_disabled,
            Self::IconAccent => tokens.colors.icon_accent,
            Self::Border => tokens.colors.border,
            Self::BorderEmphasized => tokens.colors.border_emphasized,
            Self::Skeleton => tokens.colors.skeleton,
            Self::Track => tokens.colors.track,
            Self::Shadow => tokens.colors.shadow,
            Self::TintHover => tokens.colors.tint_hover,
            Self::OverlayScrim => tokens.colors.overlay_scrim,
            Self::OverlayHover => tokens.colors.overlay_hover,
            Self::OverlayPressed => tokens.colors.overlay_pressed,
            Self::FocusRing => tokens.colors.focus_ring,
            Self::Info => tokens.colors.info,
            Self::OnInfo => tokens.colors.on_info,
            Self::InfoMuted => tokens.colors.info_muted,
            Self::OnInfoMuted => tokens.colors.on_info_muted,
            Self::TextInfo => tokens.colors.text_info,
            Self::Success => tokens.colors.success,
            Self::OnSuccess => tokens.colors.on_success,
            Self::SuccessMuted => tokens.colors.success_muted,
            Self::OnSuccessMuted => tokens.colors.on_success_muted,
            Self::TextSuccess => tokens.colors.text_success,
            Self::Warning => tokens.colors.warning,
            Self::OnWarning => tokens.colors.on_warning,
            Self::WarningMuted => tokens.colors.warning_muted,
            Self::OnWarningMuted => tokens.colors.on_warning_muted,
            Self::TextWarning => tokens.colors.text_warning,
            Self::Error => tokens.colors.error,
            Self::OnError => tokens.colors.on_error,
            Self::ErrorMuted => tokens.colors.error_muted,
            Self::OnErrorMuted => tokens.colors.on_error_muted,
            Self::TextError => tokens.colors.text_error,
        }
    }
}
