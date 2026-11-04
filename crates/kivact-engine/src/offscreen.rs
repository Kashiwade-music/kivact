//! Role: Window-independent image output of a scene through the same painting.
//!
//! Window-independent image output: a scene lowered and rasterized by the
//! same painting a canvas in a window goes through, with no node tree.
//! Text is measured with no node either ([`TextMeasurer`]): a drawn run
//! without a family or a line height takes the ones the caller gives (the
//! theme's, as a window's root does), and it shares the output's text
//! engine, so a font registered later is seen. It is drawn by Arvane's
//! CPU rasterizer alone, at a raster scale of 1, from a transparent start;
//! the rasterizer's premultiplied pixels are turned back into straight
//! alpha. The size limit is the rasterizer's frame budget, the one windows
//! are held to ([`OffscreenRenderer::accepts_size`]); the caller checks it before it
//! draws anything. Encoding, the timing of frames and cancelling between
//! them are the caller's.
//!
//! Invariants:
//! - A scene is lowered once by [`Context`] (`paint/scene.rs`), the code a
//!   window's canvas paints with, so geometry is repaired and dropped the
//!   same way. What differs is the failure policy: a window leaves a
//!   resource it cannot use out of the frame, image output returns the
//!   first failure of a command that was actually drawn. Commands the
//!   painting dropped (values that cannot be drawn) come first, as
//!   [`OffscreenError::Dropped`] with their count. A font that cannot be registered gives its message
//!   ([`OffscreenRenderer::font`]).
//! - Resources complete inside the call (the asset store is inline), and an
//!   error leaves the output usable for the next frame.
//! - The output is straight-alpha RGBA8 in its own allocation; the output
//!   owns the assets and the text engine, which its thread keeps. An
//!   [`OffscreenRenderer`] is not `Send`; dropping it releases its caches.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::text_measure::TextMeasurer;
use kivact_lower::assets::ArvaneAssets;
use kivact_lower::context::Context;
use kivact_lower::effects::Tint;
use kivact_types::{Asset, ComputedTextStyle, DrawList, PlatformResult, Rect};

/// Resources and rasterizer of window-independent drawing, confined to the
/// thread that made them.
pub struct OffscreenRenderer {
    assets: RefCell<ArvaneAssets>,
    scene_cache: RefCell<kivact_lower::SceneCache>,
    engine: Rc<RefCell<arvane::Engine>>,
    rasterizer: arvane::Rasterizer,
}

/// Why an image was not rendered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OffscreenError {
    /// The scene holds commands whose values cannot be drawn (the number
    /// of commands the painting dropped).
    Dropped(u32),
    /// A resource, the shaping of a text or the rasterization failed.
    Failed(kivact_types::Failure),
}

/// A rendered image: tightly packed straight-alpha RGBA8, top row first, in
/// an allocation of its own.
pub struct OffscreenImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl Default for OffscreenRenderer {
    fn default() -> Self {
        Self {
            assets: RefCell::new(ArvaneAssets::new(arvane::asset::AssetStore::inline())),
            scene_cache: RefCell::default(),
            engine: Rc::new(RefCell::new(arvane::Engine::new())),
            rasterizer: arvane::Rasterizer::new(),
        }
    }
}

impl OffscreenRenderer {
    /// Release rebuildable scene, text, unused asset and validation caches.
    /// Embedders call this explicitly: independent outputs do not subscribe
    /// to window memory events. Registered fonts and returned images remain valid.
    pub fn release_caches(&mut self) {
        self.scene_cache.get_mut().clear();
        self.assets.get_mut().release_caches();
        self.engine.borrow_mut().release_caches();
        self.rasterizer.release_caches();
    }

    pub fn new() -> Self {
        Self::default()
    }

    /// Whether an image of this size is rendered: the rasterizer's frame
    /// budget, the same one windows are held to.
    pub fn accepts_size(width: u32, height: u32) -> bool {
        arvane::Rasterizer::check_size(width, height).is_ok()
    }

    /// Text measurement for a drawing callback whose runs inherit
    /// `inherited`'s family and line height; fonts registered later are
    /// seen through the shared text engine.
    pub fn text_measurer(&self, inherited: ComputedTextStyle) -> TextMeasurer {
        TextMeasurer::new(Rc::clone(&self.engine), inherited)
    }

    /// Register a font file now, returning its families, or why it failed.
    pub fn font(&mut self, asset: &Asset) -> Result<Vec<String>, String> {
        let mut engine = self.engine.borrow_mut();
        match self.assets.get_mut().register_font(&mut engine, asset) {
            PlatformResult::Fonts(families) => Ok(families),
            PlatformResult::Failed(message) => Err(message),
            _ => Err("the font was not registered".to_owned()),
        }
    }

    /// Lower `scene` through the painting context, then rasterize exactly
    /// `size` pixels. Text runs without a family or a line height take
    /// `inherited`'s. The first failure of a drawn command is the error.
    pub fn render(
        &mut self,
        scene: &std::sync::Arc<DrawList>,
        size: (u32, u32),
        inherited: &ComputedTextStyle,
    ) -> Result<OffscreenImage, OffscreenError> {
        self.assets.get_mut().begin_frame();
        self.assets.get_mut().discard_failures();
        let raster = Cell::new(1.0);
        let context = Context::new(&self.assets, &raster, &self.scene_cache);
        let mut lowered = kivact_lower::SceneOut::default();
        context.paint_scene(
            scene,
            Some(inherited),
            Rect {
                x: 0.0,
                y: 0.0,
                width: size.0 as f64,
                height: size.1 as f64,
            },
            &Tint {
                alpha: 1.0,
                matrix: None,
            },
            &mut self.engine.borrow_mut(),
            &mut lowered,
        );
        // Invalid values are the drawing's fault, so they come first.
        let dropped = context.dropped();
        let failure = context.finish();
        if dropped > 0 {
            return Err(OffscreenError::Dropped(dropped));
        }
        if let Some(failure) = failure.into_iter().next() {
            return Err(OffscreenError::Failed(failure));
        }
        // The failures the store kept are the ones the context reported.
        self.assets.get_mut().discard_failures();
        let lowered = lowered.into_scene();
        let mut frame = self
            .rasterizer
            .render(&lowered, size.0, size.1)
            .map_err(|e| OffscreenError::Failed(e.into()))?;
        // PNG and Draw::pixels are straight alpha; Arvane's frame is
        // premultiplied.
        kivact_types::color::unpremultiply(&mut frame.rgba);
        Ok(OffscreenImage {
            width: frame.width,
            height: frame.height,
            rgba: frame.rgba,
        })
    }
}
