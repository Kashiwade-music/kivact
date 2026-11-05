//! Role: Attach menu builders to passive table column header contracts.
use crate::column_values::DataColumn;
use crate::menu::Menu;
use std::rc::Rc;

impl<R: 'static> DataColumn<R> {
    /// Open a menu from this column header's secondary click or context-menu key.
    pub fn header_context_menu<K: kivact_core::state::ItemKey>(
        mut self,
        menu: impl Fn() -> Menu<K> + 'static,
    ) -> Self {
        self.header_menu = Some(Rc::new(move |header| menu().attach_context(header)));
        self
    }
}
