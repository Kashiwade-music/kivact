//! Role: Compare retained geometry and baselines with disposable fresh owners.
use crate::measure::leaf::{EngineMeasure, Measure};
use crate::model::Model;
impl Model {
    pub(crate) fn layout_self_check(&mut self, engine: &mut arvane::Engine) -> Vec<String> {
        let mut fresh = self.tree.clone();
        fresh.pack_child_orders();
        let mut state = self.layout_state.clone();
        let mut columns = self.layout_columns.clone();
        columns.rebuild_all(&fresh, &mut state);
        let cache = self.measure.clone();
        for extents in &cache.extents {
            *extents.borrow_mut() = None;
        }
        let editors = self.editors.clone();
        let mut measure = Measure::new(engine, self.assets.get_mut(), &cache, &editors);
        let mut leaves = EngineMeasure {
            tree: &fresh,
            measure: &mut measure,
        };
        let mut differences = self
            .layout_columns
            .compare_measurements(&self.tree, &mut leaves);
        columns.compute(&fresh, &mut state, self.viewport, &mut leaves);
        // Placement reads anchor bounds from the model. Give it the fresh
        // geometry during the second pass, then restore the retained owners.
        std::mem::swap(&mut self.layout_columns, &mut columns);
        std::mem::swap(&mut self.layout_state, &mut state);
        self.refit_tooltips(engine, false);
        std::mem::swap(&mut self.layout_columns, &mut columns);
        std::mem::swap(&mut self.layout_state, &mut state);
        differences.extend(self.layout_columns.compare(&columns, &self.tree));
        differences
            .into_iter()
            .map(|d| format!("layout node {} {}", d.id.0, d.detail))
            .collect()
    }
}
