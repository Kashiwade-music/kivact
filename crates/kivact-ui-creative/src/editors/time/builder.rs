//! Role: The time_editor_builder! macro giving Timeline and DopeSheet a builder.

macro_rules! time_editor_builder {
    ($type:ident, $function:ident, $item:path, $height:expr, $type_doc:literal, $function_doc:literal) => {
        #[doc = $type_doc]
        pub struct $type {
            inner: $crate::editors::time::TimeEditor<$item>,
        }
        #[doc = $function_doc]
        #[track_caller]
        pub fn $function(
            items: impl kivact::state::IntoValue<Vec<$item>>,
            state: $crate::TimeState,
        ) -> $type {
            $type {
                inner: $crate::editors::time::TimeEditor::new(items, state, $height),
            }
        }
        impl $type {
            $crate::editors::shared::lane::lane_extension_methods!(inner.extensions);
            /// Report absolute item proposals and gesture boundaries.
            /// The application applies requests and restores its items on cancellation.
            pub fn on_edit<R: kivact::view::HandlerResult>(
                mut self,
                f: impl Fn(
                    $crate::editors::shared::model::EditorEvent<
                        $crate::editors::time::model::TimeRequest,
                    >,
                ) -> R
                + 'static,
            ) -> Self {
                self.inner.on_edit = Some(kivact::state::callback(f));
                self
            }
            /// Provide reactive lane labels and ordering.
            pub fn lanes(
                mut self,
                lanes: impl kivact::state::IntoValue<Vec<$crate::editors::shared::model::Lane>>,
            ) -> Self {
                self.inner.lanes = Some(kivact::state::IntoValue::into_value(lanes));
                self
            }
            /// Enable snapping for pointer edits.
            pub fn snap(mut self, snap: impl kivact::state::IntoValue<bool>) -> Self {
                self.inner.snap = kivact::state::IntoValue::into_value(snap);
                self
            }
            /// Set the reactive time increment for keys and pointer snapping (default: 1).
            /// Non-finite or non-positive values use 1. Shift keys move by ten increments.
            /// Moves beyond the finite numeric range leave the value unchanged.
            pub fn step(mut self, step: impl kivact::state::IntoValue<f64>) -> Self {
                self.inner.step = kivact::state::IntoValue::into_value(step);
                self
            }
            /// Format ruler ticks and the playhead using application time units.
            pub fn time_format(mut self, format: impl Fn(f64) -> String + 'static) -> Self {
                self.inner.ruler.format = std::rc::Rc::new(format);
                self
            }
            /// Choose automatic or application-provided ruler ticks.
            pub fn ruler_ticks(
                mut self,
                ticks: $crate::editors::shared::ruler::RulerTicks,
            ) -> Self {
                self.inner.ruler.ticks = ticks;
                self
            }
            /// Set finite positive zoom bounds; invalid bounds report a usage error.
            pub fn zoom_range(
                mut self,
                range: impl kivact::state::IntoValue<kivact::interaction::ScalarRange>,
            ) -> Self {
                self.inner.ruler.zoom = kivact::state::IntoValue::into_value(range);
                self
            }
            /// Set the minimum clip duration; invalid lengths report a usage error.
            pub fn min_duration(mut self, duration: impl kivact::state::IntoValue<f64>) -> Self {
                self.inner.ruler.min_duration =
                    $crate::editors::shared::ruler::duration(duration, 0.0625);
                self
            }
        }
        impl kivact::view::Element for $type {
            /// Return the editor root for ordinary node modifiers.
            #[doc(hidden)]
            fn node_mut(&mut self) -> &mut kivact::view::Node {
                &mut self.inner.root
            }
        }
        #[diagnostic::do_not_recommend]
        impl kivact::view::IntoView for $type {
            /// Build the editor view.
            fn into_view(self) -> kivact::view::View {
                kivact::view::IntoView::into_view(self.inner)
            }
        }
    };
}
pub(crate) use time_editor_builder;
