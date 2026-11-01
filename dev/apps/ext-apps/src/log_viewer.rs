//! Role: Log viewer: two log sources compared side by side.

mod data;
mod pane;

pub use data::{BUCKET_MINUTES, Level, LogLine, MINUTES, clock, generate};
pub use pane::log_pane;

use kivact::style::current_theme;
use std::rc::Rc;

use kivact::prelude::*;
use kivact_external_component::provide_skin;
use kivact_external_component::skin::Skin;

use crate::Rng;

/// Settings of the log viewer root (tests override a few with
/// `..LogViewerOptions::default()`).
pub struct LogViewerOptions {
    pub left: Vec<LogLine>,
    pub right: Vec<LogLine>,
    pub skin: Skin,
    /// Append a line to the right pane every 700 ms (live tail).
    pub live: bool,
}

impl Default for LogViewerOptions {
    fn default() -> Self {
        LogViewerOptions {
            left: generate(7, 60_000, 95),
            right: generate(11, 20_000, 180),
            skin: Skin::Business,
            live: true,
        }
    }
}

/// Root of the log viewer.
pub fn app(options: LogViewerOptions) -> impl IntoView {
    let skin = signal(options.skin);
    let tail_seed = var(Rng::new(99));
    let left = signal(Rc::new(options.left));
    let right = signal(Rc::new(options.right));
    if options.live {
        spawn::<()>(async move {
            loop {
                kivact::task::sleep(std::time::Duration::from_millis(700)).await;
                let mut rng = tail_seed.get();
                let mut line = generate(rng.next_u64(), 1, MINUTES - 1).remove(0);
                tail_seed.set(rng);
                right.update(|v| {
                    line.id = v.len() as u32;
                    line.minute = MINUTES - 1;
                    Rc::make_mut(v).push(line);
                });
            }
        });
    }
    provide_skin(skin, move || {
        column((
            row((
                text("Kivact Log Viewer").style(style().font_size(18).weight(600)),
                spacer(),
                button(move || format!("Skin: {}", skin.get().name()))
                    .variant(ButtonVariant::Secondary)
                    .on_press(move || skin.update(|s| *s = s.other()))
                    .test_id("skin"),
            ))
            .style(style().align_items(Align::Center).gap(8)),
            row((
                log_pane("Gateway cluster", left),
                log_pane("Worker cluster (live)", right),
            ))
            .style(style().gap(12).grow(1).min_height(0)),
        ))
        .style(move || {
            let t = &current_theme();
            style()
                .padding(12)
                .gap(12)
                .grow(1)
                .background(ColorToken::SurfaceAlt)
                .color(ColorToken::Text)
                .font_size(t.text().size)
        })
    })
}
