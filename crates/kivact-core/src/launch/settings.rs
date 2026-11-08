//! Role: Application configuration and startup validation.
use crate::i18n::{Locale, Messages};
use kivact_runtime::{ErrorKind, UiError};
use kivact_style::Theme;
use kivact_style::asset::Asset;
use kivact_types::Size;
use std::panic::Location;
/// Application configuration with a required title and optional startup settings.
#[derive(Clone)]
pub struct AppOptions {
    theme: kivact_runtime::Prop<Theme>,
    pub(super) settings: AppSettings,
}

// xtask-allow(state-fields): Independent application construction options do not encode runtime lifecycle.
/// Canonical application settings that can cross the embedded UI thread boundary.
#[derive(Clone)]
pub(crate) struct AppSettings {
    pub(super) title: String,
    /// `None`: the system's locale (`App::locale`).
    pub(super) locale: Option<Locale>,
    pub(super) messages: Messages,
    /// The window's first size (`window_size`); the engine's default when
    /// `None`.
    pub(super) window_size: Option<Size>,
    /// The window's minimum size (`min_window_size`).
    pub(super) min_window_size: Option<Size>,
    pub(super) blocking_threads: Option<usize>,
    pub(super) fonts: Vec<(Asset, &'static Location<'static>)>,
    /// How the window's GPU device is made (`gpu_preference`).
    pub(super) gpu_preference: kivact_types::GpuPreference,
    // xtask-allow(state-fields): setting - an independent opt-in a test sets to start the DevTools agent
    #[cfg(feature = "devtools")]
    pub(super) devtools_in_tests: bool,
}

impl AppOptions {
    /// An app whose window is titled `title`, with the default theme.
    pub fn new(title: impl Into<String>) -> AppOptions {
        AppOptions {
            theme: kivact_runtime::Prop::constant(Theme::default()),
            settings: AppSettings {
                title: title.into(),
                locale: None,
                messages: Messages::new(),
                window_size: None,
                min_window_size: None,
                blocking_threads: None,
                fonts: Vec::new(),
                gpu_preference: kivact_types::GpuPreference::default(),
                #[cfg(feature = "devtools")]
                devtools_in_tests: false,
            },
        }
    }

    /// Theme of the whole App, evaluated reactively under its root owner.
    /// Embedded startup accepts only a fixed theme; create reactive themes with
    /// `view::provide_theme` inside the embedded root factory.
    pub fn theme(mut self, theme: impl kivact_runtime::IntoValue<Theme>) -> Self {
        self.theme = theme.into_prop();
        self
    }

    /// The App's locale: the language and region components format for.
    /// Without it the App speaks the system's locale (`en-US` where the
    /// system names none) and follows it when the user changes it while
    /// the App runs; with it the App keeps this locale whatever the system
    /// says. To switch it at runtime, wrap the root in
    /// [`view::provide_locale`](crate::view::provide_locale) with a `Signal<Locale>`. Read it with
    /// [`i18n::locale`](crate::i18n::locale).
    pub fn locale(mut self, locale: Locale) -> Self {
        self.settings.locale = Some(locale);
        self
    }

    /// The App's texts: the replacements for the default (English) texts
    /// of components, by key (`"dialog.close"`). Each component lists its
    /// keys. To switch them at runtime, wrap the root in
    /// [`view::provide_messages`](crate::view::provide_messages) with a `Signal<Messages>`.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.settings.messages = messages;
        self
    }

    /// The window's first content size in logical pixels (default 960 by
    /// 720), e.g. `.window_size(1280, 800)`. The user can resize the
    /// window afterwards; read the current size with
    /// [`platform::window_size`](crate::platform::window_size) (to save it for the next start, say).
    /// Both values must be finite, at least one logical pixel, fit the native
    /// window extent, and not be smaller than
    /// [`crate::App::min_window_size`]: otherwise [`crate::App::run`] returns an error
    /// of kind `ErrorKind::Message` without opening the window.
    pub fn window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.settings.window_size = Some(Size::new(width.into(), height.into()));
        self
    }

    /// The smallest content size in logical pixels the user can resize the
    /// window to, e.g. `.min_window_size(480, 320)` (default: no minimum).
    /// Both values must be finite, at least one logical pixel, fit the native
    /// window extent, and not be larger than
    /// [`crate::App::window_size`] when that is set: otherwise [`crate::App::run`]
    /// returns an error of kind `ErrorKind::Message` without opening the
    /// window.
    pub fn min_window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.settings.min_window_size = Some(Size::new(width.into(), height.into()));
        self
    }

    /// Register the fonts of a TTF, OTF or TTC file before the first
    /// screen, so text can name their families with
    /// `style().font_family(..)` from the start (like CSS `@font-face`).
    /// Call it once per file; a family is the name written in the font,
    /// not the file name. Usually embedded:
    /// `.font(include_bytes!("../assets/Inter-Regular.ttf"))`. A file that
    /// is not a usable font (WOFF, unreadable) is reported as
    /// `ErrorKind::Asset`, located here; the App starts anyway.
    #[track_caller]
    pub fn font(mut self, font: impl Into<Asset>) -> Self {
        self.settings.fonts.push((font.into(), Location::caller()));
        self
    }

    /// What the App wishes of the window's GPU device (experimental,
    /// feature `gpu-interop`): the kind of adapter, the graphics API and
    /// the adapter itself (default: Arvane's own choice). A wish: the
    /// device is made on another adapter or API when the one asked for is
    /// not there, and the environment variables `WGPU_BACKEND` and
    /// `WGPU_POWER_PREF` override it. Read the adapter the device is on
    /// with [`gpu_interop::adapter`](crate::gpu_interop::adapter).
    #[cfg(feature = "gpu-interop")]
    pub fn gpu_preference(mut self, preference: crate::gpu_interop::GpuPreference) -> Self {
        self.settings.gpu_preference = preference;
        self
    }

    /// Size of the blocking pool (default `min(4, CPUs)`).
    pub fn blocking_threads(mut self, n: usize) -> Self {
        self.settings.blocking_threads = Some(n.max(1));
        self
    }

    // xtask-allow(internal-contract): kivact::test starts the DevTools agent for a test App
    #[cfg(feature = "devtools")]
    #[doc(hidden)]
    pub fn devtools_in_tests(mut self) -> Self {
        self.settings.devtools_in_tests = true;
        self
    }

    #[track_caller]
    pub fn check_window_sizes(&self) -> crate::Result<()> {
        crate::geometry::check_window_sizes(
            self.settings.window_size,
            self.settings.min_window_size,
            ("App::window_size", "App::min_window_size"),
            Location::caller(),
        )
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.settings.title = title.into();
        self
    }
    /// Initial engine settings, derived from the application options.
    pub fn host_options(&self) -> kivact_engine::api::HostOptions {
        let gpu_preference = self.settings.gpu_preference;
        kivact_engine::api::HostOptions {
            window_size: self.settings.window_size,
            gpu_preference,
        }
    }
    /// The single title used for both engine startup and later window effects.
    pub fn window_title(&self) -> &str {
        &self.settings.title
    }
    /// Extract only a fixed theme; reactive inputs stay on their owning thread.
    pub(crate) fn into_embed_settings(
        self,
        at: &'static Location<'static>,
    ) -> crate::Result<(AppSettings, Theme)> {
        let theme = self.theme.fixed().ok_or_else(|| {
            UiError::new(
                ErrorKind::Unsupported,
                "App::embed requires a fixed App theme; use view::provide_theme inside the Send root factory for an embedded reactive theme",
                Some(at),
            )
        })?;
        Ok((self.settings, theme))
    }

    pub(crate) fn into_view_settings(
        self,
    ) -> (
        kivact_runtime::Prop<Theme>,
        Option<crate::i18n::Locale>,
        Messages,
    ) {
        (self.theme, self.settings.locale, self.settings.messages)
    }
}

impl AppSettings {
    pub(crate) fn with_theme(self, theme: Theme) -> AppOptions {
        AppOptions {
            theme: kivact_runtime::Prop::constant(theme),
            settings: self,
        }
    }
}
