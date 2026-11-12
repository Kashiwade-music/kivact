//! Role: The Plot builder: a canvas whose range the pointer and keys edit.

use super::input::{Nudge, drag_view, wheel_view};
use super::model::{PlotModel, PlotRange, PlotView};
use super::paint::paint;
use super::scale::plot_area;
use super::ticks::format_value;
use kivact::geometry::{Point, Rect, Size};
use kivact::i18n::{Locale, message_with, use_locale};
use kivact::interaction::{DragApi, DragCoordinates, DragUpdate, EditSession, use_drag};
use kivact::state::{
    Binding, Callback, IntoBinding, IntoText, IntoValue, Signal, Value, callback, signal,
};
use kivact::style::{Length, pct, style};
use kivact::view::{
    EditPhase, Element, Handled, HandlerResult, IntoView, KeyEvent, MouseButton, Node,
    PointerEvent, PointerPhase, Role, View, WheelEvent, column, node, node_ref,
};

/// The accessible description when the App has no `basic.plot.description` message.
const DESCRIPTION: &str = "{series}. {x}: {x_min} to {x_max}. {y}: {y_min} to {y_max}.";

/// A plot of a [`PlotModel`]: lines, points, stems, histograms and heatmaps
/// on zoomable axes. Created by [`plot`].
pub struct Plot {
    root: Node,
    label: Value<String>,
    model: Value<PlotModel>,
    range: Option<Binding<PlotRange>>,
    on_edit: Option<Callback<EditPhase>>,
    height: Value<Length>,
}

/// A plot of `model`, named `label`, with the visible range it shows.
///
/// The plot is a focusable canvas of `Role::Image` named `label` and described
/// by its series and its visible range. Dragging pans, the wheel zooms the x
/// axis about the pointer (with Ctrl, the y axis; a horizontal wheel pans x),
/// the keys `+`, `-`, PageUp, PageDown and the arrows zoom and pan, `0` and
/// Home return to [`PlotRange::Auto`], and a double click does the same. Each
/// of these is one edit of the range, reported by [`Plot::on_edit`].
///
/// ```no_run
/// use kivact::prelude::*;
/// use kivact_ui_basic::{PlotAxis, PlotData, PlotModel, PlotSeries, PlotSeriesKind, plot};
///
/// fn response() -> impl IntoView {
///     let frequencies = PlotData::new(vec![10.0, 100.0, 1000.0]);
///     let gain = PlotData::new(vec![0.0, -3.0, -12.0]);
///     let model = PlotModel::new(PlotAxis::new("Frequency"), PlotAxis::new("Gain")).series(
///         PlotSeries::new("Gain", PlotSeriesKind::Line { x: frequencies, y: gain }),
///     );
///     plot("Frequency response", model)
/// }
/// ```
#[track_caller]
pub fn plot(label: impl IntoText, model: impl IntoValue<PlotModel>) -> Plot {
    Plot {
        root: column(()).style(style().width(pct(100)).min_width(0)),
        label: label.into_text_value(),
        model: model.into_value(),
        range: None,
        on_edit: None,
        height: Length::from(240).into_value(),
    }
}

impl Plot {
    /// Show and edit the visible range through `range`. Each edit writes the
    /// new `PlotRange::Manual` to the binding at once. Without a binding the
    /// plot keeps its own range, starting at `PlotRange::Auto`.
    pub fn range(mut self, range: impl IntoBinding<PlotRange>) -> Self {
        self.range = Some(range.into_binding());
        self
    }

    /// Report each edit of the range: `Begin` when it starts, `Commit` when it
    /// is kept, and `Cancel` when Escape or a pointer cancel restores the range
    /// it started from. A wheel step, a key or a double click is one edit that
    /// begins and commits at once. Setup only.
    pub fn on_edit<R: HandlerResult>(mut self, f: impl Fn(EditPhase) -> R + 'static) -> Self {
        self.on_edit = Some(callback(f));
        self
    }

    /// Set the height of the plot (default 240 logical pixels), constant or
    /// reactive.
    #[track_caller]
    pub fn height(mut self, height: impl IntoValue<Length>) -> Self {
        self.height = height.into_value();
        self
    }
}

impl Element for Plot {
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
            on_edit,
            height,
        } = self;
        let range = match range {
            Some(range) => range,
            None => signal(PlotRange::Auto).into_binding(),
        };
        let locale = use_locale();
        let hover = signal(None::<Point>);
        let target = node_ref();
        let size = target.rect();
        let edit = session_edit(range, on_edit);
        let session = use_drag::<PlotView>(DragCoordinates::Local)
            .edit_on_change(edit, move || range.get_untracked());
        let drag = session.on_update(move |update: DragUpdate<PlotView>| {
            apply_drag(session, &update, size, model, range);
        });
        let dragging = drag.active_payload();
        let source = drag.source_with(move || current_view(range, model));
        let description = message_with("basic.plot.description", DESCRIPTION, move || {
            let view = model.with(|m| range.with(|r| r.view(m)));
            locale.with(|locale| model.with(|m| describe(m, view, locale)))
        });
        let canvas = node()
            .role(Role::Image)
            .name(label)
            .description(description)
            .focusable(true)
            .style(move || style().width(pct(100)).height(height.get()).shrink(0))
            .node_ref(target)
            .draw(move |d, canvas| {
                let view = model.with(|m| range.with(|r| r.view(m)));
                let at = hover.get();
                locale.with(|locale| model.with(|m| paint(d, canvas, m, view, at, locale)));
            })
            .on_pointer(pointer_handler(hover, edit, range))
            .on_wheel(wheel_handler(edit, range, model, size, dragging))
            .on_key(key_handler(edit, range, model, dragging))
            .part(source);
        root.child(canvas).into_view()
    }
}

/// The edit session of the range: its start is written back on cancel, and
/// an observer given to [`Plot::on_edit`] hears each phase.
fn session_edit(
    range: Binding<PlotRange>,
    on_edit: Option<Callback<EditPhase>>,
) -> EditSession<PlotRange> {
    let edit = EditSession::new(move |initial: PlotRange| range.set(initial));
    match on_edit {
        Some(observer) => edit.on_edit(move |phase| observer.try_call(phase)),
        None => edit,
    }
}

/// The visible range the plot shows now, read without tracking.
fn current_view(range: Binding<PlotRange>, model: Value<PlotModel>) -> PlotView {
    model.with_untracked(|m| range.get_untracked().view(m))
}

/// The plot area of the canvas, or `None` while the canvas has no room for it.
fn current_area(size: Value<Option<Rect>>, model: Value<PlotModel>) -> Option<Rect> {
    let bounds = size.get_untracked()?;
    let area = model.with_untracked(|m| plot_area(Size::new(bounds.width, bounds.height), m));
    (area.width > 0.0 && area.height > 0.0).then_some(area)
}

/// One drag update: the view the drag started from, moved by the pointer.
/// The first change of the drag begins its edit.
fn apply_drag(
    session: DragApi<PlotView>,
    update: &DragUpdate<PlotView>,
    size: Value<Option<Rect>>,
    model: Value<PlotModel>,
    range: Binding<PlotRange>,
) {
    if update.delta == Point::new(0.0, 0.0) {
        return;
    }
    let Some(area) = current_area(size, model) else {
        return;
    };
    let next = model.with_untracked(|m| drag_view(m, update.payload, area, update.delta));
    session.change(|| range.set(PlotRange::Manual(next)));
}

/// The pointer handler: a double click returns to the automatic range, and
/// the pointer position feeds the crosshair. Every other event goes on to the
/// drag source.
fn pointer_handler(
    hover: Signal<Option<Point>>,
    edit: EditSession<PlotRange>,
    range: Binding<PlotRange>,
) -> impl Fn(PointerEvent) -> Handled + 'static {
    move |event: PointerEvent| match event.phase() {
        PointerPhase::Down(press) if press.button == MouseButton::Primary && press.clicks >= 2 => {
            edit.once(range.get_untracked(), || range.set(PlotRange::Auto));
            Handled::Yes
        }
        PointerPhase::Move | PointerPhase::Enter => {
            hover.set(Some(event.pos()));
            Handled::No
        }
        PointerPhase::Leave | PointerPhase::Cancel => {
            hover.set(None);
            Handled::No
        }
        PointerPhase::Down(_) | PointerPhase::Up(_) => Handled::No,
    }
}

/// The wheel handler: one wheel event is one edit of the range. A wheel
/// during a drag is consumed without changing the range.
fn wheel_handler(
    edit: EditSession<PlotRange>,
    range: Binding<PlotRange>,
    model: Value<PlotModel>,
    size: Value<Option<Rect>>,
    dragging: Value<Option<PlotView>>,
) -> impl Fn(WheelEvent) -> Handled + 'static {
    move |event: WheelEvent| {
        if dragging.with_untracked(Option::is_some) {
            return Handled::Yes;
        }
        let (at, delta) = (event.pos(), event.delta());
        if !is_finite(at) || !is_finite(delta) || delta == Point::new(0.0, 0.0) {
            return Handled::No;
        }
        let Some(area) = current_area(size, model) else {
            return Handled::No;
        };
        let base = current_view(range, model);
        let ctrl = event.modifiers().ctrl;
        let next = model.with_untracked(|m| wheel_view(m, base, area, at, delta, ctrl));
        edit.once(range.get_untracked(), || range.set(PlotRange::Manual(next)));
        Handled::Yes
    }
}

/// The key handler: `+`, `-`, PageUp, PageDown, the arrows, `0` and Home are
/// one edit each. A key with Ctrl, Alt or Meta, and any key during a drag, is
/// left to the rest of the chain (the drag takes Escape).
fn key_handler(
    edit: EditSession<PlotRange>,
    range: Binding<PlotRange>,
    model: Value<PlotModel>,
    dragging: Value<Option<PlotView>>,
) -> impl Fn(KeyEvent) -> Handled + 'static {
    move |event: KeyEvent| {
        let modifiers = event.modifiers();
        if dragging.with_untracked(Option::is_some)
            || modifiers.ctrl
            || modifiers.alt
            || modifiers.meta
        {
            return Handled::No;
        }
        let Some(nudge) = Nudge::of_key(event.key()) else {
            return Handled::No;
        };
        let base = current_view(range, model);
        let next = model.with_untracked(|m| nudge.apply(m, base));
        edit.once(range.get_untracked(), || range.set(next));
        Handled::Yes
    }
}

/// Whether both coordinates of `p` are finite.
fn is_finite(p: Point) -> bool {
    p.x.is_finite() && p.y.is_finite()
}

/// The arguments of the accessible description: the series names, the axis
/// titles and the ends of the visible range, written in `locale`.
fn describe(model: &PlotModel, view: PlotView, locale: &Locale) -> Vec<(&'static str, String)> {
    let names = model
        .series
        .iter()
        .map(|series| series.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let x = |value: f64| format_value(value, &model.x.format, 0.0, locale);
    let y = |value: f64| format_value(value, &model.y.format, 0.0, locale);
    vec![
        ("series", names),
        ("x", model.x.label.clone()),
        ("x_min", x(view.x_min)),
        ("x_max", x(view.x_max)),
        ("y", model.y.label.clone()),
        ("y_min", y(view.y_min)),
        ("y_max", y(view.y_max)),
    ]
}

#[cfg(test)]
mod tests {
    //! Role: Tests for the accessible description arguments.

    use super::*;
    use crate::plot::model::{PlotAxis, PlotData, PlotSeries, PlotSeriesKind};

    #[test]
    fn the_description_names_the_series_and_the_range_in_the_locale() {
        let model =
            PlotModel::new(PlotAxis::new("Hz"), PlotAxis::new("dB")).series(PlotSeries::new(
                "Left",
                PlotSeriesKind::Line {
                    x: PlotData::new(vec![0.0, 1.0]),
                    y: PlotData::new(vec![0.0, 1.0]),
                },
            ));
        let view = PlotView {
            x_min: 0.0,
            x_max: 0.5,
            y_min: -12.0,
            y_max: 1.0,
        };
        let args = describe(&model, view, &Locale::new("de-DE"));
        let value = |key: &str| {
            args.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, text)| text.as_str())
        };
        assert_eq!(value("series"), Some("Left"));
        assert_eq!(value("x"), Some("Hz"));
        assert_eq!(value("x_max"), Some("0,5"));
        assert_eq!(value("y_min"), Some("-12"));
    }
}
