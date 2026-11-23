//! Role: The catalog's page list: every page, in list order.

use catalog_shell::Page;

/// Every page of the catalog, in list order.
pub fn pages() -> Vec<Page> {
    vec![
        super::overview::page(),
        super::buttons::page(),
        super::numeric_fields::page(),
        super::selection_input::page(),
        super::indicators_labels::page(),
        super::transport::page(),
        super::knobs_faders::page(),
        super::meters::page(),
        super::mixer_console::page(),
        super::property_editor::page(),
        super::areas_docking::page(),
        super::outliner::page(),
        super::media_pool_data_table::page(),
        super::layer_stack::page(),
        super::arrange_timeline::page(),
        super::dope_sheet_graph::page(),
        super::node_editor::page(),
        super::color_grading_scopes::page(),
        super::color_picker::page(),
        super::viewport_3d::page(),
        super::menus::page(),
        super::dialogs_popovers::page(),
        super::jobs_log::page(),
    ]
}
