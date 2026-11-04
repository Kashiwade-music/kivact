//! Role: Explicit incremental reconciliation and debug integrity checkpoints.

use crate::EditLines;
use crate::records::{Key, Shown};
use crate::source::Source;

/// Cost and diagnostic assurance requested for one source reconciliation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RefreshMode {
    /// Reconcile incrementally without a complete representation scan.
    #[default]
    Incremental,
    /// In debug builds, independently verify retained line metadata afterward.
    /// This expensive checkpoint is useful after a source batch or adoption.
    /// In release builds it performs ordinary reconciliation only; no integrity
    /// scan, reference allocation or diagnostic assurance is provided.
    Verify,
}

/// Discrepancies found at an explicitly requested debug integrity checkpoint.
/// The source and retained cache are left intact for diagnosis, never healed
/// from the independent reference. No private representation is exposed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshError {
    /// Located descriptions of retained/reference metadata differences.
    pub discrepancies: Vec<String>,
}

impl std::fmt::Display for RefreshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.discrepancies.join("; "))
    }
}

impl std::error::Error for RefreshError {}

impl EditLines {
    /// Reconcile one source/style snapshot, retaining untouched derived caches.
    /// Later queries must use this snapshot's displayed source. Compatible damage
    /// updates touched hard lines; changed keys or incompatible damage rebuild.
    /// `Verify` additionally checks metadata in debug builds, including no-op
    /// reconciliations. Release builds reconcile normally in either mode.
    /// Integrity failure returns evidence without resetting or healing the cache.
    pub fn refresh(
        &mut self,
        source: Source<'_>,
        style: kivact_types::ComputedTextStyle,
        engine: &mut arvane::Engine,
        mode: RefreshMode,
    ) -> Result<(), RefreshError> {
        self.reconcile(source, style, engine);
        #[cfg(debug_assertions)]
        if mode == RefreshMode::Verify {
            let text = self.shown(source.value, source.placeholder);
            let discrepancies = self.discrepancies(text, engine);
            if !discrepancies.is_empty() {
                return Err(RefreshError { discrepancies });
            }
        }
        #[cfg(not(debug_assertions))]
        let _ = mode;
        Ok(())
    }

    fn rebuild_with_metrics(&mut self, text: &str, engine: &mut arvane::Engine) {
        let style = &self.key.style;
        let height = crate::line_height(engine, style);
        let blank = engine
            .shape(" ", style, None)
            .map_or(height, |s| s.baseline);
        self.rebuild(text, height, blank, engine);
    }
    /// Bring the lines up to what a text input shows (`source`) in
    /// `style`.
    fn reconcile(
        &mut self,
        source: Source<'_>,
        style: kivact_types::ComputedTextStyle,
        engine: &mut arvane::Engine,
    ) {
        let (key, align) = Key::of(&source, style);
        self.align = align;
        let text = self.shown(source.value, source.placeholder);
        if self.key != key {
            self.key = key;
            let text = self.shown(source.value, source.placeholder);
            self.rebuild_with_metrics(text, engine);
        } else if self.revision != source.revision && self.shows_value() {
            let fits = self.key.wrap_width.is_none()
                && source
                    .damage
                    .filter(|d| d.base == self.revision)
                    .is_some_and(|d| self.splice(text, d));
            if !fits {
                self.rebuild_with_metrics(text, engine);
            }
        }
        self.revision = source.revision;
    }
    /// The text the lines are of: `value`, or `placeholder` while they
    /// show it.
    pub fn shown<'a>(&self, value: &'a str, placeholder: &'a str) -> &'a str {
        match self.key.shown {
            Shown::Placeholder(_) => placeholder,
            _ => value,
        }
    }
}
