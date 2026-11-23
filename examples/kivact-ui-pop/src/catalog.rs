//! Role: The catalog's pages, in list order.
use crate::pages::{
    buttons, cards, display, feedback, forms, messaging, navigation, overlays, overview, schedule,
    selection, table, tokens,
};
use catalog_shell::Page;

/// The catalog's pages, in list order.
pub fn pages() -> Vec<Page> {
    vec![
        overview::page(),
        tokens::page(),
        buttons::page(),
        forms::page(),
        selection::page(),
        display::page(),
        table::page(),
        feedback::page(),
        overlays::page(),
        navigation::page(),
        messaging::page(),
        schedule::page(),
        cards::page(),
    ]
}
