//! Role: Application CPU image examples, independent of the UI kit renderer.
use kivact::draw::Pixels;
use kivact::prelude::{Signal, signal};
use kivact::style::Color;
use std::sync::Arc;

pub(super) fn gradient(width: u32, height: u32, mode: usize, grid: bool) -> Pixels {
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        for x in 0..width {
            let u = x as f64 / width.max(1) as f64;
            let v = y as f64 / height.max(1) as f64;
            let mut color = match mode {
                0 => [u * 0.4 + 0.1, v * 0.4 + 0.1, 0.3],
                2 => [0.2 + u * 0.6, 0.2 + v * 0.4, 0.6 - u * 0.3],
                _ => [0.1 + u * 0.4, 0.2 + v * 0.5, 0.6 - u * 0.3],
            };
            if grid && (x % 24 == 0 || y % 24 == 0) {
                color = [0.6, 0.6, 0.7];
            }
            rgba.extend(color.map(|c| (c * 255.0) as u8));
            rgba.push(255);
        }
    }
    Pixels::new(width, height, Arc::from(rgba))
}
/// A bounded channel transports owned pixels; the UI task belongs to this view.
/// Closing the view drops the receiver and causes any later send to fail.
pub(super) fn worker_gradient() -> Signal<Option<Pixels>> {
    let image = signal(None);
    let (tx, rx) = async_channel::bounded(2);
    let scale = kivact::platform::scale_factor();
    let width = (320.0 * scale).ceil().max(1.0) as u32;
    let height = (180.0 * scale).ceil().max(1.0) as u32;
    std::thread::spawn(move || {
        let _ = tx.send_blocking(gradient(width, height, 2, false));
    });
    kivact::task::spawn(async move {
        while let Ok(mut newest) = rx.recv().await {
            for _ in 0..rx.len() {
                if let Ok(frame) = rx.try_recv() {
                    newest = frame;
                }
            }
            image.set(Some(newest));
        }
    });
    image
}
pub(super) fn histogram(samples: &[Color], mode: &str) -> Pixels {
    let width = 256_u32;
    let height = 160_u32;
    let mut bins = [[0_u32; 256]; 3];
    for sample in samples {
        let channels = sample.components();
        for channel in 0..3 {
            bins[channel][(channels[channel].clamp(0.0, 1.0) * 255.0) as usize] += 1;
        }
    }
    let peak = bins.iter().flatten().copied().max().unwrap_or(1).max(1) as f64;
    let mut rgba = [16, 18, 23, 255].repeat(width as usize * height as usize);
    for x in 0..256 {
        for (channel, bins) in bins.iter().enumerate() {
            let h = (bins[x] as f64 / peak * height as f64) as usize;
            for y in 0..h {
                let p = ((height as usize - 1 - y) * width as usize + x) * 4;
                rgba[p + channel] = if mode == "waveform" { 190 } else { 235 };
            }
        }
    }
    Pixels::new(width, height, Arc::from(rgba))
}
