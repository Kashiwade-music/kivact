//! Role: Application settings that cross the embedded UI thread boundary.
use crate::App;
use crate::i18n::{Locale, Messages};
use crate::platform::AppHandle;
use kivact_style::Theme;
use kivact_types::Size;
use std::panic::Location;

/// Canonical application settings that can cross the embedded UI thread boundary.
#[derive(Clone)]
pub(crate) struct AppSettings {
    pub(crate) title: String,
    /// `None`: the system's locale (`App::locale`).
    pub(crate) locale: Option<Locale>,
    pub(crate) messages: Messages,
    /// The window's first size (`window_size`); the engine's default when
    /// `None`.
    pub(crate) window_size: Option<Size>,
    /// The window's minimum size (`min_window_size`).
    pub(crate) min_window_size: Option<Size>,
    pub(crate) blocking_threads: Option<usize>,
    /// Where the App's theme was set (`App::theme`; `App::new` until then): the
    /// theme's fonts are located there when they fail to register.
    pub(crate) theme_at: &'static Location<'static>,
    /// How the window's GPU device is made (`gpu_preference`).
    pub(crate) gpu_preference: kivact_types::GpuPreference,
    /// The handle that ends this App's run from any thread (`App::handle`).
    pub(crate) handle: AppHandle,
}

impl AppSettings {
    pub(crate) fn with_theme(self, theme: Theme) -> App {
        App {
            theme: kivact_runtime::Prop::constant(theme),
            settings: self,
        }
    }
}
