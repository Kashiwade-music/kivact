//! Role: The client's write totals, with the accessibility counts of all windows.

use crate::model::AppModel;
use crate::mutation::WriteStats;

/// The client's write totals and the last committed revision, with the
/// accessibility counts of every window.
pub(super) fn stats_of(app: &AppModel) -> WriteStats {
    let models = || app.windows.values().map(|w| &w.model);
    WriteStats {
        a11y_constructed: models().map(|m| m.a11y.constructed).sum(),
        a11y_published: models().map(|m| m.a11y.published).sum(),
        a11y_enumerated: models().map(|m| m.a11y.enumerated).sum(),
        ..app.writes.stats
    }
}
