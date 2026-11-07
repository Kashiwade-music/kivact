// Source: resvg 0.46.0, src/filter/displacement_map.rs (Copyright 2020 the Resvg Authors).
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Changes: the scale is already in pixels and the channels are kivact's Channel.

//! Role: feDisplacementMap on the CPU (each pixel takes the source pixel its map moves it to).

use kivact_types::effect::Channel;

use crate::cpu::surface::Pixmap;

/// The displaced `src`, moved by the `map` channels scaled by `scale_x` and `scale_y` px.
pub(crate) fn apply(
    src: &Pixmap,
    map: &Pixmap,
    scale: (f32, f32),
    channels: (Channel, Channel),
) -> Pixmap {
    let (w, h) = src.size();
    let mut out = Pixmap::new(w, h);
    let (wi, hi) = (i64::from(w), i64::from(h));
    for y in 0..h {
        for x in 0..w {
            let i = (y as usize * w as usize + x as usize) * 4;
            let m = &map.data[i..i + 4];
            let offset = |channel: Channel| {
                let c = match channel {
                    Channel::R => m[0],
                    Channel::G => m[1],
                    Channel::B => m[2],
                    Channel::A => m[3],
                };
                f32::from(c) / 255.0 - 0.5
            };
            let dx = offset(channels.0);
            let dy = offset(channels.1);
            let ox = (x as f32 + dx * scale.0).round() as i64;
            let oy = (y as f32 + dy * scale.1).round() as i64;
            if (0..wi).contains(&ox) && (0..hi).contains(&oy) {
                let s = src.get(ox as u32, oy as u32);
                out.set(x, y, s);
            }
        }
    }
    out
}
