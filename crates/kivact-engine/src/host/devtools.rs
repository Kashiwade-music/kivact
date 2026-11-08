//! Role: Host-side DevTools state: overlays, pick, keys and emulation.
use super::pump::Host;
use crate::contract::{
    DevtoolsChange, DevtoolsKeys, DevtoolsOverlay, EngineEmulation, HostEvent, KeyChord,
    OverlayBox, PickPhase,
};
use arvane::{Engine, ShellEvent, Windows};
use kivact_types::geometry3d::Matrix4;
use kivact_types::render::{
    CompositeLayer, Composition, LayerEffects, LayerItem, LayerKey, Paint, Plane, RasterGrid,
    RenderCmd, RenderScene,
};
use kivact_types::{
    ComputedTextStyle, Environment, Key, KeyInput, KeyPhase, MouseButton, NodeId, Point,
    PointerClass, PointerPhase, Rect, Rgba, Size, WindowId,
};
use std::collections::BTreeMap;

/// The DevTools state the host applies to its windows.
#[derive(Default)]
pub(super) struct DevtoolsHost {
    pub(super) overlays: BTreeMap<WindowId, DevtoolsOverlay>,
    pub(super) pick: Option<WindowId>,
    pub(super) keys: DevtoolsKeys,
    pub(super) emulation: EngineEmulation,
}

/// Apply the DevTools emulation to the environment of a window.
pub(super) fn apply_emulation(emulation: EngineEmulation, env: &mut Environment) {
    if let Some(theme) = emulation.theme_hint {
        env.theme_hint = theme;
    }
    if let Some(scale) = emulation.scale.filter(valid_scale) {
        env.scale_factor = scale;
    }
    if let Some(touch) = emulation.touch {
        env.pointer.primary = if touch {
            PointerClass::Coarse
        } else {
            PointerClass::Fine
        };
    }
}

/// The logical viewport of a window whose native logical size is `size` at
/// `native_scale`, with an emulated `scale` override.
pub(super) fn emulated_viewport(size: Size, native_scale: f64, scale: Option<f64>) -> Size {
    match scale.filter(valid_scale) {
        Some(scale) => Size {
            width: size.width * native_scale / scale,
            height: size.height * native_scale / scale,
        },
        None => size,
    }
}

/// The scale to draw at: the emulated one when there is a valid override.
pub(super) fn effective_scale(native: f64, scale: Option<f64>) -> f64 {
    scale.filter(valid_scale).unwrap_or(native)
}

fn valid_scale(scale: &f64) -> bool {
    scale.is_finite() && *scale > 0.0
}

/// Whether `input` is the key chord `chord` (on a press).
pub(super) fn chord_matches(chord: &KeyChord, input: &KeyInput) -> bool {
    if input.modifiers != chord.modifiers || !matches!(input.phase, KeyPhase::Down { .. }) {
        return false;
    }
    let Key::Char(wanted) = chord.key else {
        return input.key == chord.key;
    };
    match input.key {
        Key::Char(typed) => typed.to_lowercase().eq(wanted.to_lowercase()),
        Key::Unidentified => super::events::physical_letter(input.code)
            .is_some_and(|typed| typed.to_lowercase().eq(wanted.to_lowercase())),
        _ => false,
    }
}

/// Add the DevTools overlay of `overlay` to `composition` as the layer
/// `LayerKey::Devtools`, referenced last from the root layer.
pub(super) fn append_overlay_layer(
    overlay: &DevtoolsOverlay,
    composition: &mut Composition,
    engine: &mut Engine,
    size: Size,
) {
    if composition.layers.is_empty() {
        return;
    }
    let index = composition.layers.len() as u32;
    composition.layers.push(CompositeLayer {
        key: LayerKey::Devtools,
        transform: Matrix4::IDENTITY,
        bounds: Rect {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        },
        plane: Plane::Flat,
        backface: Default::default(),
        grid: RasterGrid::Target,
        effects: LayerEffects::default(),
        items: vec![LayerItem::Raster(overlay_scene(overlay, engine))],
    });
    composition.layers[0].items.push(LayerItem::Layer {
        index,
        clips: Vec::new(),
    });
}

/// The scene of the overlay: box fills, fills, outlines and labels.
fn overlay_scene(overlay: &DevtoolsOverlay, engine: &mut Engine) -> RenderScene {
    let mut commands = Vec::new();
    for overlay_box in &overlay.boxes {
        push_box(&mut commands, overlay_box);
    }
    for fill in &overlay.fills {
        commands.push(RenderCmd::Rect {
            rect: fill.rect,
            fill: Paint::Solid(fill.color),
        });
    }
    for outline in &overlay.outlines {
        commands.push(RenderCmd::StrokeRect {
            rect: outline.rect,
            width: outline.width,
            color: outline.color,
            radius: 0.0,
        });
    }
    for label in &overlay.labels {
        push_label(&mut commands, label.at, &label.text, engine);
    }
    RenderScene { commands }
}

fn push_box(commands: &mut Vec<RenderCmd>, overlay_box: &OverlayBox) {
    let colors = [
        (overlay_box.margin, Rgba::from_hex(0xf6b26b, 0.4)),
        (overlay_box.border, Rgba::from_hex(0xffe599, 0.4)),
        (overlay_box.padding, Rgba::from_hex(0x93c47d, 0.4)),
        (overlay_box.content, Rgba::from_hex(0x6fa8dc, 0.4)),
    ];
    for (rect, color) in colors {
        commands.push(RenderCmd::Rect {
            rect,
            fill: Paint::Solid(color),
        });
    }
}

/// A label: black behind white text, shaped by Arvane's engine.
fn push_label(commands: &mut Vec<RenderCmd>, at: Point, text: &str, engine: &mut Engine) {
    let white = Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let style = ComputedTextStyle {
        font_size: 12.0,
        color: white,
        ..Default::default()
    };
    let Ok(shaped) = engine.shape_glyphs(text, &style, None) else {
        return;
    };
    commands.push(RenderCmd::Rect {
        rect: Rect {
            x: at.x,
            y: at.y,
            width: shaped.size.width,
            height: shaped.size.height,
        },
        fill: Paint::Solid(Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }),
    });
    for run in &shaped.runs {
        commands.push(RenderCmd::Glyphs {
            run: run.clone(),
            origin: at,
            fill: Some(Paint::Solid(white)),
            stroke: None,
            order: Default::default(),
        });
    }
}

impl Host {
    /// Drop `event` for the DevTools when it is a start key or while `id`
    /// is picking; the DevTools event is queued instead.
    pub(super) fn intercept_devtools(&mut self, id: WindowId, event: &ShellEvent) -> bool {
        if let ShellEvent::Key(input) = event
            && matches!(input.phase, KeyPhase::Down { .. })
            && let Some(pick) = self.start_key(input)
        {
            self.app
                .borrow()
                .push_event(HostEvent::DevtoolsKey { window: id, pick });
            return true;
        }
        if self.devtools.pick != Some(id) {
            return false;
        }
        match event {
            ShellEvent::Pointer(input) => {
                let node = self.pick_node(id, input.position);
                let phase = match input.phase {
                    PointerPhase::Move => PickPhase::Hover,
                    PointerPhase::Down(press) if press.button == MouseButton::Primary => {
                        PickPhase::Select
                    }
                    _ => return false,
                };
                self.app.borrow().push_event(HostEvent::DevtoolsPick {
                    window: id,
                    node,
                    phase,
                });
                true
            }
            ShellEvent::Wheel(_) => true,
            ShellEvent::Key(input)
                if input.key == Key::Escape && matches!(input.phase, KeyPhase::Down { .. }) =>
            {
                self.app.borrow().push_event(HostEvent::DevtoolsPick {
                    window: id,
                    node: None,
                    phase: PickPhase::Cancel,
                });
                true
            }
            _ => false,
        }
    }

    /// Whether the press `input` is a start key; `Some(true)` for a pick key.
    fn start_key(&self, input: &KeyInput) -> Option<bool> {
        let pick = self
            .devtools
            .keys
            .pick
            .iter()
            .any(|chord| chord_matches(chord, input));
        if pick {
            return Some(true);
        }
        self.devtools
            .keys
            .open
            .iter()
            .any(|chord| chord_matches(chord, input))
            .then_some(false)
    }

    /// The node under `point` for pick mode, ignoring modals.
    fn pick_node(&self, id: WindowId, point: Point) -> Option<NodeId> {
        self.app
            .borrow()
            .model(id)
            .and_then(|model| model.hit_any(point))
    }

    /// Apply the DevTools changes of the turn to `DevtoolsHost`.
    pub(super) fn apply_devtools_changes(&mut self) {
        let changes = self.app.borrow_mut().take_devtools_changes();
        for change in changes {
            match change {
                DevtoolsChange::Overlay(window, overlay) => {
                    self.devtools.overlays.insert(window, overlay);
                    if let Some(model) = self.app.borrow_mut().model_mut(window) {
                        model.paint.request_redraw();
                    }
                }
                DevtoolsChange::Pick(pick) => self.devtools.pick = pick,
                DevtoolsChange::Keys(keys) => self.devtools.keys = keys,
            }
        }
    }

    /// Apply the emulation to every window after `DeferredRequest::Emulate`.
    pub(super) fn apply_emulation(
        &mut self,
        emulation: EngineEmulation,
        windows: &mut Windows<'_>,
    ) {
        self.devtools.emulation = emulation;
        let ids = self.window_ids();
        let mut sizes = Vec::new();
        for id in ids {
            if let Some(window) = windows.get(id) {
                sizes.push((id, window.size(), window.scale()));
            }
        }
        for (id, size, native_scale) in sizes {
            let viewport = emulated_viewport(size, native_scale, emulation.scale);
            let scale = effective_scale(native_scale, emulation.scale);
            if let Some(model) = self.app.borrow_mut().model_mut(id) {
                model.set_viewport(viewport);
                model.paint.set_scale(scale);
                model.update_env(|env, _| apply_emulation(emulation, env));
            }
        }
    }
}
