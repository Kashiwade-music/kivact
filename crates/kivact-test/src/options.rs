//! Role: Configure test fonts, windows and rendering.

use kivact_core::i18n::{Locale, Messages};
use kivact_style::Theme;

/// The fonts text is shaped with ([`RenderOptions::fonts`]).
#[derive(Clone, Debug, Default)]
pub enum TestFonts {
    /// Only the bundled test font (Tuffy, no CJK glyphs): the same layout
    /// and pixels on every machine.
    #[default]
    Bundled,
    /// Only these font files (TTF/OTF bytes): deterministic too.
    Only(Vec<kivact_core::Asset>),
    /// The system's fonts, as a running App sees them.
    System,
}

pub use kivact_engine::testing::MirrorMode;

fn mirror_from_env() -> MirrorMode {
    match std::env::var("KIVACT_TEST_MIRROR").as_deref() {
        Ok("off") => MirrorMode::Off,
        Ok("required") => MirrorMode::Required,
        _ => MirrorMode::Auto,
    }
}

use kivact_core::launch::AppOptions;

/// Options shared with a running App and additional test-host settings.
#[derive(Clone)]
pub struct RenderOptions {
    app: kivact_core::launch::AppOptions,
    host_locale: Option<String>,
    reduced_motion: bool,
    dark: bool,
    scale: f64,
    fonts: TestFonts,
    mirror: MirrorMode,
    gpu: bool,
    frame_period_ns: u64,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            app: AppOptions::new(std::thread::current().name().unwrap_or("test"))
                .window_size(800, 600),
            host_locale: None,
            reduced_motion: false,
            dark: false,
            scale: 1.,
            fonts: TestFonts::Bundled,
            mirror: mirror_from_env(),
            gpu: false,
            frame_period_ns: kivact_types::defaults::FRAME_PERIOD_NS,
        }
    }
}
impl RenderOptions {
    pub fn from_env() -> Self {
        let mut options = Self::default();
        options.gpu = std::env::var("KIVACT_TEST_GPU").as_deref() == Ok("1");
        options
    }
    pub fn with_theme(theme: Theme) -> Self {
        Self::default().theme(theme)
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.app = self.app.title(title);
        self
    }
    pub fn host_locale(mut self, tag: impl Into<String>) -> Self {
        self.host_locale = Some(tag.into());
        self
    }
    pub fn reduced_motion(mut self) -> Self {
        self.reduced_motion = true;
        self
    }
    pub fn dark(mut self) -> Self {
        self.dark = true;
        self
    }
    pub fn gpu(mut self) -> Self {
        self.gpu = true;
        self
    }
    pub fn theme(mut self, theme: Theme) -> Self {
        self.app = self.app.theme(theme);
        self
    }
    pub fn locale(mut self, locale: Locale) -> Self {
        self.app = self.app.locale(locale);
        self
    }
    pub fn messages(mut self, messages: Messages) -> Self {
        self.app = self.app.messages(messages);
        self
    }
    pub fn min_window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.app = self.app.min_window_size(width, height);
        self
    }
    #[track_caller]
    pub fn font(mut self, font: impl Into<kivact_core::Asset>) -> Self {
        self.app = self.app.font(font);
        self
    }
    pub fn blocking_threads(mut self, n: usize) -> Self {
        self.app = self.app.blocking_threads(n);
        self
    }
    pub fn window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.app = self.app.window_size(width, height);
        self
    }
    #[cfg(feature = "gpu-interop")]
    pub fn gpu_preference(mut self, preference: kivact_types::GpuPreference) -> Self {
        self.app = self.app.gpu_preference(preference);
        self
    }
    pub fn scale(mut self, value: f64) -> Self {
        self.scale = value;
        self
    }
    pub fn fonts(mut self, value: TestFonts) -> Self {
        self.fonts = value;
        self
    }
    pub fn mirror(mut self, value: MirrorMode) -> Self {
        self.mirror = value;
        self
    }
    pub fn frame_period_ns(mut self, value: u64) -> Self {
        self.frame_period_ns = value;
        self
    }
}

impl RenderOptions {
    pub(crate) fn uses_gpu(&self) -> bool {
        self.gpu
    }
    pub(crate) fn is_headless(&self) -> bool {
        self.mirror == MirrorMode::Off
    }
    pub(crate) fn into_host(
        self,
        gpu: bool,
        fonts: Vec<(kivact_core::Asset, &'static std::panic::Location<'static>)>,
    ) -> (AppOptions, kivact_engine::testing::TestHostOptions) {
        let mut app = self.app;
        for (font, _) in fonts {
            app = app.font(font);
        }
        app.check_window_sizes()
            .expect("valid test application window sizes");
        let title = app.window_title().to_owned();
        let initial = app.host_options();
        let size = initial
            .window_size
            .unwrap_or(kivact_types::defaults::DEFAULT_WINDOW_SIZE);
        let host = kivact_engine::testing::TestHostOptions {
            width: size.width,
            height: size.height,
            scale: self.scale,
            fonts: match self.fonts {
                TestFonts::Bundled => kivact_engine::testing::TestFonts::Bundled,
                TestFonts::System => kivact_engine::testing::TestFonts::System,
                TestFonts::Only(fonts) => kivact_engine::testing::TestFonts::Only(
                    fonts
                        .into_iter()
                        .map(|font| match font {
                            kivact_core::Asset::Shared(bytes) => bytes,
                            kivact_core::Asset::Static(bytes) => std::sync::Arc::from(bytes),
                            kivact_core::Asset::Path(path) => std::sync::Arc::from(
                                std::fs::read(path).expect("test font file is readable"),
                            ),
                        })
                        .collect(),
                ),
            },
            mirror: self.mirror,
            title,
            frame_period_ns: self.frame_period_ns,
            theme: if self.dark {
                kivact_types::ThemeHint::Dark
            } else {
                kivact_types::ThemeHint::Light
            },
            reduced_motion: self.reduced_motion,
            locale: self.host_locale,
            inline_assets: true,
            gpu: if gpu {
                kivact_engine::testing::GpuMode::Required
            } else {
                kivact_engine::testing::GpuMode::Off
            },
            gpu_preference: initial.gpu_preference,
        };
        (app, host)
    }
}

impl std::fmt::Debug for RenderOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderOptions")
            .field("title", &self.app.window_title())
            .field("window_size", &self.app.host_options().window_size)
            .field("host_locale", &self.host_locale)
            .field("reduced_motion", &self.reduced_motion)
            .field("dark", &self.dark)
            .field("scale", &self.scale)
            .field("fonts", &self.fonts)
            .field("mirror", &self.mirror)
            .field("gpu", &self.gpu)
            .field("frame_period_ns", &self.frame_period_ns)
            .finish_non_exhaustive()
    }
}
