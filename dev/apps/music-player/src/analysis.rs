//! Role: Display data calculated from the exact PCM sent to the audio device.

use crate::audio::AudioClip;
use std::f32::consts::TAU;

pub(crate) const BANDS: usize = 64;
pub(crate) const MAX_HZ: f32 = 8_000.0;
const FFT_SIZE: usize = 2048;
#[derive(Clone, Copy, Debug, Default)]
struct Complex {
    re: f32,
    im: f32,
}

impl Complex {
    fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }
}

fn fft(values: &mut [Complex; FFT_SIZE]) {
    let mut j = 0;
    for i in 1..FFT_SIZE {
        let mut bit = FFT_SIZE >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            values.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= FFT_SIZE {
        let angle = -TAU / len as f32;
        let step = Complex {
            re: angle.cos(),
            im: angle.sin(),
        };
        for start in (0..FFT_SIZE).step_by(len) {
            let mut weight = Complex { re: 1.0, im: 0.0 };
            for offset in 0..len / 2 {
                let even = values[start + offset];
                let odd = values[start + offset + len / 2].mul(weight);
                values[start + offset] = Complex {
                    re: even.re + odd.re,
                    im: even.im + odd.im,
                };
                values[start + offset + len / 2] = Complex {
                    re: even.re - odd.re,
                    im: even.im - odd.im,
                };
                weight = weight.mul(step);
            }
        }
        len *= 2;
    }
}

/// Peak amplitude per equal-width frequency band from 0 to 8 kHz.
pub(crate) fn spectrum(clip: &AudioClip, center_frame: usize) -> [f32; BANDS] {
    let mut values = [Complex::default(); FFT_SIZE];
    let start = center_frame.saturating_sub(FFT_SIZE / 2);
    for (i, slot) in values.iter_mut().enumerate() {
        let window = 0.5 - 0.5 * (TAU * i as f32 / (FFT_SIZE - 1) as f32).cos();
        slot.re = clip.mono_sample(start + i) * window;
    }
    fft(&mut values);
    let mut bands = [0.0_f32; BANDS];
    let max_hz = MAX_HZ.min(clip.sample_rate as f32 / 2.0);
    for (bin, value) in values.iter().enumerate().take(FFT_SIZE / 2 + 1).skip(1) {
        let hz = bin as f32 * clip.sample_rate as f32 / FFT_SIZE as f32;
        if hz >= max_hz {
            break;
        }
        let band = ((hz / max_hz) * BANDS as f32) as usize;
        if band < BANDS {
            // Hann's coherent gain is ~0.5; 4/N returns a sine's peak amplitude.
            let amplitude = (value.re.hypot(value.im) * 4.0 / FFT_SIZE as f32).min(1.0);
            bands[band] = bands[band].max(amplitude);
        }
    }
    bands
}

/// Min/max of the full recording within each visible time column.
pub(crate) fn waveform(clip: &AudioClip, columns: usize) -> Vec<(f32, f32)> {
    let columns = columns.clamp(1, 1024);
    let frames = clip.frame_count();
    (0..columns)
        .map(|column| {
            let start = column * frames / columns;
            let end = ((column + 1) * frames / columns).max(start + 1).min(frames);
            let (low, high, _) =
                clip.waveform_index
                    .range(&clip.samples, clip.channels, start, end);
            (low, high)
        })
        .collect()
}
