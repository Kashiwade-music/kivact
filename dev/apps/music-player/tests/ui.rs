//! Role: Music player through `kivact::test`.
//! Playback needs an audio device, so
//! the headless tests cover loading, seeking and drawing; the playback frame
//! loop is checked by an ignored test and in the real window.
#![warn(redundant_imports)]

use kivact::draw::Point;
use kivact::prelude::*;
use kivact::test::{Screen, render};
use kivact::view::A11yAction;
use kivact_music_player::player;
use std::time::Duration;

fn open() -> Screen {
    let screen = render(player);
    screen.find_by_text("Demo melody (generated PCM)");
    screen
}

fn position(screen: &Screen) -> String {
    screen.get_by_test_id("position").text()
}

/// Local x of `seconds` on the 16 s demo waveform.
fn x_for(screen: &Screen, seconds: f32) -> f32 {
    let width = screen
        .get_by_test_id("waveform")
        .rect()
        .expect("the waveform is laid out")
        .width;
    45.0 + (width - 57.0) * seconds / 16.0
}

fn wav(data: &[u8]) -> Vec<u8> {
    let mut bytes = b"RIFF".to_vec();
    bytes.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&8_000_u32.to_le_bytes());
    bytes.extend_from_slice(&16_000_u32.to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(data);
    bytes
}

#[test]
fn demo_loads_on_a_worker_and_draws() {
    let screen = open();
    assert!(
        screen
            .get_by_test_id("status")
            .text()
            .starts_with("Loaded Demo melody")
    );
    assert_eq!(position(&screen), "0:00 / 0:16  ·  Paused");
    assert!(
        screen.get_by_test_id("waveform").scene_len() > 100,
        "waveform columns"
    );
    assert!(
        screen.get_by_test_id("spectrum").scene_len() >= 64,
        "64 bands"
    );
    assert_eq!(screen.get_by_role(Role::Button, "Play").name(), "Play");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn transport_buttons_seek() {
    let screen = open();
    screen
        .get_by_role(Role::Button, "Forward 5 seconds")
        .click();
    assert_eq!(position(&screen), "0:05 / 0:16  ·  Paused");
    screen
        .get_by_role(Role::Button, "Forward 5 seconds")
        .click();
    assert_eq!(position(&screen), "0:10 / 0:16  ·  Paused");
    assert_eq!(screen.get_by_test_id("status").text(), "Paused");
    screen.get_by_role(Role::Button, "Back 5 seconds").click();
    assert_eq!(position(&screen), "0:05 / 0:16  ·  Paused");
    screen.get_by_role(Role::Button, "Stop").click();
    assert_eq!(position(&screen), "0:00 / 0:16  ·  Paused");
}

#[test]
fn waveform_click_drag_and_keys_seek() {
    let screen = open();
    let wave = screen.get_by_test_id("waveform");
    let y = 60.0;
    let (four, eleven) = (x_for(&screen, 4.4), x_for(&screen, 11.4));
    wave.drag(Point::new(four, y), Point::new(eleven, y));
    assert_eq!(position(&screen), "0:11 / 0:16  ·  Paused");
    assert_eq!(wave.value().as_deref(), Some("0:11"), "slider value");
    // Far outside on the left clamps to the start
    wave.drag(Point::new(four, y), Point::new(-500.0, y));
    assert_eq!(position(&screen), "0:00 / 0:16  ·  Paused");
    // Keyboard and assistive actions
    wave.press_key(Key::ArrowRight);
    assert_eq!(position(&screen), "0:05 / 0:16  ·  Paused");
    wave.a11y(A11yAction::Increment);
    assert_eq!(position(&screen), "0:10 / 0:16  ·  Paused");
    wave.press_key(Key::ArrowLeft);
    assert_eq!(position(&screen), "0:05 / 0:16  ·  Paused");
    wave.click();
    let expected = {
        let width = wave.rect().unwrap().width;
        let fraction = ((width / 2.0 - 45.0) / (width - 57.0)).clamp(0.0, 1.0);
        kivact_music_player::format_time(Duration::from_secs(16).mul_f32(fraction))
    };
    assert_eq!(position(&screen), format!("{expected} / 0:16  ·  Paused"));
}

#[test]
fn invalid_wav_shows_a_conditional_error() {
    let screen = open();
    screen.get_by_role(Role::Button, "Load WAV").click();
    let error = screen.find_by_test_id("player-error");
    assert!(
        error.text().starts_with("Cannot read WAV"),
        "{}",
        error.text()
    );
    assert_eq!(error.role(), Some(Role::Alert));
    screen.get_by_role(Role::Button, "Use demo").click();
    screen.find_by_text("Demo melody (generated PCM)");
    assert!(screen.query_by_test_id("player-error").is_none());
}

#[test]
fn local_wav_path_loads_and_redraws() {
    let samples: Vec<u8> = (0..16_000_i32)
        .flat_map(|i| (((i % 80) - 40) as i16 * 600).to_le_bytes())
        .collect();
    let path = std::env::temp_dir().join(format!("kivact-music-{}.wav", std::process::id()));
    std::fs::write(&path, wav(&samples)).unwrap();
    let screen = open();
    let file = path.file_name().unwrap().to_string_lossy().into_owned();
    screen
        .get_by_role(Role::TextBox, "WAV file path")
        .type_text(&path.to_string_lossy());
    screen.get_by_role(Role::Button, "Load WAV").click();
    screen.find_by_text(&file);
    assert_eq!(position(&screen), "0:00 / 0:02  ·  Paused");
    assert_eq!(
        screen.get_by_test_id("status").text(),
        format!("Loaded {file} · 8000 Hz · 1 channel(s)")
    );
    assert!(screen.query_by_test_id("player-error").is_none());
    // The album art switched to the file's embedded SVG without a failure.
    screen.get_by_role(Role::Image, "Album art");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    std::fs::remove_file(path).unwrap();
}

#[test]
fn album_art_is_a_named_square_image_beside_the_title() {
    let screen = open();
    let art = screen.get_by_role(Role::Image, "Album art");
    let r = art.rect().expect("laid out");
    assert_eq!((r.width, r.height), (56.0, 56.0), "cropped to the square");
    let title = screen
        .get_by_test_id("track-title")
        .rect()
        .expect("laid out");
    assert!(title.x >= r.x + r.width, "the title is right of the art");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn resize_redraws_the_canvas() {
    let screen = open();
    let wave = screen.get_by_test_id("waveform");
    let before = wave.rect().unwrap().width;
    let commands = wave.scene_len();
    screen.resize(500.0, 700.0);
    let after = wave.rect().unwrap().width;
    assert!(after < before, "{after} < {before}");
    assert!(
        wave.scene_len() < commands,
        "fewer waveform columns after shrinking"
    );
}

#[test]
#[ignore = "requires a working default audio playback endpoint"]
fn playback_runs_the_frame_loop_until_pause() {
    let screen = open();
    screen.get_by_role(Role::Button, "Play").click();
    assert_eq!(screen.get_by_test_id("status").text(), "Playing");
    let ticks = screen.frame_ticks();
    std::thread::sleep(Duration::from_millis(1200));
    screen.advance(Duration::from_millis(1200));
    assert!(
        screen.frame_ticks() > ticks,
        "the frame loop runs while playing"
    );
    assert!(
        position(&screen).starts_with("0:01"),
        "{}",
        position(&screen)
    );
    screen.get_by_role(Role::Button, "Pause").click();
    assert_eq!(screen.get_by_test_id("status").text(), "Paused");
    let ticks = screen.frame_ticks();
    screen.advance(Duration::from_millis(500));
    // At most the frame already requested before the pause is delivered.
    assert!(screen.frame_ticks() <= ticks + 1, "no frames while paused");
}
