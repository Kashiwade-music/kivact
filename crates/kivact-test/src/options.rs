//! Role: Configure test fonts, windows and rendering.

use kivact_core::i18n::{Locale, Messages};
use kivact_style::Theme;

/// The fonts text is shaped with ([`RenderOptions::fonts`]). The fonts a
/// theme carries ([`Theme::with_fonts`]) are added to them.
#[derive(Clone, Debug, Default)]
pub enum TestFonts {
    /// Only the bundled test font (Tuffy, no CJK glyphs): the same layout
    /// and pixels on every machine. A theme with fonts replaces it with
    /// those fonts alone.
    #[default]
    Bundled,
    /// The system's fonts, as a running App sees them.
    System,
}

pub use kivact_engine::testing::{MirrorMode, Renderer};

/// How a test window uses the GPU (`RenderOptions::gpu`, `gpu_on_demand`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TestGpu {
    /// The CPU only (`ARVANE_RENDERER=cpu`).
    #[default]
    Off,
    /// The device starts with the window (`ARVANE_RENDERER=gpu`).
    AtLaunch,
    /// The device starts when the window needs it (`ARVANE_RENDERER=auto`).
    OnDemand,
}

fn mirror_from_env() -> MirrorMode {
    match std::env::var("KIVACT_TEST_MIRROR").as_deref() {
        Ok("off") => MirrorMode::Off,
        Ok("required") => MirrorMode::Required,
        _ => MirrorMode::Auto,
    }
}

/// Options shared with a running App and additional test-host settings.
pub struct RenderOptions {
    app: kivact_core::App,
    /// Whether the DevTools agent starts for this test App.
    devtools: bool,
    host_locale: Option<String>,
    reduced_motion: bool,
    dark: bool,
    scale: f64,
    fonts: TestFonts,
    mirror: MirrorMode,
    gpu: TestGpu,
    frame_period_ns: u64,
    single_window: bool,
    foreign_windows: bool,
    restored_state: std::collections::BTreeMap<String, Vec<u8>>,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            app: kivact_core::App::new(std::thread::current().name().unwrap_or("test"))
                .window_size(800, 600),
            devtools: false,
            host_locale: None,
            reduced_motion: false,
            dark: false,
            scale: 1.,
            fonts: TestFonts::Bundled,
            mirror: mirror_from_env(),
            gpu: TestGpu::Off,
            frame_period_ns: kivact_types::defaults::FRAME_PERIOD_NS,
            single_window: false,
            foreign_windows: false,
            restored_state: Default::default(),
        }
    }
}
impl RenderOptions {
    /// Supply the state saved by an earlier virtual launch.
    pub fn restored_state(mut self, state: std::collections::BTreeMap<String, Vec<u8>>) -> Self {
        self.restored_state = state;
        self
    }
    pub fn from_env() -> Self {
        Self {
            gpu: if std::env::var("KIVACT_TEST_GPU").as_deref() == Ok("1") {
                TestGpu::AtLaunch
            } else {
                TestGpu::Off
            },
            ..Self::default()
        }
    }
    #[track_caller]
    pub fn with_theme(theme: Theme) -> Self {
        Self::default().theme(theme)
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.app = kivact_core::__test::retitle(self.app, title);
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
    /// Draw on the GPU from the first frame: the window's device starts with
    /// the window and the test waits until it is ready, as
    /// `ARVANE_RENDERER=gpu` does for a running App (`GpuStart::AtLaunch`).
    /// Needs a real window (`MirrorMode::Required`) and a GPU
    /// ([`gpu_unavailable`](crate::gpu_unavailable)).
    pub fn gpu(mut self) -> Self {
        self.gpu = TestGpu::AtLaunch;
        self
    }
    /// Draw on the CPU until the window needs the GPU, then start its device,
    /// as a running App does by default (`GpuStart::OnDemand`,
    /// `ARVANE_RENDERER=auto`): a user shader drawn, or the App reading
    /// `gpu_interop::adapter` (a `shared_image` does), starts it.
    /// [`Screen::renderer`](crate::Screen::renderer) says what draws, and
    /// [`Screen::wait_for_gpu`](crate::Screen::wait_for_gpu) waits for the
    /// device. Needs a real window and a GPU, as [`Self::gpu`] does.
    pub fn gpu_on_demand(mut self) -> Self {
        self.gpu = TestGpu::OnDemand;
        self
    }
    #[track_caller]
    pub fn theme(mut self, theme: impl kivact_runtime::IntoValue<Theme>) -> Self {
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
    pub fn blocking_threads(mut self, n: usize) -> Self {
        self.app = self.app.blocking_threads(n);
        self
    }
    pub fn window_size(mut self, width: impl Into<f64>, height: impl Into<f64>) -> Self {
        self.app = self.app.window_size(width, height);
        self
    }
    /// Its `start` is the test's: [`Self::gpu`] or [`Self::gpu_on_demand`]
    /// decide it, as `ARVANE_RENDERER` does for a running App.
    pub fn gpu_preference(mut self, preference: kivact_core::platform::GpuPreference) -> Self {
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
    /// The test platform opens no window beyond the first (`Capabilities.multi_window` and `window_placement` false), like an embedded editor.
    pub fn single_window(mut self) -> Self {
        self.single_window = true;
        self
    }
    /// The test platform opens windows whose content is one native child
    /// window (`Capabilities.foreign_window`), as `foreign_window` needs.
    pub fn foreign_windows(mut self) -> Self {
        self.foreign_windows = true;
        self
    }
    /// Start the DevTools agent for this test App.
    #[cfg(feature = "devtools")]
    pub fn devtools(mut self) -> Self {
        self.devtools = true;
        self
    }
}

impl RenderOptions {
    pub(crate) fn is_headless(&self) -> bool {
        self.mirror == MirrorMode::Off
    }
    pub(crate) fn into_host(
        self,
    ) -> (
        kivact_core::App,
        bool,
        kivact_engine::testing::TestHostOptions,
    ) {
        let app = self.app;
        kivact_core::__test::check_window_sizes(&app).expect("valid test application window sizes");
        let title = kivact_core::__test::window_title(&app).to_owned();
        let initial = kivact_core::__test::host_options(&app);
        let size = initial
            .window_size
            .unwrap_or(kivact_types::defaults::DEFAULT_WINDOW_SIZE);
        let host = kivact_engine::testing::TestHostOptions {
            restored_state: self.restored_state,
            width: size.width,
            height: size.height,
            scale: self.scale,
            // The theme's fonts are the App's to register: a theme with fonts
            // starts the host without the bundled font.
            fonts: match self.fonts {
                TestFonts::Bundled if kivact_core::__test::app_theme_has_fonts(&app) => {
                    kivact_engine::testing::TestFonts::Only(Vec::new())
                }
                TestFonts::Bundled => kivact_engine::testing::TestFonts::Bundled,
                TestFonts::System => kivact_engine::testing::TestFonts::System,
            },
            mirror: self.mirror,
            title,
            frame_period_ns: self.frame_period_ns,
            single_window: self.single_window,
            foreign_windows: self.foreign_windows,
            theme: if self.dark {
                kivact_types::ThemeMode::Dark
            } else {
                kivact_types::ThemeMode::Light
            },
            reduced_motion: self.reduced_motion,
            locale: self.host_locale,
            inline_assets: true,
            gpu: match self.gpu {
                TestGpu::Off => kivact_engine::testing::GpuMode::Off,
                TestGpu::AtLaunch | TestGpu::OnDemand => kivact_engine::testing::GpuMode::Required,
            },
            gpu_preference: kivact_types::GpuPreference {
                start: match self.gpu {
                    TestGpu::OnDemand => kivact_types::gpu::GpuStart::OnDemand,
                    TestGpu::Off => kivact_types::gpu::GpuStart::Never,
                    TestGpu::AtLaunch => kivact_types::gpu::GpuStart::AtLaunch,
                },
                ..initial.gpu_preference
            },
        };
        (app, self.devtools, host)
    }
}

impl std::fmt::Debug for RenderOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderOptions")
            .field("title", &kivact_core::__test::window_title(&self.app))
            .field(
                "window_size",
                &kivact_core::__test::host_options(&self.app).window_size,
            )
            .field("host_locale", &self.host_locale)
            .field("reduced_motion", &self.reduced_motion)
            .field("dark", &self.dark)
            .field("scale", &self.scale)
            .field("fonts", &self.fonts)
            .field("mirror", &self.mirror)
            .field("gpu", &self.gpu)
            .field("frame_period_ns", &self.frame_period_ns)
            .field("foreign_windows", &self.foreign_windows)
            .finish_non_exhaustive()
    }
}
