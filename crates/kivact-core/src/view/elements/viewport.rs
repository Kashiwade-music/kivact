//! Role: Display application-owned pixels through the standard canvas pipeline.
use crate::draw::{ImageOptions, Pixels};
use crate::geometry::{Rect, Size};
use crate::gpu_interop::GpuFrame;
use crate::state::{IntoValue, Value, callback, effect, memo};
use crate::view::{
    HandlerResult, ImageFit, IntoView, Node, NodeBuilder, Role, View, node, node_ref,
};
use kivact_types::ViewTransform;
use kivact_types::cell::Sent;
use std::cell::RefCell;

// xtask-allow(internal-contract): Boundary adapter lets interaction attach its single zoom/pan model without a core dependency.
#[doc(hidden)]
pub trait ViewportTransform: 'static {
    fn view_transform(&self) -> Value<ViewTransform>;
    fn attach(self, node: Node, content: Value<Rect>) -> Node;
}
type Attach = Box<dyn FnOnce(Node, Value<Rect>) -> Node>;

/// The surface's measured logical size and physical rendering extent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSize {
    pub logical: Size,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
}

// xtask-allow(state-fields): Immutable NodeBuilder inputs select independent display and attachment capabilities; the attached zoom/pan machine owns all interaction phases.
/// Application pixels painted by a canvas with standard node modifiers.
pub struct Viewport {
    root: Node,
    pixels: Value<Pixels>,
    fit: Value<ImageFit>,
    transform: Option<(Value<ViewTransform>, Attach)>,
    pixelated: Value<bool>,
    resize: Option<crate::state::Callback<SurfaceSize>>,
    gpu_frame: Value<Option<GpuFrame>>,
}

/// Display pixels, centered with `ImageFit::Contain` by default.
/// Add input, capture, accessible regions and overlays through standard node modifiers.
#[track_caller]
pub fn viewport(pixels: impl IntoValue<Pixels>) -> Viewport {
    Viewport {
        root: node()
            .role(Role::Canvas)
            .style(crate::style::style().overflow(crate::style::Overflow::Hidden)),
        pixels: pixels.into_value(),
        fit: ImageFit::Contain.into_value(),
        transform: None,
        pixelated: false.into_value(),
        resize: None,
        gpu_frame: None::<GpuFrame>.into_value(),
    }
}

impl Viewport {
    /// Share the pose and input behavior owned by a zoom/pan controller.
    pub fn transform(mut self, api: impl ViewportTransform) -> Self {
        self.transform = Some((
            api.view_transform(),
            Box::new(move |node, content| api.attach(node, content)),
        ));
        self
    }
    /// Choose how the image fills the surface; excess content is clipped.
    pub fn fit(mut self, fit: impl IntoValue<ImageFit>) -> Self {
        self.fit = fit.into_value();
        self
    }
    /// Use nearest-neighbor sampling when enlarging pixels.
    pub fn pixelated(mut self, pixelated: impl IntoValue<bool>) -> Self {
        self.pixelated = pixelated.into_value();
        self
    }
    /// Report changed physical rendering extents after layout or scale changes.
    pub fn on_resize<R: HandlerResult>(mut self, f: impl Fn(SurfaceSize) -> R + 'static) -> Self {
        self.resize = Some(callback(f));
        self
    }
    /// Display a shared GPU frame when the window supports import; retain the
    /// supplied pixels for CPU rendering and subtree snapshots.
    #[cfg(feature = "gpu-interop")]
    pub fn gpu_frame(mut self, frame: impl IntoValue<Option<GpuFrame>>) -> Self {
        self.gpu_frame = frame.into_value();
        self
    }
}
impl NodeBuilder for Viewport {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}
impl IntoView for Viewport {
    fn into_view(self) -> View {
        let Self {
            root,
            pixels,
            fit,
            transform,
            pixelated,
            resize,
            gpu_frame,
        } = self;
        let reference = node_ref();
        let rect = reference.rect();
        let size = memo(move || {
            let rect = rect.get()?;
            let scale_factor = crate::platform::scale_factor().get();
            Some(SurfaceSize {
                logical: Size::new(rect.width, rect.height),
                scale_factor,
                physical_width: (rect.width * scale_factor).round().max(0.0) as u32,
                physical_height: (rect.height * scale_factor).round().max(0.0) as u32,
            })
        });
        let sent = RefCell::new(Sent::new());
        effect(move || {
            if let Some(resize) = resize
                && let Some(size) = size.get()
                && sent.borrow_mut().update(size).is_some()
            {
                resize.call(size);
            }
        });
        let content = memo(move || {
            if let Some(frame) = gpu_frame
                .get()
                .filter(|_| crate::gpu_interop::adapter().is_some())
            {
                return Rect::new(0.0, 0.0, frame.width() as f64, frame.height() as f64);
            }
            let image = pixels.get();
            Rect::new(0.0, 0.0, image.width as f64, image.height as f64)
        });
        let (pose, attach) = match transform {
            Some((pose, attach)) => (Some(pose), Some(attach)),
            None => (None, None),
        };
        let root = root.node_ref(reference).draw(move |draw, size| {
            let image = pixels.get();
            let bounds = content.get();
            let natural = Size::new(bounds.width, bounds.height);
            let destination = pose.map_or_else(
                || fit_rect(natural, size, fit.get()),
                |pose| {
                    let pose = pose.get();
                    Rect::new(
                        pose.offset.x,
                        pose.offset.y,
                        natural.width * pose.scale,
                        natural.height * pose.scale,
                    )
                },
            );
            let options = ImageOptions {
                pixelated: pixelated.get(),
                ..Default::default()
            };
            if let Some(frame) = gpu_frame
                .get()
                .filter(|_| crate::gpu_interop::adapter().is_some())
            {
                draw.gpu_frame_with_pixels(&frame, &image, destination, options);
                return;
            }
            draw.pixels(&image, destination, options);
        });
        match attach {
            Some(attach) => attach(root, content).into_view(),
            None => root.into_view(),
        }
    }
}
fn fit_rect(content: Size, view: Size, fit: ImageFit) -> Rect {
    let contain = ViewTransform::fit(content, view);
    let scale = match fit {
        ImageFit::Contain => contain.scale,
        ImageFit::Cover if content.width > 0.0 && content.height > 0.0 => {
            (view.width / content.width).max(view.height / content.height)
        }
        ImageFit::ScaleDown => contain.scale.min(1.0),
        ImageFit::Fill => return Rect::new(0.0, 0.0, view.width, view.height),
        _ => 1.0,
    };
    let width = content.width * scale;
    let height = content.height * scale;
    Rect::new(
        (view.width - width) / 2.0,
        (view.height - height) / 2.0,
        width,
        height,
    )
}
