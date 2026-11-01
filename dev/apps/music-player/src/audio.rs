//! Role: PCM shared by playback and visual analysis.

use crate::waveform_index::WaveformIndex;
use rodio::{OutputStream, Sink, Source, source::SeekError};
use std::{f32::consts::TAU, fs, io::Read, path::Path, sync::Arc, time::Duration};

/// Title of the generated demo clip.
pub(crate) const DEMO_TITLE: &str = "Demo melody (generated PCM)";

#[derive(Clone, Debug)]
pub struct AudioClip {
    pub title: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub waveform_index: WaveformIndex,
}

impl AudioClip {
    pub fn empty() -> Self {
        Self {
            title: "Loading…".into(),
            sample_rate: 44_100,
            channels: 1,
            samples: Vec::new(),
            waveform_index: WaveformIndex::build(&[], 1, &|| false)
                .expect("invariant: an empty clip always builds an index"),
        }
    }

    pub fn demo() -> Self {
        let sample_rate = 44_100;
        let seconds = 16;
        let notes = [
            220.0_f32, 261.63, 329.63, 392.0, 440.0, 523.25, 659.25, 587.33,
        ];
        let frame_count = sample_rate as usize * seconds;
        let mut samples = Vec::with_capacity(frame_count);
        for i in 0..frame_count {
            let t = i as f32 / sample_rate as f32;
            let step = (t as usize / 2).min(notes.len() - 1);
            let phase = (t % 2.0) / 2.0;
            let envelope = (phase * 25.0).min(1.0) * ((1.0 - phase) * 5.0).min(1.0);
            let note = notes[step];
            let value = ((TAU * note * t).sin() * 0.26
                + (TAU * note * 2.0 * t).sin() * 0.075
                + (TAU * note * 3.0 * t).sin() * 0.025)
                * envelope;
            samples.push(value);
        }
        let waveform_index =
            WaveformIndex::build(&samples, 1, &|| false).expect("generated demo waveform is valid");
        Self {
            title: DEMO_TITLE.into(),
            sample_rate,
            channels: 1,
            samples,
            waveform_index,
        }
    }

    pub fn frame_count(&self) -> usize {
        self.samples.len() / usize::from(self.channels)
    }

    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.frame_count() as f64 / f64::from(self.sample_rate))
    }

    pub fn mono_sample(&self, frame: usize) -> f32 {
        let start = frame.saturating_mul(usize::from(self.channels));
        if start + usize::from(self.channels) > self.samples.len() {
            return 0.0;
        }
        self.samples[start..start + usize::from(self.channels)]
            .iter()
            .copied()
            .sum::<f32>()
            / f32::from(self.channels)
    }

    /// Check cancellation while reading and converting a bounded WAV file.
    pub fn from_wav(path: &Path, cancelled: &impl Fn() -> bool) -> Result<Self, String> {
        let file_bytes = fs::metadata(path)
            .map_err(|error| format!("Cannot read WAV: {error}"))?
            .len();
        if file_bytes > 128 * 1024 * 1024 + 4096 {
            return Err("WAV exceeds the tutorial file limit".into());
        }
        let mut file = fs::File::open(path).map_err(|error| format!("Cannot read WAV: {error}"))?;
        let mut bytes = Vec::with_capacity(file_bytes as usize);
        let mut chunk = [0_u8; 64 * 1024];
        loop {
            if cancelled() {
                return Err("WAV load cancelled".into());
            }
            let read = file
                .read(&mut chunk)
                .map_err(|error| format!("Cannot read WAV: {error}"))?;
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&chunk[..read]);
            if bytes.len() > 128 * 1024 * 1024 + 4096 {
                return Err("WAV exceeds the tutorial file limit".into());
            }
        }
        let mut clip = Self::decode_wav(&bytes, cancelled)?;
        clip.title = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        Ok(clip)
    }

    fn decode_wav(bytes: &[u8], cancelled: &impl Fn() -> bool) -> Result<Self, String> {
        if cancelled() {
            return Err("WAV load cancelled".into());
        }
        if bytes.len() < 12 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
            return Err("Expected a RIFF/WAVE file".into());
        }
        let mut format = None;
        let mut data = None;
        let mut at = 12usize;
        while at + 8 <= bytes.len() {
            if cancelled() {
                return Err("WAV load cancelled".into());
            }
            let len = le_u32(bytes, at + 4) as usize;
            let start = at + 8;
            let end = start.checked_add(len).ok_or("WAV chunk is too large")?;
            if end > bytes.len() {
                return Err("Truncated WAV chunk".into());
            }
            match &bytes[at..at + 4] {
                b"fmt " => format = Some(&bytes[start..end]),
                b"data" => data = Some(&bytes[start..end]),
                _ => {}
            }
            at = end.checked_add(len & 1).ok_or("WAV chunk is too large")?;
        }
        let fmt = format.ok_or("WAV has no fmt chunk")?;
        let raw = data.ok_or("WAV has no data chunk")?;
        if fmt.len() < 16 {
            return Err("Truncated WAV format".into());
        }
        let encoding = le_u16(fmt, 0);
        let channels = le_u16(fmt, 2);
        let sample_rate = le_u32(fmt, 4);
        let block_align = usize::from(le_u16(fmt, 12));
        let bits = le_u16(fmt, 14);
        if !(channels == 1 || channels == 2) || sample_rate == 0 || sample_rate > 192_000 {
            return Err("WAV must be mono/stereo with a supported sample rate".into());
        }
        let bytes_per_sample = match (encoding, bits) {
            (1, 16) => 2,
            (3, 32) => 4,
            _ => return Err("Supported WAV formats: PCM16 or float32".into()),
        };
        if block_align != usize::from(channels) * bytes_per_sample || raw.len() % block_align != 0 {
            return Err("Invalid WAV block alignment".into());
        }
        if raw.is_empty() {
            return Err("WAV contains no samples".into());
        }
        // The tutorial keeps the whole clip in memory for seek and analysis.
        if raw.len() > 128 * 1024 * 1024 {
            return Err("WAV exceeds the 128 MiB tutorial limit".into());
        }
        let samples = raw
            .chunks_exact(bytes_per_sample)
            .enumerate()
            .map(|(index, chunk)| -> Result<f32, String> {
                if index % 4096 == 0 && cancelled() {
                    return Err("WAV load cancelled".into());
                }
                match encoding {
                    1 => Ok(f32::from(i16::from_le_bytes([chunk[0], chunk[1]])) / 32768.0),
                    _ => {
                        let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        if !value.is_finite() {
                            return Err("WAV contains a non-finite sample".into());
                        }
                        Ok(value.clamp(-1.0, 1.0))
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let waveform_index = WaveformIndex::build(&samples, channels, cancelled)?;
        Ok(Self {
            title: "WAV".into(),
            sample_rate,
            channels,
            samples,
            waveform_index,
        })
    }
}

/// A decoded clip shared by playback, waveform and spectrum. Equality is
/// identity: a new load is a new clip, so memos stop on unchanged clips
/// without comparing PCM.
#[derive(Clone, Debug)]
pub(crate) struct SharedClip(pub Arc<AudioClip>);

impl PartialEq for SharedClip {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// `m:ss`.
pub fn format_time(duration: Duration) -> String {
    let total = duration.as_secs();
    format!("{}:{:02}", total / 60, total % 60)
}

/// Rodio reads samples from the same Arc used by waveform and spectrum.
/// Constructing this Source on the UI thread does not copy the PCM vector.
struct ArcClipSource {
    clip: Arc<AudioClip>,
    position: usize,
}

impl ArcClipSource {
    fn new(clip: Arc<AudioClip>) -> Self {
        Self { clip, position: 0 }
    }
}

impl Iterator for ArcClipSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = *self.clip.samples.get(self.position)?;
        self.position += 1;
        Some(sample)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.clip.samples.len().saturating_sub(self.position);
        (remaining, Some(remaining))
    }
}

impl Source for ArcClipSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.clip.samples.len().saturating_sub(self.position))
    }

    fn channels(&self) -> u16 {
        self.clip.channels
    }

    fn sample_rate(&self) -> u32 {
        self.clip.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.clip.duration())
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let frame = (pos.as_secs_f64() * f64::from(self.clip.sample_rate)) as usize;
        let samples = frame.saturating_mul(usize::from(self.clip.channels));
        self.position = samples.min(self.clip.samples.len());
        Ok(())
    }
}

pub(crate) struct AudioPlayer {
    pub clip: Arc<AudioClip>,
    stream: Option<OutputStream>,
    sink: Option<Sink>,
    parked: Duration,
    pub error: Option<String>,
}

impl AudioPlayer {
    pub(crate) fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.stream = None;
        self.parked = Duration::ZERO;
        self.error = None;
    }
    pub(crate) fn new(clip: AudioClip) -> Self {
        Self {
            clip: Arc::new(clip),
            stream: None,
            sink: None,
            parked: Duration::ZERO,
            error: None,
        }
    }

    fn ensure_sink(&mut self) -> Result<(), String> {
        if self.sink.is_some() {
            return Ok(());
        }
        let (stream, handle) = OutputStream::try_default()
            .map_err(|error| format!("Audio output unavailable: {error}"))?;
        let sink =
            Sink::try_new(&handle).map_err(|error| format!("Audio output failed: {error}"))?;
        sink.append(ArcClipSource::new(self.clip.clone()));
        sink.pause();
        self.stream = Some(stream);
        self.sink = Some(sink);
        Ok(())
    }

    /// Replace playback and visuals with one shared decoded clip. A worker can
    /// hand this Arc to the UI thread without cloning a large PCM buffer.
    pub(crate) fn load(&mut self, clip: Arc<AudioClip>) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.stream = None;
        self.clip = clip;
        self.parked = Duration::ZERO;
        self.error = None;
    }

    pub(crate) fn is_playing(&self) -> bool {
        self.sink
            .as_ref()
            .is_some_and(|sink| !sink.is_paused() && !sink.empty())
    }

    pub(crate) fn position(&self) -> Duration {
        self.sink
            .as_ref()
            .map_or(self.parked, Sink::get_pos)
            .min(self.clip.duration())
    }

    pub(crate) fn toggle(&mut self) {
        if self.is_playing() {
            if let Some(sink) = &self.sink {
                sink.pause();
            }
            self.parked = self.position();
            return;
        }
        if self.position() >= self.clip.duration() {
            self.sink = None;
            self.stream = None;
            self.parked = Duration::ZERO;
        }
        if let Err(error) = self.ensure_sink() {
            self.error = Some(error);
            return;
        }
        if let Some(sink) = &self.sink {
            if let Err(error) = sink.try_seek(self.parked) {
                self.error = Some(format!("Seek failed: {error}"));
                return;
            }
            sink.play();
        }
        self.error = None;
    }

    pub(crate) fn seek(&mut self, position: Duration) {
        let position = position.min(self.clip.duration());
        self.parked = position;
        if let Some(sink) = &self.sink
            && let Err(error) = sink.try_seek(position)
        {
            self.error = Some(format!("Seek failed: {error}"));
        }
    }

    pub(crate) fn seek_delta(&mut self, seconds: i64) {
        let current = self.position().as_secs_f64();
        self.seek(Duration::from_secs_f64((current + seconds as f64).max(0.0)));
    }
}

/// Little-endian `u16` at `at`. Callers check `bytes.len() >= at + 2` first.
fn le_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

/// Little-endian `u32` at `at`. Callers check `bytes.len() >= at + 4` first.
fn le_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}
