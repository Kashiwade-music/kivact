//! Role: The time_editor_builder! macro that declares a timeline or dope sheet component.

/// Declare a time editor component named `$function`, over items of `$item`
/// with lanes of `$height` pixels. Its props are the settings of the editor;
/// `items` and `state` are its positional arguments.
macro_rules! time_editor_builder {
    ($function:ident, $item:path, $height:expr, $doc:literal) => {
        #[::kivact::component]
        #[doc = $doc]
        pub fn $function(
            items: ::kivact::state::Value<Vec<$item>>,
            state: $crate::TimeState,
            #[prop(default = false)] snap: ::kivact::state::Value<bool>,
            #[prop(default = 1.0)] step: ::kivact::state::Value<f64>,
            time_format: Option<::std::rc::Rc<dyn Fn(f64) -> String>>,
            ruler_ticks: Option<$crate::editors::shared::ruler::RulerTicks>,
            zoom_range: Option<::kivact::state::Value<::kivact::interaction::ScalarRange>>,
            min_duration: Option<::kivact::state::Value<f64>>,
            lanes: Option<::kivact::state::Value<Vec<$crate::editors::shared::model::Lane>>>,
            on_edit: Option<
                ::kivact::state::Callback<
                    $crate::editors::shared::model::EditorEvent<
                        $crate::editors::time::model::TimeRequest,
                    >,
                >,
            >,
            #[prop(default = Vec::new())] lane_cells: Vec<$crate::editors::shared::lane::LaneCell>,
            on_lane_activate: Option<::kivact::state::Callback<u64>>,
            on_activate: Option<::kivact::state::Callback<u64>>,
            context_menu: Option<$crate::editors::shared::lane::EditorMenu>,
        ) -> impl ::kivact::view::Element {
            let mut editor = $crate::editors::time::TimeEditor::new(items, state, $height);
            editor.snap = snap;
            editor.step = step;
            editor.lanes = lanes;
            editor.on_edit = on_edit;
            editor.extensions.lane_activate = on_lane_activate;
            editor.extensions.item_activate = on_activate;
            editor.extensions.menu = context_menu.map(|menu| menu.0);
            for cell in lane_cells {
                editor.extensions.cells.push((cell.width, cell.render));
            }
            if let Some(format) = time_format {
                editor.ruler.format = format;
            }
            if let Some(ticks) = ruler_ticks {
                editor.ruler.ticks = ticks;
            }
            if let Some(zoom) = zoom_range {
                editor.ruler.zoom = zoom;
            }
            if let Some(min) = min_duration {
                editor.ruler.min_duration = $crate::editors::shared::ruler::duration(min, 0.0625);
            }
            editor.into_node()
        }
    };
}
pub(crate) use time_editor_builder;
