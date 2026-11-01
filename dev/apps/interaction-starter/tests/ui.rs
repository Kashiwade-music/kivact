//! Role: The complete authoring route through public real-window operations.
#![warn(redundant_imports)]

use kivact::prelude::*;
use kivact::test::{RenderOptions, WindowMode, render_with};
use kivact::view::MouseButton;
use std::path::PathBuf;
use std::time::Duration;

fn render(path: PathBuf) -> kivact::test::Screen {
    let gpu = std::env::var("KIVACT_TEST_GPU").is_ok_and(|value| value == "1");
    render_with(
        RenderOptions {
            gpu,
            window: WindowMode::Real,
            ..Default::default()
        },
        move || kivact_interaction_starter::app(path),
    )
}

#[test]
fn search_detail_native_submit_retry_and_real_file_save() {
    let output = std::env::temp_dir().join(format!("kivact-starter-{}.txt", std::process::id()));
    let screen = render(output.clone());
    screen.find_by_text("Rows: 3");
    screen
        .get_by_role(Role::TextBox, "Search devices")
        .type_text("Pump");
    screen.find_by_text("Rows: 1");
    screen.get_by_role(Role::Button, "Pump").click();
    assert_eq!(screen.get_by_test_id("selected-key").text(), "device:pump");
    screen.get_by_role(Role::Button, "Save note").click();
    screen.find_by_text("Save failed: Write a note before saving.");
    let note = screen.get_by_role(Role::TextBox, "Device note");
    note.type_text("Checked today");
    note.press_key(Key::Enter);
    screen.find_by_text("Saved");
    assert_eq!(
        std::fs::read_to_string(&output).expect("saved file"),
        "device:pump\n50\nChecked today"
    );
    screen
        .get_by_role(Role::Button, "Simulate load error")
        .click();
    screen.find_by_text("Load failed. Retry.");
    screen.get_by_role(Role::Button, "Retry load").click();
    screen.find_by_text("Rows: 1");
    assert_eq!(
        screen
            .get_by_role(Role::TextBox, "Device note")
            .value()
            .as_deref(),
        Some("Checked today")
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    std::fs::remove_file(output).expect("remove own test output");
}

#[test]
fn cancellation_drops_pending_load_and_retry_loads_again() {
    let screen = render(std::env::temp_dir().join("kivact-unused-starter-save.txt"));
    screen.get_by_role(Role::Button, "Cancel load").click();
    screen.advance(Duration::from_millis(500));
    screen.get_by_text("Load cancelled");
    screen.get_by_role(Role::Button, "Retry load").click();
    screen.find_by_text("Rows: 3");
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
}

#[test]
fn calibration_drag_survives_secondary_release_and_saves_the_result() {
    let output =
        std::env::temp_dir().join(format!("kivact-starter-drag-{}.txt", std::process::id()));
    let screen = render(output.clone());
    screen.find_by_text("Rows: 3");
    screen.get_by_role(Role::Button, "Pump").click();
    let slider = screen.get_by_role(Role::Slider, "Calibration");
    let rect = slider.rect().expect("calibration geometry");
    let y = rect.y + rect.height / 2.0;
    let x = |fraction| rect.x + 8.0 + (rect.width - 16.0) * fraction;
    screen.mouse_move(x(0.2), y);
    screen.mouse_down(MouseButton::Primary);
    assert_eq!(slider.value().as_deref(), Some("20"));
    screen.mouse_move(x(0.4), y);
    assert_eq!(slider.value().as_deref(), Some("40"));
    screen.mouse_down(MouseButton::Secondary);
    screen.mouse_up(MouseButton::Secondary);
    screen.mouse_move(x(0.8), y);
    assert_eq!(
        slider.value().as_deref(),
        Some("80"),
        "secondary release preserves the primary drag"
    );
    screen.mouse_up(MouseButton::Primary);
    screen.mouse_move(x(0.1), y);
    assert_eq!(
        slider.value().as_deref(),
        Some("80"),
        "primary release ends the drag"
    );
    screen
        .get_by_role(Role::TextBox, "Device note")
        .type_text("Calibrated");
    screen.get_by_role(Role::Button, "Save note").click();
    screen.find_by_text("Saved");
    assert_eq!(
        std::fs::read_to_string(&output).expect("saved calibration"),
        "device:pump\n80\nCalibrated"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    std::fs::remove_file(output).expect("remove own test output");
}

#[test]
fn failed_file_save_retries_the_same_selected_snapshot() {
    let folder = std::env::temp_dir().join(format!("kivact-starter-retry-{}", std::process::id()));
    let output = folder.join("note.txt");
    let screen = render(output.clone());
    screen.find_by_text("Rows: 3");
    screen.get_by_role(Role::Button, "Camera").click();
    screen
        .get_by_role(Role::TextBox, "Device note")
        .type_text("Original request");
    screen.get_by_role(Role::Button, "Save note").click();
    screen.wait_for(Duration::from_secs(5), |screen| {
        screen
            .get_by_test_id("save-status")
            .text()
            .starts_with("Save failed:")
    });
    assert!(
        screen
            .get_by_test_id("save-status")
            .text()
            .starts_with("Save failed:")
    );
    std::fs::create_dir(&folder).expect("create own missing output folder");
    screen
        .get_by_role(Role::TextBox, "Device note")
        .type_text(" new draft");
    screen.get_by_role(Role::Button, "Retry save").click();
    screen.find_by_text("Saved");
    assert_eq!(
        std::fs::read_to_string(&output).expect("retried file"),
        "device:camera\n50\nOriginal request"
    );
    assert!(screen.errors().is_empty(), "{:?}", screen.errors());
    std::fs::remove_file(output).expect("remove own output file");
    std::fs::remove_dir(folder).expect("remove own empty test folder");
}
