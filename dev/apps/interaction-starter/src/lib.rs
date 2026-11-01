//! Role: A short, public-API path from search to a saved device note.
//!
//! The loader simulates a delayed service; the saver writes an actual file
//! on the blocking pool. A failed request is displayed as its `AsyncState`,
//! rather than assumed to be an error-boundary fallback.
#![warn(redundant_imports)]

use kivact::draw::{Point, Rect};
use kivact::interaction::{DragCoordinates, DragUpdate, use_drag, use_scalar};
use kivact::prelude::*;
use kivact::ui::ScalarRange;
use kivact::view::Handled;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, PartialEq)]
struct Device {
    key: String,
    label: String,
}

fn devices() -> Vec<Device> {
    [
        ("device:camera", "Camera"),
        ("device:pump", "Pump"),
        ("device:router", "Router"),
    ]
    .into_iter()
    .map(|(key, label)| Device {
        key: key.into(),
        label: label.into(),
    })
    .collect()
}

/// Build the complete screen. The caller chooses the output file, so tests
/// and independently packaged applications do not write user settings.
pub fn app(output: PathBuf) -> impl IntoView {
    let query = signal(String::new());
    let fail_load = signal(false);
    let selected = signal(None::<String>);
    let note = signal(String::new());
    let level = signal(50.0);
    let rows = resource(
        move || query.get(),
        move |query: String| {
            let fail = fail_load.get();
            async move {
                kivact::task::sleep(Duration::from_millis(120)).await;
                if fail {
                    return Err(UiError::msg("Load failed. Retry."));
                }
                let query = query.to_lowercase();
                Ok(devices()
                    .into_iter()
                    .filter(|row| row.label.to_lowercase().contains(&query))
                    .collect::<Vec<_>>())
            }
        },
    );
    let save = action(move |(key, note, level): (String, String, f64)| {
        let output = output.clone();
        async move {
            if note.trim().is_empty() {
                return Err(UiError::msg("Write a note before saving."));
            }
            let content = format!("{key}\n{level}\n{note}");
            // UI handles stay on the UI thread. Only owned data crosses into
            // the blocking closure; its I/O failure becomes the action's state.
            blocking(move || std::fs::write(output, content)).await?;
            Ok(())
        }
    });

    column((
        text("Device notes").style(style().font_size(24).weight(700)),
        text_input("Search devices", query).test_id("search"),
        row((
            button("Simulate load error")
                .variant(ButtonVariant::Secondary)
                .on_press(move || {
                    fail_load.set(true);
                    rows.refetch();
                }),
            button("Retry load")
                .variant(ButtonVariant::Secondary)
                .on_press(move || {
                    fail_load.set(false);
                    rows.refetch();
                }),
            button("Cancel load")
                .variant(ButtonVariant::Secondary)
                .on_press(move || rows.cancel()),
        ))
        .style(style().gap(8)),
        text(move || match rows.state() {
            AsyncState::Pending => "Loading".to_owned(),
            AsyncState::Failed(error) if error.is_cancelled() => "Load cancelled".to_owned(),
            AsyncState::Ready(rows) => format!("Rows: {}", rows.len()),
            AsyncState::Failed(_) => "Load error".to_owned(),
            AsyncState::Idle => "Idle".to_owned(),
        })
        .test_id("load-status"),
        show_some(
            move || rows.error().filter(|error| !error.is_cancelled()),
            |error| text(move || error.get().to_string()).role(Role::Alert),
        ),
        row((
            column(for_each(
                move || rows.latest().unwrap_or_default(),
                |row| row.key.clone(),
                move |_, row| {
                    button(memo(move || row.with(|row| row.label.clone())))
                        .variant(ButtonVariant::Secondary)
                        .on_press(move || {
                            selected.set(Some(row.get().key));
                            note.set(String::new());
                            level.set(50.0);
                        })
                },
            ))
            .style(style().width(160).gap(8)),
            show(
                move || selected.get().is_some(),
                move || {
                    let calibration = use_scalar(
                        level,
                        ScalarRange::new(0.0, 100.0)
                            .and_then(|range| range.with_step(1.0))
                            .expect("invariant: finite ordered calibration bounds and step"),
                    );
                    let calibration_control = calibration.thumb();
                    let calibration_node = calibration_control.node_ref();
                    let follow = move |update: DragUpdate<()>| {
                        if let Some(rect) = calibration_node.rect().get_untracked() {
                            calibration_node.focus();
                            let ratio = ((update.pos.x - 8.0) / (rect.width - 16.0).max(1.0))
                                .clamp(0.0, 1.0);
                            calibration.set(100.0 * f64::from(ratio));
                        }
                    };
                    let calibration_drag = use_drag(DragCoordinates::Local)
                        .on_start(follow)
                        .on_update(follow);
                    column((
                        text(move || selected.get().unwrap_or_default()).test_id("selected-key"),
                        text_input("Device note", note)
                            .on_submit(move || {
                                save.run((
                                    selected.get().unwrap_or_default(),
                                    note.get(),
                                    level.get(),
                                ))
                            })
                            // Custom key observation continues into native submit.
                            .on_key(|_| Handled::No)
                            .test_id("note"),
                        node()
                            .part(calibration_control)
                            .part(calibration_drag.source(()))
                            .name("Calibration")
                            .draw(move |draw, size| {
                                let width = (size.width - 16.0).max(0.0);
                                let progress = (level.get() / 100.0) as f32;
                                draw.rounded_rect(
                                    Rect {
                                        x: 8.0,
                                        y: 13.0,
                                        width,
                                        height: 6.0,
                                    },
                                    3.0,
                                    ColorToken::Border,
                                );
                                draw.rounded_rect(
                                    Rect {
                                        x: 8.0,
                                        y: 13.0,
                                        width: width * progress,
                                        height: 6.0,
                                    },
                                    3.0,
                                    ColorToken::Accent,
                                );
                                draw.circle(
                                    Point::new(8.0 + width * progress, 16.0),
                                    7.0,
                                    ColorToken::Accent,
                                );
                            })
                            .style(style().width(240).height(32)),
                        button("Save note")
                            .disabled(move || save.state().is_pending())
                            .on_press(move || {
                                save.run((
                                    selected.get().unwrap_or_default(),
                                    note.get(),
                                    level.get(),
                                ))
                            }),
                        button("Retry save")
                            .variant(ButtonVariant::Secondary)
                            .on_press(move || save.retry()),
                        text(move || match save.state() {
                            AsyncState::Idle => "Not saved".to_owned(),
                            AsyncState::Pending => "Saving".to_owned(),
                            AsyncState::Ready(()) => "Saved".to_owned(),
                            AsyncState::Failed(error) if error.is_cancelled() => {
                                "Save cancelled".to_owned()
                            }
                            AsyncState::Failed(error) => format!("Save failed: {error}"),
                        })
                        .test_id("save-status"),
                    ))
                    .style(style().width(360).gap(8))
                },
            ),
        ))
        .style(style().gap(24)),
    ))
    .style(style().padding(24).gap(12))
}
