//! Role: Public application builder and native window entry.
use crate::Result;
use crate::i18n::{Locale, Messages};
use crate::launch::{AppSettings, LaunchApp, LaunchPolicy};
use crate::platform::{AppHandle, Begin, GpuPreference};
use crate::view::IntoView;
use std::panic::Location;

use kivact_runtime::{ErrorKind, UiError};
use kivact_style::Theme;
use kivact_types::Size;

/// The App: its window, theme, locale, texts and run. Build it with
/// `App::new(title)` and the setters, then `run` it once.
pub struct App {
    pub(crate) theme: kivact_runtime::Prop<Theme>,
    pub(crate) settings: AppSettings,
}

impl App {
    /// An app whose window is titled `title`, with the default theme.
    #[track_caller]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            theme: kivact_runtime::Prop::constant(Theme::default()),
            settings: AppSettings {
                title: title.into(),
                locale: None,
                messages: Messages::new(),
                window_size: None,
                min_window_size: None,
                blocking_threads: None,
                theme_at: Location::caller(),
                gpu_preference: GpuPreference::default(),
                handle: AppHandle::new(),
            },
        }
    }
    /// Theme of the whole App, evaluated reactively under its root owner.
    /// The fonts the theme carries ([`Theme::with_fonts`](kivact_style::Theme::with_fonts))
    /// are registered before the first screen, so text can name their
    /// families with `style().font_family(..)` from the start (like CSS
    /// `@font-face`). A file that is not a usable font (WOFF, unreadable)
    /// is reported as `ErrorKind::Asset`, located at this call; the App
    /// starts anyway.
    /// Embedded startup accepts only a fixed theme; create reactive themes with
    /// [`style::provide_theme`](crate::style::provide_theme) inside the embedded root factory.
    #[track_caller]
    pub fn theme(mut self, theme: impl kivact_runtime::IntoValue<Theme>) -> Self {
        self.theme = theme.into_prop();
        self.settings.theme_at = Location::caller();
        self
    }
    /// The App's locale: the language and region components format for.
    /// Without it the App speaks the system's locale (`en-US` where the
    /// system names none) and follows it when the user changes it while
    /// the App runs; with it the App keeps this locale whatever the system
    /// says. To switch it at runtime, wrap the root in
    /// [`i18n::provide_locale`](crate::i18n::provide_locale) with a `Signal<Locale>`. Read it with
    /// [`i18n::use_locale`](crate::i18n::use_locale).
    pub fn locale(mut self, locale: Locale) -> Self {
        self.settings.locale = Some(locale);
        self
    }
    /// The App's texts: the replacements for the default (English) texts
    /// of components, by key (`"dialog.close"`). Each component lists its
    /// keys. To switch them at runtime, wrap the root in
    /// [`i18n::provide_messages`](crate::i18n::provide_messages) with a `Signal<Messages>`.
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
    /// [`App::min_window_size`]: otherwise [`App::run`] returns an error
    /// of kind `ErrorKind::Message` without opening the window.
    pub fn window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.settings.window_size = Some(Size::new(width.into(), height.into()));
        self
    }
    /// The smallest content size in logical pixels the user can resize the
    /// window to, e.g. `.min_window_size(480, 320)` (default: no minimum).
    /// Both values must be finite, at least one logical pixel, fit the native
    /// window extent, and not be larger than
    /// [`App::window_size`] when that is set: otherwise [`App::run`]
    /// returns an error of kind `ErrorKind::Message` without opening the
    /// window.
    pub fn min_window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.settings.min_window_size = Some(Size::new(width.into(), height.into()));
        self
    }
    /// Size of the blocking pool (default `min(4, CPUs)`).
    pub fn blocking_threads(mut self, n: usize) -> Self {
        self.settings.blocking_threads = Some(n.max(1));
        self
    }
    /// What the App wishes of the window's GPU device: when it starts
    /// ([`GpuStart`](crate::platform::GpuStart), default on demand), the kind of
    /// adapter, the graphics API and the adapter itself (default: Arvane's own
    /// choice). A wish: the device is made on another adapter or API when the
    /// one asked for is not there, and the environment variables
    /// `ARVANE_RENDERER`, `WGPU_BACKEND` and `WGPU_POWER_PREF` override it. A
    /// build without the feature `gpu` draws on the CPU and ignores it. Read the
    /// adapter the device is on with `kivact::gpu_interop::adapter` (feature
    /// `gpu-interop`).
    pub fn gpu_preference(mut self, preference: GpuPreference) -> Self {
        self.settings.gpu_preference = preference;
        self
    }
    /// The handle that ends this App's run from any thread
    /// ([`platform::AppHandle`](crate::platform::AppHandle)). It belongs to the
    /// one run [`App::run`] starts.
    /// Inside the App, [`platform::app_handle`](crate::platform::app_handle)
    /// returns the same handle.
    pub fn handle(&self) -> AppHandle {
        self.settings.handle.clone()
    }
    /// Open the window and run until it closes. `root` is the body of the
    /// root component (usually a plain function such as `app` or
    /// `|| counter(0)`); it runs once, in the setup of the App's root owner.
    ///
    /// Errors when the window settings are invalid ([`App::window_size`],
    /// [`App::min_window_size`]; `ErrorKind::Message`, located here,
    /// before anything starts) or when the host fails to start (no window
    /// can be opened; `ErrorKind::Platform`).
    ///
    /// If the handle from [`App::handle`] was closed before this call, the
    /// run returns `Ok(())` without opening the window. A close while the App
    /// runs ends it normally (see
    /// [`platform::AppHandle::close`](crate::platform::AppHandle::close)).
    ///
    /// ```no_run
    /// # use kivact_core as kivact;
    /// use kivact::{App, Result};
    /// use kivact::view::{IntoView, Element, text};
    ///
    /// fn app() -> impl IntoView {
    ///     text("Hello")
    /// }
    ///
    /// fn main() -> kivact::Result<()> {
    ///     App::new("Hello").window_size(800, 600).run(app)
    /// }
    /// ```
    #[track_caller]
    pub fn run<V: IntoView>(self, root: impl FnOnce() -> V + 'static) -> Result<()> {
        self.check_window_sizes()?;
        if self.handle().begin() == Begin::Skip {
            return Ok(());
        }
        let options = self.host_options();
        let (client, _) =
            crate::launch::Launch::new(self.into_launch(), LaunchPolicy::window(), move || {
                root().into_view()
            })
            .into_engine_client();
        crate::launch::run_native(client, options)
    }

    #[track_caller]
    pub(crate) fn check_window_sizes(&self) -> Result<()> {
        crate::geometry::check_window_sizes(
            self.settings.window_size,
            self.settings.min_window_size,
            ("App::window_size", "App::min_window_size"),
            Location::caller(),
        )
    }
    /// Initial engine settings, derived from the application options.
    pub(crate) fn host_options(&self) -> kivact_engine::api::HostOptions {
        kivact_engine::api::HostOptions {
            window_size: self.settings.window_size,
            gpu_preference: self.settings.gpu_preference,
        }
    }
    /// The single title used for both engine startup and later window effects.
    pub(crate) fn window_title(&self) -> &str {
        &self.settings.title
    }
    /// Replace the window's title (kivact-test names its Apps).
    pub(crate) fn retitle(mut self, title: impl Into<String>) -> Self {
        self.settings.title = title.into();
        self
    }
    /// Extract only a fixed theme; reactive inputs stay on their owning thread.
    pub(crate) fn into_embed_settings(
        self,
        at: &'static Location<'static>,
    ) -> crate::Result<(AppSettings, Theme)> {
        let theme = self.theme.fixed().ok_or_else(|| {
            UiError::new(
                ErrorKind::Unsupported,
                "App::embed requires a fixed App theme; use style::provide_theme inside the Send root factory for an embedded reactive theme",
                Some(at),
            )
        })?;
        Ok((self.settings, theme))
    }

    /// The theme and settings a launch takes from this App.
    pub(crate) fn into_launch(self) -> LaunchApp {
        LaunchApp {
            theme: self.theme,
            settings: self.settings,
        }
    }

    /// An App with a fixed `theme` and `settings` (the embedded startup's pair).
    pub(crate) fn with_settings(settings: AppSettings, theme: Theme) -> Self {
        Self {
            theme: kivact_runtime::Prop::constant(theme),
            settings,
        }
    }
}

#[cfg(test)]
mod window_size_tests {
    //! Role: Tests for window sizes.
    //!
    //! The window settings are checked before anything starts, so these
    //! need no window.

    use super::App;
    use crate::Result;
    use crate::geometry::check_window_sizes;
    use kivact_runtime::ErrorKind;
    use kivact_types::Size;
    use std::panic::Location;

    fn check(window: Option<(f64, f64)>, min: Option<(f64, f64)>) -> Result<()> {
        let size = |(w, h): (f64, f64)| Size::new(w, h);
        check_window_sizes(
            window.map(size),
            min.map(size),
            ("App::window_size", "App::min_window_size"),
            Location::caller(),
        )
    }

    #[test]
    fn valid_window_sizes_pass() {
        assert!(check(None, None).is_ok());
        assert!(check(Some((1280.0, 800.0)), None).is_ok());
        assert!(check(None, Some((480.0, 320.0))).is_ok());
        assert!(check(Some((800.0, 600.0)), Some((800.0, 600.0))).is_ok());
        let app = App::new("t")
            .window_size(1280, 800)
            .min_window_size(640u16, 480.5);
        let ok = app.check_window_sizes();
        assert!(ok.is_ok(), "integer and float literals are taken");
        assert_eq!(
            app.host_options().window_size,
            Some(Size {
                width: 1280.0,
                height: 800.0
            })
        );
    }

    #[test]
    fn unusable_window_sizes_are_errors() {
        for bad in [
            (0.0, 600.0),
            (800.0, -1.0),
            (f64::NAN, 600.0),
            (800.0, f64::INFINITY),
            (1e300, 600.0),
        ] {
            let error = check(Some(bad), None).expect_err("window_size refused");
            assert_eq!(*error.kind(), ErrorKind::Message);
            assert!(error.to_string().contains("App::window_size"), "{error}");
            assert!(error.location().is_some(), "located at App::run");
            let error = check(None, Some(bad)).expect_err("min_window_size refused");
            assert!(
                error.to_string().contains("App::min_window_size"),
                "{error}"
            );
        }
    }

    #[test]
    fn a_minimum_larger_than_the_size_is_an_error() {
        for min in [(801.0, 600.0), (800.0, 601.0)] {
            let error = check(Some((800.0, 600.0)), Some(min)).expect_err("refused");
            assert_eq!(*error.kind(), ErrorKind::Message);
            assert!(error.to_string().contains("larger than"), "{error}");
        }
    }
}

#[cfg(test)]
mod handle_tests {
    //! Role: Tests for AppHandle at the run boundaries.

    use super::App;
    use crate::view::text;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn closing_before_the_run_returns_without_opening_a_window() {
        let app = App::new("Closed early");
        let handle = app.handle();
        handle.close();
        let ran = Rc::new(Cell::new(false));
        let started = ran.clone();
        let result = app.run(move || {
            started.set(true);
            text("never")
        });
        assert!(result.is_ok(), "a closed run returns Ok");
        assert!(!ran.get(), "the root never runs");
    }
}
