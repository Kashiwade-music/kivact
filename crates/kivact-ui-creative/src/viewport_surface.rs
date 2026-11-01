//! Role: Display application pixels with shared transforms and semantic regions.

use crate::editors::shared::targets;
use kivact::control::{Node, NodeBuilder, node};
use kivact::draw::{Draw, ImageOptions, Layer, Pixels, Point, Rect, RegionId, Size, Transform};
#[cfg(feature = "gpu-interop")]
use kivact::gpu_interop::GpuFrame;
use kivact::interaction::{ViewTransform, use_zoom_pan};
use kivact::primitive::{NodeExt, part};
use kivact::style::{ColorToken, Overflow, style};
use kivact::ui::EditPhase;
use kivact::view::HandlerResult;
use kivact::view::{
    Handled, IntoValue, IntoView, Modifiers, MouseButton, PointerId, PointerPhase, Role, View,
    callback, node_ref,
};
use kivact::{Binding, Callback, Value};
use std::rc::Rc;

/// The display area in logical and physical pixels.
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SurfaceSize {
    pub logical: Size,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f32,
}

/// Pointer input in image coordinates, after the display transform is inverted.
/// Produced by the framework; read its fields.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct SurfacePointer {
    pub position: Point,
    pub phase: PointerPhase,
    pub button: Option<MouseButton>,
    pub modifiers: Modifiers,
    pub pointer_id: PointerId,
    pub region: Option<u64>,
}

/// An application-owned semantic rectangle in image coordinates.
/// Built with [`SurfaceRegion::new`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SurfaceRegion {
    pub id: u64,
    pub name: String,
    pub role: Role,
    pub bounds: Rect,
}
impl SurfaceRegion {
    pub fn new(id: u64, name: impl Into<String>, role: Role, bounds: Rect) -> Self {
        Self {
            id,
            name: name.into(),
            role,
            bounds,
        }
    }
}

type Overlay = Rc<dyn Fn(&mut Draw, ViewTransform)>;

/// An image display and interaction surface; the application renders its contents.
pub struct ViewportSurface {
    root: Node,
    image: Value<Option<Pixels>>,
    transform: Option<Binding<ViewTransform>>,
    resize: Option<Callback<SurfaceSize>>,
    pointer: Option<Callback<SurfacePointer>>,
    overlay: Option<Overlay>,
    regions: Value<Vec<SurfaceRegion>>,
    pixelated: Value<bool>,
    on_edit: Option<Callback<EditPhase>>,
    #[cfg(feature = "gpu-interop")]
    gpu_frame: Value<Option<GpuFrame>>,
}

/// What the surface shows now: the application's `Pixels`, or (feature
/// `gpu-interop`) a frame of an image shared on the GPU.
enum Shown {
    Pixels(Pixels),
    #[cfg(feature = "gpu-interop")]
    Gpu(GpuFrame),
}
impl Shown {
    /// The size in pixels of the image shown, which fit, zoom, pan, pointer
    /// coordinates, overlay and regions use as the natural size.
    fn size(&self) -> Size {
        let (width, height) = match self {
            Self::Pixels(image) => (image.width, image.height),
            #[cfg(feature = "gpu-interop")]
            Self::Gpu(frame) => (frame.width(), frame.height()),
        };
        Size::new(width as f32, height as f32)
    }
}

#[track_caller]
pub fn viewport_surface(image: impl IntoValue<Option<Pixels>>) -> ViewportSurface {
    ViewportSurface {
        root: node().role(Role::Canvas).focusable(true).style(
            style()
                .height(280)
                .overflow(Overflow::Hidden)
                .background(ColorToken::Surface),
        ),
        image: image.into_value(),
        transform: None,
        resize: None,
        pointer: None,
        overlay: None,
        regions: Vec::new().into_value(),
        pixelated: false.into_value(),
        on_edit: None,
        #[cfg(feature = "gpu-interop")]
        gpu_frame: None::<GpuFrame>.into_value(),
    }
}
impl ViewportSurface {
    /// Observe edits to the optional shared transform.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.on_edit = Some(callback(f));
        self
    }
    /// Bind shared zoom/pan. Without a binding, the image always fits the surface.
    pub fn transform(mut self, transform: impl Into<Binding<ViewTransform>>) -> Self {
        self.transform = Some(transform.into());
        self
    }
    pub fn on_resize<R: HandlerResult>(mut self, f: impl Fn(SurfaceSize) -> R + 'static) -> Self {
        self.resize = Some(callback(f));
        self
    }
    pub fn on_pointer<R: HandlerResult>(
        mut self,
        f: impl Fn(SurfacePointer) -> R + 'static,
    ) -> Self {
        self.pointer = Some(callback(f));
        self
    }
    /// Draw overlays in image coordinates through the same transform as the pixels.
    pub fn overlay(mut self, overlay: impl Fn(&mut Draw, ViewTransform) + 'static) -> Self {
        self.overlay = Some(Rc::new(overlay));
        self
    }
    pub fn regions(mut self, regions: impl IntoValue<Vec<SurfaceRegion>>) -> Self {
        self.regions = regions.into_value();
        self
    }
    /// Use nearest-neighbor sampling while enlarged beyond the image's native size.
    pub fn pixelated(mut self, pixelated: impl IntoValue<bool>) -> Self {
        self.pixelated = pixelated.into_value();
        self
    }
    /// Show a frame of an image the application's render engine shares on
    /// the GPU (`kivact::gpu_interop`, experimental), without reading it
    /// back. Feature `gpu-interop`.
    ///
    /// While `frame` is `Some` and `kivact::gpu_interop::adapter()` is
    /// `Some`, the surface shows that frame; otherwise it shows the
    /// `Pixels` given to [`viewport_surface`], as it does without this
    /// call. The image shown, whichever it is, is the one that fit, zoom,
    /// pan, the pointer's image coordinates, the overlay and the regions
    /// take their size from (a frame's is [`GpuFrame::width`] by
    /// [`GpuFrame::height`]). `pixelated` applies to both.
    ///
    /// Keep a `Pixels` image of the same picture for the other case: the
    /// window's GPU device is not ready yet (the adapter is `None` at
    /// first), the window draws on the CPU, or the import is `Lost` and
    /// the application has no frame (`None`) until it imports again. The
    /// surface does not look at the import's status; the application sets
    /// the frame to `None` when it has none to show.
    ///
    /// A `kivact::gpu_interop::shared_image` does all of that: its
    /// `frame()` is `None` unless the engine's frames are imported, and its
    /// `pixels()` are the engine's pictures drawn on the CPU while the
    /// window cannot import. The engine's thread takes `image.link()`
    /// (see the `kivact::gpu_interop` module); the surface's physical size
    /// is the size to ask for.
    ///
    /// ```ignore
    /// use kivact::gpu_interop::shared_image;
    /// use kivact_ui_creative::viewport_surface;
    ///
    /// let image = shared_image(1280, 720);
    /// let (frame, pixels, sized) = (image.clone(), image.clone(), image);
    /// viewport_surface(move || pixels.pixels())
    ///     .gpu_frame(move || frame.frame())
    ///     .on_resize(move |size| sized.set_size(size.physical_width, size.physical_height))
    /// ```
    #[cfg(feature = "gpu-interop")]
    pub fn gpu_frame(mut self, frame: impl IntoValue<Option<GpuFrame>>) -> Self {
        self.gpu_frame = frame.into_value();
        self
    }
}
impl NodeBuilder for ViewportSurface {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

#[derive(Clone, PartialEq)]
struct SemanticRegion {
    id: RegionId,
    data: SurfaceRegion,
}

impl IntoView for ViewportSurface {
    fn into_view(self) -> View {
        let Self {
            root,
            image,
            transform,
            resize,
            pointer,
            overlay,
            regions,
            pixelated,
            on_edit,
            #[cfg(feature = "gpu-interop")]
            gpu_frame,
        } = self;
        let reference = node_ref();
        let rect = reference.rect();
        // Whether the window's GPU device can import: only then a frame shows.
        #[cfg(feature = "gpu-interop")]
        let gpu = kivact::memo(|| kivact::gpu_interop::adapter().is_some());
        let shown = move || -> Option<Shown> {
            #[cfg(feature = "gpu-interop")]
            if let Some(frame) = gpu_frame.get().filter(|_| gpu.get()) {
                return Some(Shown::Gpu(frame));
            }
            image.get().map(Shown::Pixels)
        };
        let content =
            kivact::memo(move || shown().map_or(Size::new(0.0, 0.0), |shown| shown.size()));
        let view = match transform {
            Some(transform) => transform.into_value(),
            None => kivact::memo(move || {
                ViewTransform::fit(
                    content.get(),
                    rect.get()
                        .map_or(Size::new(0.0, 0.0), |r| Size::new(r.width, r.height)),
                )
            }),
        };
        // Var reads are untracked, so this cache does not subscribe the effect.
        let previous = kivact::var(None::<SurfaceSize>);
        kivact::effect(move || {
            let Some(rect) = rect.get() else {
                return;
            };
            let scale_factor = kivact::platform::scale_factor();
            let size = SurfaceSize {
                logical: Size::new(rect.width, rect.height),
                scale_factor,
                physical_width: (rect.width * scale_factor).round().max(0.0) as u32,
                physical_height: (rect.height * scale_factor).round().max(0.0) as u32,
            };
            if previous.get() != Some(size) {
                previous.set(Some(size));
                if let Some(callback) = resize {
                    callback.call(size);
                }
            }
        });
        let scene = targets::targets(move || {
            Ok(regions.get().into_iter().map(|r| (r.id, r.role)).collect())
        });
        let semantics = kivact::memo(move || {
            scene.with(|scene| match scene {
                Ok(scene) => regions
                    .get()
                    .into_iter()
                    .filter_map(|data| {
                        scene
                            .id(&(data.id, data.role))
                            .map(|id| SemanticRegion { id, data })
                    })
                    .collect::<Vec<_>>(),
                Err(_) => Vec::new(),
            })
        });
        let root = root
            .node_ref(reference)
            .regions(
                semantics,
                |r| r.id,
                move |_, data| {
                    let role = data.get_untracked().data.role;
                    part(|input| input)
                        .role(role)
                        .name(move || data.get().data.name)
                },
            )
            .on_pointer_default(move |event| {
                if let Some(callback) = pointer {
                    callback.call(SurfacePointer {
                        position: view.get().to_content(event.pos),
                        phase: event.phase,
                        button: event.button,
                        modifiers: event.modifiers,
                        pointer_id: event.id,
                        region: event.region().and_then(|id| {
                            scene.with(|scene| {
                                scene
                                    .as_ref()
                                    .ok()
                                    .and_then(|scene| scene.target(id).map(|target| target.0))
                            })
                        }),
                    });
                }
                Handled::No
            })
            .draw(move |draw, _| {
                let transform = view.get();
                draw.layer(
                    Layer::new().transform(
                        Transform::new()
                            .translate(transform.offset.x, transform.offset.y)
                            .scale(transform.scale),
                    ),
                    |draw| {
                        if let Some(shown) = shown() {
                            let size = shown.size();
                            let rect = Rect::new(0.0, 0.0, size.width, size.height);
                            let options = ImageOptions {
                                pixelated: pixelated.get() && transform.scale > 1.0,
                                ..Default::default()
                            };
                            match shown {
                                Shown::Pixels(image) => draw.pixels(&image, rect, options),
                                #[cfg(feature = "gpu-interop")]
                                Shown::Gpu(frame) => draw.gpu_frame(&frame, rect, options),
                            }
                        }
                        if let Some(overlay) = &overlay {
                            overlay(draw, transform);
                        }
                        for region in semantics.get() {
                            draw.region(region.id, region.data.bounds);
                        }
                    },
                );
            });
        match transform {
            Some(transform) => {
                let behavior = use_zoom_pan(transform).content_size(content);
                let behavior = if let Some(observer) = on_edit {
                    behavior.on_edit(move |value| observer.try_call(value))
                } else {
                    behavior
                };
                root.part(behavior.viewport()).into_view()
            }
            None => root.into_view(),
        }
    }
}
