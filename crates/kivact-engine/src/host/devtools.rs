//! Role: Host-side DevTools state: overlays, pick, keys and emulation.
use crate::contract::{DevtoolsKeys, DevtoolsOverlay, EngineEmulation, KeyChord, OverlayBox};
use arvane::Engine;
use kivact_types::geometry3d::Matrix4;
use kivact_types::render::{
    CompositeLayer, Composition, LayerEffects, LayerItem, LayerKey, Paint, Plane, RasterGrid,
    RenderCmd, RenderScene,
};
use kivact_types::{
    ComputedTextStyle, Environment, Key, KeyCode, KeyInput, KeyPhase, Point, PointerClass, Rect,
    Rgba, Size, WindowId,
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
        Key::Unidentified => physical_letter(input.code)
            .is_some_and(|typed| typed.to_lowercase().eq(wanted.to_lowercase())),
        _ => false,
    }
}

/// The letter of a physical `KeyA` .. `KeyZ` position.
pub(super) fn physical_letter(code: KeyCode) -> Option<char> {
    match code {
        KeyCode::KeyA => Some('a'),
        KeyCode::KeyB => Some('b'),
        KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'),
        KeyCode::KeyE => Some('e'),
        KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'),
        KeyCode::KeyH => Some('h'),
        KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'),
        KeyCode::KeyK => Some('k'),
        KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'),
        KeyCode::KeyN => Some('n'),
        KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'),
        KeyCode::KeyQ => Some('q'),
        KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'),
        KeyCode::KeyT => Some('t'),
        KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'),
        KeyCode::KeyW => Some('w'),
        KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'),
        KeyCode::KeyZ => Some('z'),
        _ => None,
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
