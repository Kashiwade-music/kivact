//! Role: App::embed: validate, open and report startup failure.

use super::Embedded;
use super::Parent;
use crate::App;
use crate::Result;
use crate::UiError;
use kivact_runtime::ErrorKind;
use std::panic::Location;

impl App {
    /// Run `root` inside `parent` and return once it is built (feature `embed`).
    /// Call on the host GUI thread. `root` and every App callback run on the App
    /// UI thread, which may be another thread; they must not wait for the host GUI
    /// thread. The initial physical size is `window_size` times the scale (default
    /// 960 by 720 logical pixels). Invalid sizes or scale return
    /// `ErrorKind::Message`; an invalid parent, unavailable OS resources or an
    /// unsupported OS return `ErrorKind::Platform`. A failed open either creates
    /// no window or cleans up a child already created. A panic in `root` or
    /// while opening returns `ErrorKind::Panicked` and leaves no window; a
    /// later panic stops the App and is reported by [`Embedded::status`].
    /// Kivact never unwinds into the caller. Startup failure normally waits for
    /// cleanup, with the same two-second limit per owned thread as drop;
    /// unfinished cleanup may continue under module retention after `Err` is
    /// returned. Title and fullscreen have no effect.
    /// The App theme must be fixed. A reactive theme returns `ErrorKind::Unsupported`;
    /// create it with [`style::provide_theme`](crate::style::provide_theme) inside
    /// the `Send` root factory so it belongs to the App UI thread.
    /// `min_window_size` clamps `platform::request_window_size`, not the host.
    #[track_caller]
    #[cfg(feature = "embed")]
    pub fn embed<V: crate::view::IntoView>(
        self,
        parent: Parent,
        root: impl FnOnce() -> V + Send + 'static,
    ) -> Result<Embedded> {
        self.embed_impl(parent, root, Location::caller())
    }

    #[track_caller]
    pub(crate) fn embed_impl<V: crate::view::IntoView>(
        self,
        parent: Parent,
        root: impl FnOnce() -> V + Send + 'static,
        at: &'static Location<'static>,
    ) -> Result<Embedded> {
        self.options.check_window_sizes()?;
        if parent
            .scale
            .is_some_and(|scale| !scale.is_finite() || scale <= 0.0)
        {
            return Err(UiError::new(
                ErrorKind::Message,
                "Parent::scale must be finite and positive in f64",
                Some(at),
            ));
        }
        let window = parent.window.ok_or_else(|| {
            UiError::new(
                ErrorKind::Platform,
                "the parent window handle is null",
                Some(at),
            )
        })?;
        let panic = kivact_engine::embed::PanicSlot::default();
        let options = kivact_types::EmbedOptions {
            size: self
                .options
                .host_options()
                .window_size
                .unwrap_or(kivact_types::defaults::DEFAULT_WINDOW_SIZE),
            scale: parent.scale,
            visibility: parent.visibility,
            unhandled_keys: parent.unhandled_keys,
            prefer_gpu: true,
            gpu_preference: self.options.host_options().gpu_preference,
        };
        let hooks = kivact_engine::embed::ChildHooks {
            on_resize_request: parent.on_resize_request,
            on_notice: parent.on_notice,
            panic: panic.clone(),
        };
        let (settings, theme) = self.options.into_embed_settings(at)?;
        let opened = kivact_engine::embed::embed_native(window, options, hooks, move |closing| {
            crate::launch::Launch::new(
                settings.with_theme(theme),
                crate::launch::ClientMode::Embedded(closing),
                move || root().into_view(),
            )
            .into_engine_client()
            .0
        });
        match opened {
            // The App panicked while opening: close what it built and report
            // the panic as a value (the payload is already forgotten).
            Ok(host) if panic.is_set() => {
                drop(host);
                Err(panicked(&panic, at))
            }
            Ok(host) => Ok(Embedded::new(host, panic)),
            Err(_) if panic.is_set() => Err(panicked(&panic, at)),
            Err(error) => Err(UiError::new(ErrorKind::Platform, error.message, None)),
        }
    }
}

/// The startup error of a panic that stopped the App while opening.
fn panicked(panic: &kivact_engine::embed::PanicSlot, at: &'static Location<'static>) -> UiError {
    let message = panic
        .message()
        .unwrap_or_else(|| "the embedded App panicked while opening".to_owned());
    UiError::new(ErrorKind::Panicked, message, Some(at))
}
