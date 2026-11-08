//! Role: The environment emulation of a target.

use serde::{Deserialize, Serialize};

/// An emulated theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeEmulation {
    /// The light theme.
    Light,
    /// The dark theme.
    Dark,
}

/// An emulated pseudo-locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PseudoLocale {
    /// No pseudo-locale.
    Off,
    /// Accented letters.
    Accented,
    /// Accented letters with padding.
    Long,
    /// Right-to-left text.
    Rtl,
}

/// The environment emulation; every `None` leaves the setting alone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Emulation {
    /// The theme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemeEmulation>,
    /// Whether reduced motion is on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reduced_motion: Option<bool>,
    /// The locale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// The pseudo-locale.
    pub pseudo_locale: PseudoLocale,
    /// The DPI scale factor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    /// Whether the device has touch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub touch: Option<bool>,
}
