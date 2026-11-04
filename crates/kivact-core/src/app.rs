//! Role: Public application builder and native window entry.
use crate::i18n::{Locale, Messages};
use crate::launch::AppOptions;
use crate::view::IntoView;
use crate::{Asset, Result, UiError};
use kivact_runtime::ErrorKind;
use kivact_style::Theme;
use kivact_types::Size;
use std::panic::Location;
#[derive(Clone)]
pub struct App {
    pub(crate) options: AppOptions,
}
impl App {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            options: AppOptions::new(title),
        }
    }
    pub fn theme(mut self, theme: Theme) -> Self {
        self.options = self.options.theme(theme);
        self
    }
    pub fn locale(mut self, locale: Locale) -> Self {
        self.options = self.options.locale(locale);
        self
    }
    pub fn messages(mut self, messages: Messages) -> Self {
        self.options = self.options.messages(messages);
        self
    }
    pub fn window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.options = self.options.window_size(width, height);
        self
    }
    pub fn min_window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.options = self.options.min_window_size(width, height);
        self
    }
    #[track_caller]
    pub fn font(mut self, font: impl Into<Asset>) -> Self {
        self.options = self.options.font(font);
        self
    }
    pub fn blocking_threads(mut self, n: usize) -> Self {
        self.options = self.options.blocking_threads(n);
        self
    }
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
        self.options.check_window_sizes()?;
        let options = kivact_engine::api::HostOptions {
            window_size: self.options.host_options().window_size,
            gpu_preference: self.options.host_options().gpu_preference,
        };
        let (client, _) = crate::launch::Launch::new(
            self.options,
            crate::launch::ClientMode::Window,
            move || root().into_view(),
        )
        .into_engine_client();
        crate::platform::run_native(client, options)
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
        let ok = check_window_sizes(
            app.options.host_options().window_size,
            Some(Size::new(640.0, 480.5)),
            Location::caller(),
        );
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
