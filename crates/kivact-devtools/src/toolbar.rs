//! Role: The toolbar of the DevTools app: target, window, pick, record and review.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::{Panel, Session};

/// The target, the inspected window, the pick and record buttons, the review count and pop out.
pub(crate) fn view(session: Session) -> impl IntoView {
    row((
        text(move || session.target.with(|target| target.app.name.clone())),
        select(
            "Window",
            move || {
                session.windows.with(|windows| {
                    windows
                        .iter()
                        .map(|window| ListItem::new(window.id, window.title.clone()))
                        .collect::<Vec<_>>()
                })
            },
            binding(
                move || Some(session.window()),
                move |window: Option<p::WindowId>| {
                    if let Some(window) = window {
                        session.load_window(window, None);
                    }
                },
            ),
        ),
        button("Pick").on_press(move || {
            session.send::<p::StartPick>(
                p::StartPickParams {
                    window: session.window(),
                },
                |_| (),
            );
        }),
        button(move || if recording(session) { "Stop" } else { "Record" }).on_press(move || {
            if recording(session) {
                session.send::<p::StopRecording>(p::Empty {}, move |_| {
                    session.recording.refetch();
                });
            } else {
                session.send::<p::StartRecording>(
                    p::RecordingSettings {
                        updates: true,
                        frames: true,
                        input: false,
                    },
                    move |_| {
                        session.recording.refetch();
                    },
                );
            }
        }),
        button(move || {
            let count = session
                .review
                .latest()
                .map(|review| review.items.len())
                .unwrap_or(0);
            format!("Review ({count})")
        })
        .on_press(move || session.panel.set(Panel::Review)),
        spacer(),
        button("Pop out").on_press(move || {
            let panel = session.panel.get_untracked();
            session.popped.update(|popped| {
                if !popped.contains(&panel) {
                    popped.push(panel);
                }
            });
        }),
    ))
    .style(
        style()
            .gap(8)
            .padding_x(8)
            .padding_y(6)
            .align_items(Align::Center),
    )
}

/// Whether the target is recording turns, frames or input.
fn recording(session: Session) -> bool {
    session
        .recording
        .latest()
        .is_some_and(|settings| settings.updates || settings.frames || settings.input)
}
