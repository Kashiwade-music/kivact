//! Role: Creates one winit window from a window request.

use crate::a11y::native::NativeAccessibility;
use crate::native_input::state::NativeInput;
use crate::port::winit::window::{Surface, WinitWindow};
use crate::present::CpuSurface;
use crate::services::Services;
use arvane_shell::{ShellError, ShellWaker};
#[cfg(any(windows, target_os = "linux"))]
use kivact_types::WindowMode;
use kivact_types::{Rect, Size, WindowPlacement, WindowRequest};
use std::num::NonZeroU64;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use winit::dpi::{LogicalSize, PhysicalPosition};
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

/// Create a hidden window; its caller shows it after the accepting callback.
pub(super) fn create_window(
    el: &ActiveEventLoop,
    request: &WindowRequest,
    parent: Option<&WinitWindow>,
    waker: &ShellWaker,
    started: Instant,
) -> Result<WinitWindow, ShellError> {
    let mut attributes = Window::default_attributes()
        .with_visible(false)
        .with_title(&request.title)
        .with_inner_size(LogicalSize::new(request.size.width, request.size.height));
    if let Some(min_size) = request.min_size {
        attributes =
            attributes.with_min_inner_size(LogicalSize::new(min_size.width, min_size.height));
    }
    let main_placement = match request.parent {
        None => {
            let logical = (request.size.width, request.size.height);
            let placement = crate::display::initial_window_position(el, logical);
            if let Some((position, _)) = &placement {
                attributes = attributes.with_position(*position);
            }
            placement
        }
        Some(_) => {
            attributes = place_child(el, attributes, request, parent);
            None
        }
    };
    attributes = with_mode(attributes, request, parent);
    let window = Arc::new(
        el.create_window(attributes)
            .map_err(|e| ShellError::new(e.to_string()))?,
    );
    if let Some((_, placement)) = &main_placement {
        crate::display::center_window(
            placement,
            &window,
            (request.size.width, request.size.height),
        );
    }
    disable_modal_parent(request, parent);
    let accessibility = NativeAccessibility::new(el, &window, waker.clone());
    let input = Rc::new(NativeInput::new(
        kivact_types::Environment {
            scale_factor: window.scale_factor(),
            reduced_motion: crate::native_input::reduced_motion(),
            pointer: crate::native_input::pointer_settings(window.scale_factor()),
            locale: crate::native_input::locale::system_locale(),
            ..Default::default()
        },
        #[cfg(windows)]
        Some(waker.clone()),
    ));
    let monitor = window.current_monitor();
    let mut window = WinitWindow {
        #[cfg(windows)]
        subclass: crate::native_input::Subclass::install(&window, &input),
        input,
        surface: Surface::Live {
            generation: NonZeroU64::MIN,
            cpu: CpuSurface::Released,
        },
        window,
        started,
        affinity: Rc::new(()),
        accessibility,
        services: Services::default(),
        monitor,
        frame_period_ns: kivact_types::defaults::FRAME_PERIOD_NS,
    };
    window.read_frame_period();
    Ok(window)
}

/// Position a child window next to its parent when the platform honours it.
fn place_child(
    el: &ActiveEventLoop,
    attributes: winit::window::WindowAttributes,
    request: &WindowRequest,
    parent: Option<&WinitWindow>,
) -> winit::window::WindowAttributes {
    if request.placement == WindowPlacement::Auto || !crate::display::placement_supported(el) {
        return attributes;
    }
    let Some(parent) = parent else {
        return attributes;
    };
    let (Ok(position), monitor) = (
        parent.window.outer_position(),
        parent.window.current_monitor(),
    ) else {
        return attributes;
    };
    let Some(monitor) = monitor else {
        return attributes;
    };
    let outer = parent.window.outer_size();
    let scale = parent.window.scale_factor();
    let parent_outer = Rect {
        x: f64::from(position.x),
        y: f64::from(position.y),
        width: f64::from(outer.width),
        height: f64::from(outer.height),
    };
    let size = Size {
        width: request.size.width * scale,
        height: request.size.height * scale,
    };
    let work_area = crate::display::work_area_of(&monitor);
    let Some(at) = crate::display::place_relative(parent_outer, size, work_area, request.placement)
    else {
        return attributes;
    };
    attributes.with_position(PhysicalPosition::new(
        at.x.round() as i32,
        at.y.round() as i32,
    ))
}

/// Give the window its platform mode: owned and taskbar-less tools, owned
/// modal dialogs, and X11's utility and dialog window types.
fn with_mode(
    attributes: winit::window::WindowAttributes,
    request: &WindowRequest,
    parent: Option<&WinitWindow>,
) -> winit::window::WindowAttributes {
    #[cfg(windows)]
    {
        use winit::platform::windows::WindowAttributesExtWindows;
        if let (Some(parent), Some(_)) = (parent, request.parent) {
            let hwnd = win32_handle(parent);
            let attributes = match request.mode {
                WindowMode::Normal => attributes,
                WindowMode::Tool => attributes.with_owner_window(hwnd).with_skip_taskbar(true),
                WindowMode::Modal => attributes.with_owner_window(hwnd),
            };
            return attributes;
        }
    }
    #[cfg(target_os = "linux")]
    {
        use winit::platform::x11::{WindowAttributesExtX11, WindowType};
        if request.parent.is_some() {
            let window_types = match request.mode {
                WindowMode::Normal => Vec::new(),
                WindowMode::Tool => vec![WindowType::Utility],
                WindowMode::Modal => vec![WindowType::Dialog],
            };
            if !window_types.is_empty() {
                return attributes.with_x11_window_type(window_types);
            }
        }
    }
    let _ = (request, parent);
    attributes
}

/// The Win32 window handle of a winit window.
#[cfg(windows)]
fn win32_handle(window: &WinitWindow) -> windows_sys::Win32::Foundation::HWND {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle = window
        .window
        .window_handle()
        .expect("invariant: a created window has a Win32 handle");
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        unreachable!("invariant: a Win32 window has a Win32 handle")
    };
    handle.hwnd.get()
}

/// Disable the parent of a modal window while it is open.
fn disable_modal_parent(request: &WindowRequest, parent: Option<&WinitWindow>) {
    #[cfg(windows)]
    {
        use winit::platform::windows::WindowExtWindows;
        if request.mode == WindowMode::Modal
            && request.parent.is_some()
            && let Some(parent) = parent
        {
            parent.window.set_enable(false);
        }
    }
    let _ = (request, parent);
}
