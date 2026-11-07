//! Role: UI-local image and font resources with located failure histories.
//!
//! Images and fonts through Arvane's asset layer
//! (`arvane::asset`): the asset store and its worker thread, the pixels handed to Arvane, font
//! registration and family resolution, and the failure reports.
//!
//! Invariants:
//! - Arvane reads no encoded image: the store decodes and
//!   rasterizes, and the host hands Arvane the store's straight-alpha RGBA8
//!   as `ImageData` sharing the store's allocation (no copy; Arvane's
//!   `ImageData` is straight-alpha RGBA8 too).
//! - What is drawn for an image always comes from the store for the
//!   node's current asset ([`ArvaneAssets::pixels`]), so a new asset never
//!   shows the old asset's pixels.
//! - Arvane resolves every family ([`family`]) against its platform table
//!   and font database (`Engine::resolve_font_family`). This module only
//!   converts the choice to a text style and reports missing names.
//! - An image failure is reported once per node and asset (the last
//!   reported asset per node, forgotten when the node goes, so a removed
//!   node keeps no bytes alive); a missing family once per family, at the
//!   first text node measured with it.
//! - Nodes and scenes pass their neutral `Asset` sources directly to the store;
//!   no converted image-source copy is kept on a node.
//! - Images are drawn at the device pixels of the rect they are drawn in
//!   (× the frame's scale, × the transforms' zoom); an SVG is rasterized
//!   again at that size.
//! - An image that cannot be read draws nothing, and a family that is not
//!   installed draws in the system UI font; each sends one
//!   `AssetEvent::AssetFailed` (the client reports it located at the node's
//!   creation, to the default sink only). The failure of a scene's image or
//!   of a background image is not reported.
//! - A font file is registered at once, while its write is applied
//!   (`HostRequest::RegisterFont`: `Engine::register_font`), so the
//!   next layout measures with it; the answer is its families
//!   (`PlatformResult::Fonts`) or why it failed.
//! - Thread: lives on the UI thread with the model; only the store's
//!   worker thread runs loads, decodes and rasters, and wakes the window's
//!   event loop through the host's waker when it finishes; the next turn
//!   applies its work.

use std::collections::HashMap;

use crate::lower::AssetEvent;
use arvane::asset::{
    AnimationInfo, AssetStore, FamilyChoice, ImageState, PixelSize, Region, load_font,
};
use kivact_types::{Asset, ComputedTextStyle, NodeId, PlatformResult, Rgba, Size};

/// UI-thread resource owner with image and font diagnostic deduplication.
/// Owner keys are supplied by the caller; this manager never traverses a tree.
pub(crate) struct ArvaneAssets {
    fonts: kivact_types::cell::Generation,
    store: AssetStore,
    revision: u64,
    /// Families of the registered font files, in registration order (for
    /// the missing-family diagnostic).
    registered: Vec<String>,
    /// The last failed asset reported for each `Image` node.
    reported_images: HashMap<NodeId, Asset>,
    /// Missing families already reported (lowercase).
    reported_families: kivact_types::cell::ReportOnceSet<String>,
    /// Reports found during a layout pass (missing families).
    pending: Vec<AssetEvent>,
}

impl ArvaneAssets {
    /// Own an image store with empty font-registration and report histories.
    pub(crate) fn new(store: AssetStore) -> Self {
        ArvaneAssets {
            fonts: kivact_types::cell::Generation::default(),
            store,
            revision: 0,
            registered: Vec::new(),
            reported_images: HashMap::new(),
            reported_families: kivact_types::cell::ReportOnceSet::new(),
            pending: Vec::new(),
        }
    }

    pub(crate) fn fonts(&self) -> kivact_types::cell::Generation {
        self.fonts
    }

    pub(crate) fn revision(&self) -> u64 {
        self.revision
    }

    fn changed(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("invariant: resource revision fits u64");
    }

    /// Start one image-use frame, updating only the store's recency clock.
    pub(crate) fn begin_frame(&mut self) {
        self.store.begin_frame();
    }

    /// Release unused decoded assets and excess storage, retaining font
    /// registrations and diagnostic histories. Evictions follow the normal
    /// poll path (Arvane retains no separate device textures).
    pub(crate) fn release_caches(&mut self) {
        self.store.release_caches();
        self.registered.shrink_to_fit();
        self.reported_images.shrink_to_fit();
        self.pending.shrink_to_fit();
    }

    /// Consume the store's failure queue without touching evictions or reports.
    pub(crate) fn discard_failures(&mut self) {
        self.store.take_failures();
    }

    /// Current readiness without starting a load or consuming any notices.
    pub(crate) fn image_status(&self, asset: &Asset) -> kivact_types::ImageStatus {
        match self.status(asset) {
            Some(ImageState::Ready { .. }) => kivact_types::ImageStatus::Ready,
            Some(ImageState::Failed(_)) => kivact_types::ImageStatus::Failed,
            Some(ImageState::Pending) | None => kivact_types::ImageStatus::Loading,
        }
    }

    pub(crate) fn status(&self, asset: &Asset) -> Option<ImageState> {
        self.store.peek(asset)
    }

    // ----- images -----------------------------------------------------------

    /// Whether the store's answer for an `Image` node's `image` now
    /// differs from the natural size its layout used, without starting a
    /// load (an image whose pixels were evicted keeps its size; one that
    /// left the cache is laid out as before until it is asked for again).
    pub(crate) fn natural_changed(
        &self,
        image: Option<&kivact_types::Asset>,
        used: Option<Size>,
    ) -> bool {
        let Some(asset) = image else {
            return false;
        };
        match self.store.peek(asset) {
            Some(ImageState::Ready { natural }) => used != Some(natural),
            Some(ImageState::Failed(_)) => used.is_some(),
            Some(ImageState::Pending) | None => false,
        }
    }

    /// The natural size of an `Image` node's `image`, once loaded (the
    /// layout's lookup: starts the load when needed).
    pub(crate) fn natural_size(&mut self, image: Option<&kivact_types::Asset>) -> Option<Size> {
        match self.store.image(image?) {
            ImageState::Ready { natural } => Some(natural),
            ImageState::Pending | ImageState::Failed(_) => None,
        }
    }

    /// The natural size of `asset` once loaded (starts the load when
    /// needed).
    pub(crate) fn natural(&mut self, asset: &kivact_types::Asset) -> Option<Size> {
        match self.store.image(asset) {
            ImageState::Ready { natural } => Some(natural),
            ImageState::Pending | ImageState::Failed(_) => None,
        }
    }

    /// The timing of the animated `asset` (`None` while it is not ready or
    /// is not an animation).
    pub(crate) fn animation(&mut self, asset: &kivact_types::Asset) -> Option<AnimationInfo> {
        self.store.animation(asset)
    }

    /// The image to draw for the part `visible` of `asset` drawn at
    /// `device` pixels, in `tint` when given, at animation frame `frame`
    /// (0 for a still image), and the part of the image it covers (`None`:
    /// nothing to draw yet). Shares the store's pixels.
    pub(crate) fn pixels(
        &mut self,
        asset: &kivact_types::Asset,
        device: PixelSize,
        visible: Region,
        tint: Option<Rgba>,
        frame: u32,
    ) -> Option<(kivact_types::Pixels, Region)> {
        if device.is_empty() {
            return None;
        }
        let drawn = self.store.pixels_in(asset, device, visible, tint, frame);
        // Arvane keeps no device textures: evicted pixels need no work.
        self.store.take_evicted();
        let (pixels, region) = drawn?;
        let image = kivact_types::Pixels {
            width: pixels.width(),
            height: pixels.height(),
            rgba: kivact_types::AssetBytes::from_shared(std::sync::Arc::clone(pixels.rgba())),
        };
        Some((image, region))
    }

    /// Applies the store's finished work: whether anything changed, and
    /// whether any asset failed since (the image nodes are checked then;
    /// the store names failures by its cache keys, which a relative path's
    /// prop does not equal).
    pub(crate) fn poll(&mut self) -> (bool, bool) {
        let changed = self.store.poll();
        self.store.take_evicted();
        let failed = !self.store.take_failures().is_empty();
        if changed || failed {
            self.changed();
        }
        (changed, failed)
    }

    /// The report for `Image` node `id` showing `asset` (the source in the
    /// store) when it failed and was not reported for this node yet.
    pub(crate) fn check_image(&mut self, id: NodeId, asset: &Asset) -> Option<AssetEvent> {
        if self
            .reported_images
            .get(&id)
            .is_some_and(|reported| reported != asset)
        {
            self.reported_images.remove(&id);
        }
        match self.store.peek(asset) {
            Some(ImageState::Failed(e)) if self.reported_images.get(&id) != Some(asset) => {
                self.reported_images.insert(id, asset.clone());
                Some(AssetEvent::AssetFailed {
                    node: id,
                    message: e.message().to_owned(),
                })
            }
            _ => None,
        }
    }

    /// Forget the image reports of nodes that no longer exist (at a
    /// commit: removed nodes must not keep their assets' bytes alive).
    pub(crate) fn prune_reports(&mut self, exists: impl Fn(NodeId) -> bool) {
        if !self.reported_images.is_empty() {
            self.reported_images.retain(|id, _| exists(*id));
        }
    }

    // ----- fonts ------------------------------------------------------------

    /// Register the font file `asset` with Arvane
    /// (`HostRequest::RegisterFont`): its families, or why it failed.
    pub(crate) fn register_font(
        &mut self,
        engine: &mut arvane::Engine,
        asset: &Asset,
    ) -> PlatformResult {
        let file = match load_font(asset) {
            Ok(file) => file,
            Err(e) => {
                return PlatformResult::Failed(kivact_types::Failure::new(
                    kivact_types::FailureKind::Resource,
                    e.message(),
                ));
            }
        };
        let bytes = file.bytes().to_shared();
        if let Err(e) = engine.register_font(bytes.clone()) {
            return PlatformResult::Failed(kivact_types::Failure::new(
                kivact_types::FailureKind::Resource,
                format!(
                    "the text system did not accept the font file ({}): {e}",
                    file.families().join(", ")
                ),
            ));
        }
        self.store.register_font(bytes);
        for family in file.families() {
            if !self.registered.contains(family) {
                self.registered.push(family.clone());
            }
        }
        self.fonts.next();
        self.changed();
        PlatformResult::Fonts(file.families().to_vec())
    }

    /// Resolve the family text node `id` is measured with; a missing one
    /// is reported once per family (queued for after the layout pass).
    pub(crate) fn check_family(
        &mut self,
        engine: &mut arvane::Engine,
        id: NodeId,
        family: Option<&str>,
    ) {
        let Some(name) = family else {
            return;
        };
        let (_, missing) = resolve(engine, name);
        if missing && self.reported_families.note(name.to_lowercase()) {
            let registered = match self.registered.as_slice() {
                [] => "none".to_owned(),
                r => r.join(", "),
            };
            self.pending.push(AssetEvent::AssetFailed {
                node: id,
                message: format!(
                    "font family `{name}` is not installed; the system UI font is used \
                     instead (registered families: {registered})"
                ),
            });
        }
    }

    /// The reports found during the last layout pass.
    pub(crate) fn take_pending(&mut self) -> Vec<AssetEvent> {
        std::mem::take(&mut self.pending)
    }
}

/// The family Arvane shapes with for `name` (`None`: the system UI font)
/// and whether `name` is a missing family.
fn resolve(engine: &mut arvane::Engine, name: &str) -> (Option<String>, bool) {
    let resolution = engine.resolve_font_family(name);
    let family = match resolution.choice {
        FamilyChoice::Family(f) => Some(f),
        FamilyChoice::SystemUi => None,
    };
    (family, resolution.missing)
}

/// The family Arvane shapes with for a requested family name or generic
/// family (`None`: the system UI font, Parley's `system-ui`).
pub(crate) fn family(engine: &mut arvane::Engine, name: Option<&str>) -> Option<String> {
    name.and_then(|name| resolve(engine, name).0)
}

/// The Arvane text style for `props`, its family resolved.
pub(crate) fn text_style(
    engine: &mut arvane::Engine,
    props: &ComputedTextStyle,
) -> kivact_types::ComputedTextStyle {
    let mut style = props.clone();
    style.family = family(engine, props.family.as_deref());
    style
}
