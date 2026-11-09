//! Role: Layer trees of one frame, handed from the engine to the compositor.
use super::scene::RenderScene;
use crate::color::Rgba;
use crate::effect::{FilterGraph, ShaderUse};
use crate::geometry::{Point, Rect, Transform};
use crate::geometry3d::Matrix4;
use crate::gpu::ExternalFrame;
use crate::ids::{DrawableId, NodeId};
use crate::paint::{BackfaceVisibility, MaskComposite, MaskMode};
use crate::pixel::{ColorMatrix, Pixels};
use crate::scene::Shape;

/// The most 3D context planes one context may hold. Above it the
/// engine and the compositor draw the planes flat, in tree order.
pub const MAX_CONTEXT_PLANES: usize = 256;

/// One frame: `layers[0]` is the window's root layer (window logical px).
/// Child layers are referenced by index from `LayerItem::Layer`. Layers
/// referenced only from `exports` are drawn into their export sinks.
#[derive(Clone, Debug, PartialEq)]
pub struct Composition {
    /// The layers; index 0 is the root.
    pub layers: Vec<CompositeLayer>,
    /// The drawable exports of this frame.
    pub exports: Vec<Export>,
}

impl Composition {
    /// Whether drawing this frame needs the window's GPU device: a layer
    /// applies a user shader as a layer filter, a backdrop filter or a paint
    /// shader. External GPU textures and GPU exports are demands of their
    /// own (the engine asks for the device when they are made, so they do
    /// not appear in a composition before the device is ready).
    pub fn needs_gpu(&self) -> bool {
        self.layers.iter().any(|layer| {
            layer.effects.filters.iter().any(is_shader)
                || layer
                    .effects
                    .backdrop
                    .as_ref()
                    .is_some_and(|backdrop| backdrop.filters.iter().any(is_shader))
                || layer
                    .items
                    .iter()
                    .any(|item| matches!(item, LayerItem::Shader(_)))
        })
    }
}

/// Whether `filter` is a user shader.
fn is_shader(filter: &LayerFilter) -> bool {
    matches!(filter, LayerFilter::Shader(_))
}

/// Stable identity of a layer across frames (cache key).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayerKey {
    /// The window's root layer.
    Root,
    /// The DevTools overlay of a window, drawn above everything and never hit-tested.
    Devtools,
    /// The layer of a tree node.
    Node(NodeId),
    /// A layer opened by a Draw recording: the node, its scene slot (0 draw,
    /// 1 draw_over) and the ordinal of the layer within the recording.
    Scene {
        /// The node that owns the recording.
        node: NodeId,
        /// Scene slot: 0 draw, 1 draw_over.
        slot: u8,
        /// Ordinal of the layer within the recording.
        ordinal: u32,
    },
    /// The layer of a drawable.
    Drawable(DrawableId),
}

/// One layer of a composition: its transform, bounds, effects and items.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeLayer {
    /// Stable identity across frames.
    pub key: LayerKey,
    /// Layer space → parent layer space (root: → window logical px).
    pub transform: Matrix4,
    /// Content bounds in layer space before filters (raster and effect extent).
    pub bounds: Rect,
    /// Whether the layer is flat or shares a 3D context with its parent.
    pub plane: Plane,
    /// Whether the back face of the layer is drawn.
    pub backface: BackfaceVisibility,
    /// Where the surface's pixel grid sits.
    pub grid: RasterGrid,
    /// Opacity, blending, filters, clip, masks and backdrop.
    pub effects: LayerEffects,
    /// What the layer draws, in order.
    pub items: Vec<LayerItem>,
}

/// Whether a layer is flat or takes part in a 3D context.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Plane {
    /// Flattened into its parent.
    #[default]
    Flat,
    /// Shares a 3D context with its parent.
    Preserve3d,
}

/// Where a layer's raster grid sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RasterGrid {
    /// Snap to the target's pixels: crisp, re-rastered when the fraction of
    /// the position changes.
    #[default]
    Target,
    /// Fixed in layer space: the raster survives any movement; the final
    /// draw places it with bilinear sampling (moving layers).
    Layer,
}

/// The effects a compositor layer applies to its whole surface.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerEffects {
    /// Group opacity.
    pub opacity: f64,
    /// Blend with what is beneath.
    pub blend: crate::pixel::BlendMode,
    /// Porter–Duff operator with what is beneath.
    pub composite: Composite,
    /// Filters applied in order to the surface.
    pub filters: Vec<LayerFilter>,
    /// Layer space.
    pub clip: Option<Shape>,
    /// Alpha masks over the surface.
    pub masks: Vec<RenderMask>,
    /// What lies behind the layer, filtered.
    pub backdrop: Option<RenderBackdrop>,
}

impl Default for LayerEffects {
    fn default() -> Self {
        LayerEffects {
            opacity: 1.0,
            blend: crate::pixel::BlendMode::Normal,
            composite: Composite::SourceOver,
            filters: Vec::new(),
            clip: None,
            masks: Vec::new(),
            backdrop: None,
        }
    }
}

/// A Porter–Duff compositing operator.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Composite {
    /// Source over destination.
    #[default]
    SourceOver,
    /// Source in destination.
    SourceIn,
    /// Source out destination.
    SourceOut,
    /// Source atop destination.
    SourceAtop,
    /// Destination over source.
    DestinationOver,
    /// Destination in source.
    DestinationIn,
    /// Destination out source.
    DestinationOut,
    /// Destination atop source.
    DestinationAtop,
    /// Additive.
    Lighter,
    /// Replace the destination.
    Copy,
    /// Exclusive or.
    Xor,
}

/// A filter of a compositor layer.
#[derive(Clone, Debug, PartialEq)]
pub enum LayerFilter {
    /// Gaussian blur.
    Blur {
        /// Standard deviation in layer space.
        std_dev: f64,
    },
    /// A blurred, offset silhouette beneath the content.
    DropShadow {
        /// Offset of the silhouette.
        offset: Point,
        /// Standard deviation of its blur.
        std_dev: f64,
        /// Color of the silhouette.
        color: Rgba,
    },
    /// A color matrix over the surface.
    ColorMatrix(ColorMatrix),
    /// Images resolved to pixels by the engine.
    // xtask-allow(api-shape): Shared immutable payload, cloned without copying its data; one allocation is shared by all clones.
    Graph(std::sync::Arc<FilterGraph<Pixels>>),
    /// A filter shader.
    Shader(ShaderUse),
}

/// A mask layer drawn as a scene in layer space; its alpha (or luminance × alpha) masks.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderMask {
    /// The mask's content.
    pub scene: RenderScene,
    /// Alpha or luminance.
    pub mode: MaskMode,
    /// How it combines with the layers below.
    pub composite: MaskComposite,
}

/// CSS backdrop-filter: what the layer covers in its parent, filtered and
/// clipped to `shape` (layer space).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderBackdrop {
    /// Where the backdrop shows.
    pub shape: Shape,
    /// The filters applied to what lies behind.
    pub filters: Vec<LayerFilter>,
}

/// One item of a compositor layer, drawn in order.
#[derive(Clone, Debug, PartialEq)]
pub enum LayerItem {
    /// Vello-drawn commands in layer space.
    Raster(RenderScene),
    /// A child layer, clipped by `clips` (parent layer space, all applied) when drawn into this layer.
    Layer {
        /// Index into `Composition::layers`.
        index: u32,
        /// Clips applied to the child, in parent layer space.
        clips: Vec<Shape>,
    },
    /// A frame of an external render engine's image.
    External(ExternalItem),
    /// A shader painted over a region.
    Shader(ShaderItem),
}

/// A frame of an external image drawn as a layer item.
#[derive(Clone, Debug, PartialEq)]
pub struct ExternalItem {
    /// The frame to draw.
    pub frame: ExternalFrame,
    /// Destination rectangle in item space.
    pub rect: Rect,
    /// Source pixels of the frame; None draws the whole frame.
    pub source: Option<Rect>,
    /// 2D map of `rect` into layer space.
    pub transform: Transform,
    /// Layer space, all applied.
    pub clips: Vec<Shape>,
    /// Alpha multiplier.
    pub opacity: f64,
    /// Disable smoothing.
    pub pixelated: bool,
    /// Pixels the CPU compositor draws instead (Draw::gpu_frame_with_pixels).
    pub cpu: Option<Pixels>,
}

/// A shader drawn as a layer item.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderItem {
    /// The shader with its parameters, images and time.
    pub shader: ShaderUse,
    /// The shader's coordinate box (position 0..size in WGSL), in item space.
    pub rect: Rect,
    /// Item space → layer space.
    pub transform: Transform,
    /// White where the shader shows, as a scene in item space (shape, clips).
    pub coverage: RenderScene,
    /// What the CPU compositor draws instead, in item space.
    pub fallback: RenderScene,
    /// Alpha multiplier.
    pub opacity: f64,
}

/// Draw `layers[layer]` into the export sink of `sink` at `size` device px
/// (`scale` device px per logical px), answering request `generation`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Export {
    /// The layer to draw.
    pub layer: u32,
    /// The drawable that receives the frame.
    pub sink: DrawableId,
    /// Size in device px.
    pub size: (u32, u32),
    /// Device px per logical px.
    pub scale: f64,
    /// The request this answers.
    pub generation: u64,
    /// Pixels or a GPU texture.
    pub mode: ExportMode,
}

/// A rectangle of device pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PixelRect {
    /// Left edge.
    pub x: u32,
    /// Top edge.
    pub y: u32,
    /// Width.
    pub width: u32,
    /// Height.
    pub height: u32,
}

/// How an export delivers its frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportMode {
    /// Read back as pixels.
    Pixels,
    /// Shared as a GPU texture.
    Gpu,
}

/// An exported drawable frame; the generation travels in ExportedFrame.
#[derive(Clone, Debug, PartialEq)]
pub enum ExportFrame {
    /// The frame as pixels.
    Pixels(Pixels),
    /// The frame as a slot of a shared pool, released when the fence reaches `value`.
    Gpu {
        /// The pool slot.
        slot: u32,
        /// The fence value that signals the frame is ready.
        value: u64,
    },
}

/// One exported frame of the request `generation` of `sink`, from the
/// compositor through arvane and the engine to core unchanged.
#[derive(Clone, Debug, PartialEq)]
pub struct ExportedFrame {
    /// The drawable that receives the frame.
    pub sink: DrawableId,
    /// The request this answers.
    pub generation: u64,
    /// The frame itself.
    pub frame: ExportFrame,
}

/// A GPU export pool lost with the device, of the request `generation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExportLoss {
    /// The drawable whose pool was lost.
    pub sink: DrawableId,
    /// The request the pool served.
    pub generation: u64,
}

/// What a presented frame produced besides pixels.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FrameReport {
    /// Exported frames of this presentation.
    pub exports: Vec<ExportedFrame>,
    /// External items drawn without their frame.
    pub missed: Vec<ExternalFrame>,
    /// (program hash, message) of shader pipelines that failed, once each.
    pub shader_failures: Vec<(u64, String)>,
    /// Exports skipped this frame because no slot was released.
    pub pending_exports: Vec<DrawableId>,
    /// Work counts of this frame.
    pub stats: FrameStats,
}

/// Work a frame did (statistics, S-9), for diagnostics and tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct FrameStats {
    /// Chunks rastered by Vello.
    pub chunks_rastered: u32,
    /// Pixels rastered.
    pub pixels_rastered: u64,
    /// Surfaces reused from the cache.
    pub surfaces_reused: u32,
    /// Area of the damage.
    pub damage_pixels: u64,
    /// Targets used by the frame.
    pub targets: u32,
}
