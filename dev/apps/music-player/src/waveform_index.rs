//! Role: Precomputed waveform bounds over shared PCM samples.

const WAVE_BIN_FRAMES: usize = 64;

/// Min/max pyramid prepared before a clip reaches the UI thread. A display
/// column reads at most two partial 64-frame bins from PCM; the rest comes
/// from precomputed levels, so resizing does not scan the whole recording.
#[derive(Clone, Debug)]
pub struct WaveformIndex {
    levels: Vec<Vec<(f32, f32)>>,
}

impl WaveformIndex {
    pub fn build(
        samples: &[f32],
        channels: u16,
        cancelled: &impl Fn() -> bool,
    ) -> Result<Self, String> {
        let channels = usize::from(channels);
        let frames = samples.len() / channels;
        let mut base = Vec::with_capacity(frames.div_ceil(WAVE_BIN_FRAMES));
        for bin_start in (0..frames).step_by(WAVE_BIN_FRAMES) {
            if cancelled() {
                return Err("WAV load cancelled".into());
            }
            let mut low = 1.0_f32;
            let mut high = -1.0_f32;
            for frame in bin_start..(bin_start + WAVE_BIN_FRAMES).min(frames) {
                let start = frame * channels;
                let value = samples[start..start + channels].iter().sum::<f32>() / channels as f32;
                low = low.min(value);
                high = high.max(value);
            }
            base.push((low, high));
        }
        let mut levels = vec![base];
        while levels.last().is_some_and(|level| level.len() > 1) {
            if cancelled() {
                return Err("WAV load cancelled".into());
            }
            let previous = levels
                .last()
                .expect("invariant: the loop condition saw a last level");
            let next = previous
                .chunks(2)
                .map(|pair| {
                    if pair.len() == 2 {
                        (pair[0].0.min(pair[1].0), pair[0].1.max(pair[1].1))
                    } else {
                        pair[0]
                    }
                })
                .collect();
            levels.push(next);
        }
        Ok(Self { levels })
    }

    pub(crate) fn range(
        &self,
        samples: &[f32],
        channels: u16,
        start: usize,
        end: usize,
    ) -> (f32, f32, usize) {
        let frames = samples.len() / usize::from(channels);
        let start = start.min(frames);
        let end = end.min(frames);
        if start >= end {
            return (0.0, 0.0, 0);
        }
        let mut low = 1.0_f32;
        let mut high = -1.0_f32;
        let mut scanned = 0;
        let first_full = start
            .div_ceil(WAVE_BIN_FRAMES)
            .saturating_mul(WAVE_BIN_FRAMES)
            .min(end);
        scan_raw(
            samples,
            channels,
            start,
            first_full,
            &mut low,
            &mut high,
            &mut scanned,
        );
        let last_full = end / WAVE_BIN_FRAMES * WAVE_BIN_FRAMES;
        if first_full < last_full {
            let mut bin = first_full / WAVE_BIN_FRAMES;
            let end_bin = last_full / WAVE_BIN_FRAMES;
            while bin < end_bin {
                let remaining = end_bin - bin;
                let alignment = if bin == 0 {
                    usize::BITS
                } else {
                    bin.trailing_zeros()
                };
                let span = usize::BITS - 1 - remaining.leading_zeros();
                let level = alignment.min(span) as usize;
                let bounds = self.levels[level][bin >> level];
                low = low.min(bounds.0);
                high = high.max(bounds.1);
                bin += 1 << level;
            }
            scan_raw(
                samples,
                channels,
                last_full,
                end,
                &mut low,
                &mut high,
                &mut scanned,
            );
        } else if first_full < end {
            scan_raw(
                samples,
                channels,
                first_full,
                end,
                &mut low,
                &mut high,
                &mut scanned,
            );
        }
        (low, high, scanned)
    }
}

fn scan_raw(
    samples: &[f32],
    channels: u16,
    from: usize,
    to: usize,
    low: &mut f32,
    high: &mut f32,
    scanned: &mut usize,
) {
    for frame in from..to {
        let value = mono_sample(samples, channels, frame);
        *low = low.min(value);
        *high = high.max(value);
        *scanned += 1;
    }
}

fn mono_sample(samples: &[f32], channels: u16, frame: usize) -> f32 {
    let start = frame.saturating_mul(usize::from(channels));
    if start + usize::from(channels) > samples.len() {
        return 0.0;
    }
    samples[start..start + usize::from(channels)]
        .iter()
        .copied()
        .sum::<f32>()
        / f32::from(channels)
}
