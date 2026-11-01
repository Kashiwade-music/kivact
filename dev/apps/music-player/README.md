# Music player

The demo sends generated PCM to the default playback device through rodio. The same
samples drive the full-track waveform and the 64-band FFT spectrum. The spectrum
axes show frequency in hertz and normalized amplitude.

```powershell
cargo run -p kivact-music-player
```

Use **Play/Pause**, **Stop**, the ±5 second buttons, or click and drag the waveform
to seek. With the waveform focused, ←/→ seek by 5 seconds (it is a slider for
assistive technology too). Enter an absolute path to an uncompressed RIFF/WAVE file
and press **Load WAV**. The parser accepts PCM16 or IEEE float32, mono or stereo, up
to 128 MiB of sample data. **Use demo** restores the generated clip. A conditional
error message appears if loading or audio output fails.

## Structure (one component per file)

| File | Role |
|---|---|
| `components/app.rs` | Owns the player, the loader action and the frame loop; derives the status line and the error |
| `components/cover.rs` | Album art: `image(..).fit(ImageFit::Cover)` from an embedded SVG (`assets/`), one for the demo and one for a loaded file |
| `components/transport.rs` | Path field, Load WAV / Use demo, and the transport buttons with the position label |
| `components/waveform.rs` | Canvas with the waveform (`draw`) and the playback cursor (`draw_over`); pointer, keys and a11y actions seek |
| `components/spectrum.rs` | Canvas with the spectrum of the PCM around the playback position |
| `player.rs` | `PlayerCx`, the Copy handles shared through context |
| `audio.rs` / `analysis.rs` | WAV decoding, rodio playback, min/max waveform pyramid and FFT |

- **External object in a signal:** `AudioPlayer` (rodio stream and sink) is neither
  `Clone` nor `PartialEq`. It lives in `signal(..)` and changes with
  `player.update(..)`, which always notifies. Readers use memos that stop on equal
  values: `clip` (identity of the shared `Arc<AudioClip>`), `position` and `playing`.
  The waveform therefore redraws only when a new clip loads, while the cursor and the
  spectrum follow the position.
- **Playback position:** rodio advances on its own thread. While `playing` is true, an
  effect runs `frame_loop(|_| player.notify())` at 30 Hz; the natural end flips
  `playing` to false and stops the loop.
- **Loading:** an `action` runs `blocking` on the App's blocking
  pool; loading again drops the previous future and its `CancelToken` stops the
  decoder. The min/max pyramid is built on the worker before the clip reaches the UI.
  Playback reads the same PCM allocation.
- **Album art:** `image(move || Asset::from(if demo { DEMO_COVER } else { FILE_COVER }))`
  switches between two original 3:2 SVGs embedded with `include_bytes!`; `ImageFit::Cover`
  crops them to the 56×56 square, and `.name("Album art")` is the alternative text.
- **Seeking:** the pointer handler keeps the dragging pointer in `var(..)` (a `useRef`)
  and reads the canvas width from `NodeRef::rect()`.

## Tests

`tests/ui.rs` (`kivact::test`: real Arvane windows, virtual clock) covers demo loading and drawing, the album art, the seek
buttons, waveform drag/keys/slider actions, an invalid path error and recovery,
loading a local WAV, and redraw on resize. Playback needs an audio device and is
covered by an ignored test and by the real-window check. `src/audio.rs` and
`src/analysis.rs` unit-test WAV decoding, cancellation, the rodio source and the
FFT/waveform index.

```powershell
cargo test -p kivact-music-player
cargo test -p kivact-music-player -- --ignored   # with a playback device
```
