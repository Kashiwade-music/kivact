//! Role: The status bar of the DevTools app: target, recording and failures.

use kivact::prelude::*;
use kivact_devtools_protocol as p;

use crate::state::Session;

/// The target, the recording and emulation marks, the override count and the last failure.
pub(crate) fn view(session: Session) -> impl IntoView {
    row((
        text(move || {
            session
                .target
                .with(|target| format!("{} · pid {}", target.app.name, target.app.pid))
        }),
        show(move || recording(session), || text("Recording")),
        show(move || emulating(session), || text("Emulating")),
        text(move || {
            let count = session
                .overrides
                .latest()
                .map(|list| list.overrides.len())
                .unwrap_or(0);
            format!("{count} overrides")
        }),
        show_some(session.status, |status| text(status)),
    ))
    .style(style().gap(12).padding_x(8).padding_y(4).font_size(12))
}

/// Whether the target is recording turns, frames or input.
fn recording(session: Session) -> bool {
    session
        .recording
        .latest()
        .is_some_and(|settings| settings.updates || settings.frames || settings.input)
}

/// Whether any environment emulation is set.
fn emulating(session: Session) -> bool {
    session.emulation.latest().is_some_and(|emulation| {
        emulation.theme.is_some()
            || emulation.reduced_motion.is_some()
            || emulation.locale.is_some()
            || emulation.scale.is_some()
            || emulation.touch.is_some()
            || emulation.pseudo_locale != p::PseudoLocale::Off
    })
}
