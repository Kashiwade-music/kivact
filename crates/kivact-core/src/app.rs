//! Role: Public application builder and native window entry.
use crate::Result;
use crate::i18n::{Locale, Messages};
use crate::launch::AppOptions;
use crate::view::Asset;
use crate::view::IntoView;

use kivact_style::Theme;

#[derive(Clone)]
pub struct App {
    pub(crate) options: AppOptions,
}
impl App {
    /// An app whose window is titled `title`, with the default theme.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            options: AppOptions::new(title),
        }
    }
    /// Theme of the whole App, evaluated reactively under its root owner.
    /// Embedded startup accepts only a fixed theme; create reactive themes with
    /// [`style::provide_theme`](crate::style::provide_theme) inside the embedded root factory.
    pub fn theme(mut self, theme: impl kivact_runtime::IntoValue<Theme>) -> Self {
        self.options = self.options.theme(theme);
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
        self.options = self.options.locale(locale);
        self
    }
    /// The App's texts: the replacements for the default (English) texts
    /// of components, by key (`"dialog.close"`). Each component lists its
    /// keys. To switch them at runtime, wrap the root in
    /// [`i18n::provide_messages`](crate::i18n::provide_messages) with a `Signal<Messages>`.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.options = self.options.messages(messages);
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
        self.options = self.options.window_size(width, height);
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
        self.options = self.options.min_window_size(width, height);
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
        self.options = self.options.font(font);
        self
    }
    /// Register an ordered collection of font files with the same loading and
    /// validation behavior as [`Self::font`].
    #[track_caller]
    pub fn fonts(mut self, fonts: impl IntoIterator<Item = impl Into<Asset>>) -> Self {
        for font in fonts {
            self = self.font(font);
        }
        self
    }
    /// Size of the blocking pool (default `min(4, CPUs)`).
    pub fn blocking_threads(mut self, n: usize) -> Self {
        self.options = self.options.blocking_threads(n);
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
        self.options = self.options.gpu_preference(preference);
        self
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
        self.options.check_window_sizes()?;
        let options = self.options.host_options();
        let (client, _) = crate::launch::Launch::new(
            self.options,
            crate::launch::ClientMode::Window,
            move || root().into_view(),
        )
        .into_engine_client();
        crate::launch::run_native(client, options)
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
        let ok = app.options.check_window_sizes();
        assert!(ok.is_ok(), "integer and float literals are taken");
        assert_eq!(
            app.options.host_options().window_size,
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
