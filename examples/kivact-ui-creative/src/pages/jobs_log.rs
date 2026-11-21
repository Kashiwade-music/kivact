//! Role: The page of the Creative catalog.

use catalog_shell::{Example, Page};
use super::common::{TextRow, bind, col, line, note, spec, specs, text_table};
use kivact::draw::{LineCap, LineJoin, Stroke};
use kivact::geometry::Point;
use kivact::geometry::Rect;
use kivact::interaction::use_toaster;
use kivact::motion::FrameRate;
use kivact::motion::frame_loop;
use kivact::prelude::{Element, IntoView, Signal, effect, node, row, signal, spacer, text};
use kivact::state::IntoValue;
use kivact::style::GridPlacement;
use kivact::style::style;
use kivact::view::Node;
use kivact_ui_creative as ui;
use super::jobs::*;
use super::common::*;

pub(crate) fn page() -> Page {
    Page::new("jobs-log", "Jobs & Log", "Feedback", include_str!("jobs_log.rs"))
        .description("Progress, notifications and log.")
        .example(Example::new("jobs-log", "Jobs & Log", "", demo))
}

// example: jobs-log
fn demo() -> impl IntoView {
    let status = signal(String::new());
    let job = kivact::state::machine(JobRun::Idle, || JobMachine, |_, ()| {});
    let running = kivact::state::memo(move || job.get() == JobRun::Running);
    let progress = signal(vec![1.0_f64, 0.0, 0.0, 0.0]);
    let rows = signal(queue_rows());
    let log=signal("04:00:49  ›  Accent: lime\n04:00:50  ›  Compact rows (20 px)\n04:01:02  ›  Accent: iris\n04:01:03  ›  Regular rows (24 px)\n04:01:22  ›  Accent: lime\n04:01:23  ›  Compact rows (20 px)\n04:01:25  ›  Accent: iris\n04:01:26  ›  Regular rows (24 px)".split('\n').map(|text|LogLine{level:0,text:text.to_owned()}).collect::<Vec<_>>());
    let severity = signal(vec![true; 4]);
    let amount = signal(0.0_f64);
    let clock = frame_loop(move |tick| {
        amount.update(|v| *v = (*v + tick.delta.as_secs_f64() * 0.12).min(1.0));
        progress.update(|p| {
            if let Some((_, v)) = p.iter_mut().enumerate().find(|(index, value)| {
                **value < 1.0
                    && rows
                        .with_untracked(|rows| rows.iter().any(|row| row.id == *index as u64 + 1))
            }) {
                *v = (*v + tick.delta.as_secs_f64() * 0.2).min(1.0);
            }
        });
        if rows.with_untracked(|rows| {
            rows.iter()
                .all(|row| progress.with_untracked(|p| p[row.id as usize - 1] >= 1.0))
        }) {
            job.send(JobEvent::Finished);
            use_toaster().push(
                kivact::interaction::Toast::new("Catalog report")
                    .tone(kivact::interaction::Tone::Info)
                    .body("UI operation completed."),
            );
            status.set("Render complete · four catalog jobs".into());
            log.update(|v| {
                v.push(LogLine {
                    level: 1,
                    text: "04:01:30  INFO  Render complete".into(),
                })
            });
        }
    });
    effect(move || {
        if running.get() {
            clock.start(FrameRate::Hz(24.0));
        } else {
            clock.stop();
        }
    });
    specs((
        queue_view(status, running, job, progress, rows),
        progress_view(running, amount),
        log_view(log, severity),
        report_view(status, log),
    ))
}
// end example
