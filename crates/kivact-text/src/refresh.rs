//! Role: Explicit incremental reconciliation and debug integrity checkpoints.

use crate::records::{Key, Shown};
use crate::{EditLines, RefreshError, RefreshMode, Source};

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
        style: ui_types::TextStyle,
        engine: &mut arvane::Engine,
        mode: RefreshMode,
    ) -> Result<(), RefreshError> {
        self.reconcile(source, style, engine);
        #[cfg(debug_assertions)]
        if mode == RefreshMode::Verify {
            let text = self.shown(source.value, source.placeholder);
            let discrepancies = self.discrepancies(text);
            if !discrepancies.is_empty() {
                return Err(RefreshError { discrepancies });
            }
        }
        #[cfg(not(debug_assertions))]
        let _ = mode;
        Ok(())
    }

    fn rebuild_with_metrics(&mut self, text: &str, engine: &mut arvane::Engine) {
        let style = &self.key().style;
        let height = crate::line_height(engine, style);
        let blank = engine
            .shape(" ", style, None)
            .map_or(height, |s| s.baseline);
        self.rebuild(text, height, blank);
    }
    /// Bring the lines up to what a text input shows (`source`) in
    /// `style`.
    fn reconcile(
        &mut self,
        source: Source<'_>,
        mut style: ui_types::TextStyle,
        engine: &mut arvane::Engine,
    ) {
        let shown = if source.value.is_empty() {
            Shown::Placeholder(source.placeholder.to_owned())
        } else if source.password {
            Shown::Masked
        } else {
            Shown::Value
        };
        // Shaped start aligned and without a color: aligning and coloring
        // (a hover, a focus style) never shape again.
        self.align = style.align;
        style.align = ui_types::TextAlign::Start;
        style.color = ui_types::TextStyle::default().color;
        let key = Key {
            style,
            shown,
            multiline: source.multiline,
        };
        let text = self.shown(source.value, source.placeholder);
        if self.key.as_ref() != Some(&key) {
            self.key = Some(key);
            let text = self.shown(source.value, source.placeholder);
            self.rebuild_with_metrics(text, engine);
        } else if self.revision != source.revision && self.shows_value() {
            let fits = source
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
        match self.key.as_ref().map(|k| &k.shown) {
            Some(Shown::Placeholder(_)) => placeholder,
            _ => value,
        }
    }
}
