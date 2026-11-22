//! Role: Application settings that cross the embedded UI thread boundary.
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

/// What a launch takes from an App: its theme and its settings. `App` turns
/// into it (`App::into_launch`), so the launch never names `App`.
pub(crate) struct LaunchApp {
    pub(crate) theme: kivact_runtime::Prop<Theme>,
    pub(crate) settings: AppSettings,
}

impl LaunchApp {
    /// The handle that ends this App's run from any thread.
    pub(crate) fn handle(&self) -> AppHandle {
        self.settings.handle.clone()
    }

    pub(crate) fn into_view_settings(
        self,
    ) -> (
        kivact_runtime::Prop<Theme>,
        Option<Locale>,
        Messages,
        &'static Location<'static>,
    ) {
        (
            self.theme,
            self.settings.locale,
            self.settings.messages,
            self.settings.theme_at,
        )
    }
}
