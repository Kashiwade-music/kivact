//! Role: App setup: title, theme, locale, fonts, window sizes and the run entry.

use crate::i18n::{Locale, Messages};
use crate::view::IntoView;
use crate::{Asset, Result, UiError};
use kivact_runtime::ErrorKind;
use kivact_style::Theme;
use kivact_types::Size;
use std::panic::Location;

/// An application: title, window size, theme, fonts and the root
/// component.
#[derive(Clone)]
pub struct App {
    title: String,
    theme: Theme,
    /// `None`: the system's locale (`App::locale`).
    locale: Option<Locale>,
    messages: Messages,
    /// The window's first size (`window_size`); the engine's default when
    /// `None`.
    pub(crate) window_size: Option<Size>,
    /// The window's minimum size (`min_window_size`).
    pub(crate) min_window_size: Option<Size>,
    threads: Option<usize>,
    fonts: Vec<(Asset, &'static Location<'static>)>,
    /// How the window's GPU device is made (`gpu_preference`).
    #[cfg(feature = "gpu-interop")]
    pub(crate) gpu_preference: crate::gpu_interop::GpuPreference,
}

impl App {
    /// An app whose window is titled `title`, with the default theme.
    pub fn new(title: impl Into<String>) -> App {
        App {
            title: title.into(),
            theme: Theme::default(),
            locale: None,
            messages: Messages::new(),
            window_size: None,
            min_window_size: None,
            threads: None,
            fonts: Vec::new(),
            #[cfg(feature = "gpu-interop")]
            gpu_preference: crate::gpu_interop::GpuPreference::default(),
        }
    }

    /// Theme of the whole App (to switch it at runtime, wrap the root in
    /// `view::provide_theme` with a `Signal<Theme>`).
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
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
        self.locale = Some(locale);
        self
    }

    /// The App's texts: the replacements for the default (English) texts
    /// of components, by key (`"dialog.close"`). Each component lists its
    /// keys. To switch them at runtime, wrap the root in
    /// [`view::provide_messages`](crate::view::provide_messages) with a `Signal<Messages>`.
    pub fn messages(mut self, messages: Messages) -> Self {
        self.messages = messages;
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
        self.window_size = Some(Size::new(width.into(), height.into()));
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
        self.min_window_size = Some(Size::new(width.into(), height.into()));
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
        self.fonts.push((font.into(), Location::caller()));
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
        self.gpu_preference = preference;
        self
    }

    /// Size of the blocking pool (default `min(4, CPUs)`).
    pub fn blocking_threads(mut self, n: usize) -> Self {
        self.threads = Some(n.max(1));
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
    /// use kivact::prelude::*;
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
        let at = Location::caller();
        check_window_sizes(self.window_size, self.min_window_size, at)?;
        let options = kivact_engine::api::HostOptions {
            window_size: self.window_size,
            #[cfg(feature = "gpu-interop")]
            gpu_preference: self.gpu_preference,
            #[cfg(not(feature = "gpu-interop"))]
            gpu_preference: Default::default(),
        };
        crate::platform::run_native(self.client(root, at), options)
    }

    fn client<V: IntoView>(
        self,
        root: impl FnOnce() -> V + 'static,
        at: &'static Location<'static>,
    ) -> kivact_engine::api::Client {
        self.app_client(root, at).into_client(None)
    }

    pub(crate) fn app_client<V: IntoView>(
        self,
        root: impl FnOnce() -> V + 'static,
        at: &'static Location<'static>,
    ) -> crate::client::AppClient {
        let mut config = crate::client::AppConfig::new(
            self.title,
            self.theme,
            Box::new(move || root().into_view()),
        );
        config.settings.locale = self.locale;
        config.settings.messages = self.messages;
        config.fonts = self.fonts;
        config.min_window_size = self.min_window_size;
        crate::client::AppClient::new(config, self.threads, at)
    }
}

/// Refuse a window size or minimum size that cannot form a drawable extent,
/// and a minimum larger than the size, as an error located at `at`
/// (`App::run`): the platform would otherwise clamp or ignore it silently.
pub(crate) fn check_window_sizes(
    window: Option<Size>,
    min: Option<Size>,
    at: &'static Location<'static>,
) -> Result<()> {
    let fail = |message: String| Err(UiError::new(ErrorKind::Message, message, Some(at)));
    for (name, size) in [("window_size", window), ("min_window_size", min)] {
        if let Some(s) = size
            && !kivact_types::env::is_valid_window_size(s)
        {
            return fail(format!(
                "App::{name}: {} by {} must be finite, at least one logical pixel and fit the native window extent",
                s.width, s.height
            ));
        }
    }
    if let (Some(window), Some(min)) = (window, min)
        && (min.width > window.width || min.height > window.height)
    {
        return fail(format!(
            "App::min_window_size: {} by {} is larger than App::window_size {} by {}",
            min.width, min.height, window.width, window.height
        ));
    }
    Ok(())
}

#[cfg(test)]
mod window_size_tests {
    //! Role: Tests for window sizes.
    //!
    //! The window settings are checked before anything starts, so these
    //! need no window.

    use super::*;

    fn check(window: Option<(f64, f64)>, min: Option<(f64, f64)>) -> Result<()> {
        let size = |(w, h): (f64, f64)| Size::new(w, h);
        check_window_sizes(window.map(size), min.map(size), Location::caller())
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
        let ok = check_window_sizes(app.window_size, app.min_window_size, Location::caller());
        assert!(ok.is_ok(), "integer and float literals are taken");
        assert_eq!(
            app.window_size,
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
