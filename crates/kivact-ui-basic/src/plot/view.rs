//! Role: The plot builder: a canvas with drag, wheel and double-click zoom and pan, and its region.

use super::decimate::Bounds;
use super::input::{pan, zoom};
use super::model::{PlotModel, PlotRange, PlotView};
use super::paint::{layout, paint};
use super::scale::auto_view;
use kivact::custom::{Part, PartState, part};
use kivact::draw::RegionId;
use kivact::geometry::{Rect, Size};
use kivact::i18n::message_with;
use kivact::state::{Binding, Callback, IntoBinding, IntoText, IntoValue, Value, callback, signal};
use kivact::style::{Length, pct, style};
use kivact::task::{sleep, spawn};
use kivact::view::{
    Handled, HandlerResult, IntoView, MouseButton, Node, NodeBuilder, PointerEvent, PointerPhase,
    Role, View, WheelEvent, node,
};
use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

/// How long the wheel must rest before its steps are committed.
const WHEEL_SETTLE: Duration = Duration::from_millis(300);
/// The wheel's zoom factor per 40 units of `delta.y`.
const WHEEL_BASE: f64 = 1.1;
const WHEEL_UNIT: f64 = 40.0;
/// The identifier of the plot's region.
const REGION: RegionId = RegionId(0);

/// The visible range of `model` under `range`: the fit of the data, or the fixed range.
fn resolve(model: &PlotModel, range: PlotRange) -> PlotView {
    match range {
        PlotRange::Auto => auto_view(model),
        PlotRange::Manual(view) => view,
    }
}

/// The plot rectangle of `bounds`, as a rectangle.
fn plot_rect(bounds: Bounds) -> Rect {
    Rect::new(
        bounds.left,
        bounds.top,
        bounds.right - bounds.left,
        bounds.bottom - bounds.top,
    )
}

/// A plot of a [`PlotModel`]: lines, points, stems, histograms and heatmaps
/// on zoomable axes. Created by [`plot`].
pub struct Plot {
    root: Node,
    label: Value<String>,
    model: Value<PlotModel>,
    range: Binding<PlotRange>,
    on_commit: Option<Callback<PlotRange>>,
}

/// A plot of `model`, whose visible range is `range`.
///
/// The plot is a canvas named `label`. Its region is named after the label
/// and the number of series. Dragging pans, the wheel zooms the x axis about
/// the pointer (with Ctrl, the y axis; with Shift, pans x), and a double click
/// returns to [`PlotRange::Auto`]. A drag ends at the release, the wheel
/// commits 300 ms after its last step. Each commit writes `range` and calls
/// [`on_range_commit`](Plot::on_range_commit).
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic::plot::{PlotData, PlotModel, PlotRange, plot};
///
/// fn response(model: Value<PlotModel>) -> impl IntoView {
///     let range = signal(PlotRange::Auto);
///     plot("Frequency response", model, range)
/// }
/// ```
#[track_caller]
pub fn plot(
    label: impl IntoText,
    model: impl IntoValue<PlotModel>,
    range: impl IntoBinding<PlotRange>,
) -> Plot {
    Plot {
        root: node()
            .role(Role::Canvas)
            .style(style().width(pct(100)).height(Length::from(240)).shrink(0)),
        label: label.into_text_value(),
        model: model.into_value(),
        range: range.into_binding(),
        on_commit: None,
    }
}

impl Plot {
    /// Report the range at the end of each gesture (a drag, a settled wheel,
    /// a double click), after it is written to the range binding.
    pub fn on_range_commit<R: HandlerResult>(
        mut self,
        f: impl Fn(PlotRange) -> R + 'static,
    ) -> Self {
        self.on_commit = Some(callback(f));
        self
    }
}

impl NodeBuilder for Plot {
    #[doc(hidden)]
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl IntoView for Plot {
    fn into_view(self) -> View {
        let Plot {
            root,
            label,
            model,
            range,
            on_commit,
        } = self;
        let live = signal(None::<PlotView>);
        let hover = signal(None::<(f64, f64)>);
        let size = Rc::new(Cell::new(Size::new(0.0, 0.0)));
        let drag = Rc::new(Cell::new(None::<(f64, f64)>));
        let wheel_steps = Rc::new(Cell::new(0_u64));

        let input = part(|state: PartState| state);
        let region_state = input.state();
        let summary = message_with("basic.plot.summary", "{label}: {count} series", move || {
            [
                ("label", label.get()),
                ("count", model.with(|m| m.series.len()).to_string()),
            ]
        });
        let region = Part::new(region_state, move |host| {
            host.part(input).role(Role::Image).name(summary)
        });

        // The range a gesture reads from: the live one, or the committed one.
        let current = move || {
            live.get_untracked()
                .unwrap_or_else(|| model.with_untracked(|m| resolve(m, range.get_untracked())))
        };
        let commit = move |value: PlotRange| {
            live.set(None);
            range.set(value);
            if let Some(callback) = on_commit {
                callback.call(value);
            }
        };

        let draw_size = size.clone();
        let drag_size = size.clone();
        let wheel_size = size;
        let root = root
            .name(label)
            .region(REGION, region)
            .draw(move |d, canvas| {
                draw_size.set(canvas);
                let view = live
                    .get()
                    .unwrap_or_else(|| model.with(|m| resolve(m, range.get())));
                let pointer = hover.get();
                model.with(|m| {
                    paint(d, canvas, m, view, pointer);
                    d.region(REGION, plot_rect(layout(canvas, m)));
                });
            })
            .on_pointer({
                let drag = drag.clone();
                move |event: PointerEvent| {
                    let at = event.pos();
                    match event.phase() {
                        PointerPhase::Down(press) => {
                            if press.button != MouseButton::Primary {
                                return;
                            }
                            if press.clicks >= 2 {
                                drag.set(None);
                                commit(PlotRange::Auto);
                                return;
                            }
                            event.capture();
                            drag.set(Some((at.x, at.y)));
                        }
                        PointerPhase::Move => {
                            hover.set(Some((at.x, at.y)));
                            if let Some((last_x, last_y)) = drag.get() {
                                drag.set(Some((at.x, at.y)));
                                let (dx, dy) = (at.x - last_x, at.y - last_y);
                                let base = current();
                                let next = model.with_untracked(|m| {
                                    pan(m, base, layout(drag_size.get(), m), dx, dy)
                                });
                                live.set(Some(next));
                            }
                        }
                        PointerPhase::Up(_) => {
                            if drag.take().is_some() {
                                commit(PlotRange::Manual(current()));
                            }
                        }
                        PointerPhase::Cancel => {
                            hover.set(None);
                            if drag.take().is_some() {
                                commit(PlotRange::Manual(current()));
                            }
                        }
                        PointerPhase::Enter => hover.set(Some((at.x, at.y))),
                        PointerPhase::Leave => hover.set(None),
                    }
                }
            })
            .on_wheel(move |event: WheelEvent| {
                let delta = event.delta();
                let at = event.pos();
                let modifiers = event.modifiers();
                let base = current();
                let next = model.with_untracked(|m| {
                    let area = layout(wheel_size.get(), m);
                    if modifiers.shift {
                        pan(m, base, area, delta.y, 0.0)
                    } else {
                        let factor = WHEEL_BASE.powf(delta.y / WHEEL_UNIT);
                        zoom(m, base, area, (at.x, at.y), modifiers.ctrl, factor)
                    }
                });
                live.set(Some(next));
                hover.set(Some((at.x, at.y)));
                let step = wheel_steps.get() + 1;
                wheel_steps.set(step);
                let steps = wheel_steps.clone();
                spawn(async move {
                    sleep(WHEEL_SETTLE).await;
                    if steps.get() == step
                        && let Some(view) = live.get_untracked()
                    {
                        commit(PlotRange::Manual(view));
                    }
                });
                Handled::Yes
            });
        root.into_view()
    }
}
