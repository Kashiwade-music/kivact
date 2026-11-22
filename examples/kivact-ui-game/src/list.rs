//! Role: The game catalog's page list: the eleven pages, in list order.

use catalog_shell::Page;

/// The eleven pages in the order of the list.
pub fn pages() -> Vec<Page> {
    vec![
        crate::pages::foundations::page(),
        crate::pages::buttons::page(),
        crate::pages::inputs::page(),
        crate::pages::selection::page(),
        crate::pages::navigation::page(),
        crate::pages::data_display::page(),
        crate::pages::data_table::page(),
        crate::pages::feedback::page(),
        crate::pages::overlays::page(),
        crate::pages::native_layout::page(),
        crate::pages::effects::page(),
    ]
}
