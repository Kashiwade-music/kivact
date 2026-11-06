//! Role: One month of the calendar: caption, weekday row and day cells.

use super::caption::{Names, title};
use super::state::Shared;
use crate::layer::with_recipe_style;
use crate::{ButtonVariant, ControlSize};
use kivact_core::custom::{Part, PartState};
use kivact_core::i18n::{CalendarDate, CalendarMonth, Weekday};
use kivact_core::style::{Align, Direction, Justify, Position, TextAlign, pct, style};
use kivact_core::view::{IntoView, Role, View, show};
use kivact_core::view::{Node, NodeBuilder, column, node, row, text};
use std::rc::Rc;

/// A ghost icon button of the caption row, 28 px square, behind a paging part.
fn nav_button(
    glyph: &'static str,
    name: kivact_core::state::Value<String>,
    paging: Part<PartState>,
) -> View {
    crate::button(name)
        .variant(ButtonVariant::Ghost)
        .hide_label(true)
        .icon(kivact_core::view::icon(glyph))
        .size(ControlSize::Small)
        .with_node(|root| root.part(paging))
        .into_view()
}

fn spacer(size: f64) -> Node {
    node().style(style().size(size, size).shrink(0))
}

pub(super) fn month_panel(
    shared: &Rc<Shared>,
    names: &Rc<Names>,
    index: usize,
    month: CalendarMonth,
    first: Weekday,
    entering: f64,
) -> Node {
    let grid = shared.grid;
    let swipe = shared.swipe;
    let caption = month_caption(shared, names, index, month);
    let locale = kivact_core::i18n::use_locale();
    let headers = row((0..7)
        .map(|c| {
            node()
                .child(text(names.weekday(first.days_after(c))))
                .with_node(|root| root.role(Role::ColumnHeader))
                .name(move || {
                    locale
                        .get()
                        .weekday_name(first.days_after(c), kivact_core::i18n::NameWidth::Long)
                })
                .style(
                    style()
                        .width(shared.cell_size)
                        .height(20)
                        .font_size(12.8)
                        .text_align(TextAlign::Center)
                        .color(kivact_core::style::ColorToken::TextMuted),
                )
                .into_view()
        })
        .collect::<Vec<_>>())
    .with_node(|root| root.part(grid.row()));
    let rows = month
        .weeks(first, if shared.variable_rows.get() { 0 } else { 6 })
        .into_iter()
        .map(|week| {
            row(week
                .into_iter()
                .map(|day| day_cell(shared, month, day))
                .collect::<Vec<_>>())
            .with_node(|root| root.part(grid.row()))
            .style(
                style().margin_top(
                    kivact_core::style::use_theme()
                        .get()
                        .ext::<crate::UiTokens>()
                        .calendar
                        .week_gap,
                ),
            )
            .style(move || week_motion(swipe.get(), entering))
            .into_view()
        })
        .collect::<Vec<_>>();
    with_recipe_style(
        column((
            caption,
            column((headers, rows))
                .with_node(|root| root.part(grid.grid()))
                .name(format!("{}-{:02}", month.year(), month.month()))
                .style(style().overflow(kivact_core::style::Overflow::Hidden)),
        ))
        .style(
            style()
                .gap(
                    kivact_core::style::use_theme()
                        .get()
                        .ext::<crate::UiTokens>()
                        .calendar
                        .month_gap,
                )
                .width(shared.cell_size * 7.0)
                .shrink(0),
        ),
        |r| r.calendar_recipes().month_recipe(),
        &shared.variants,
        PartState::default,
    )
}

/// The weeks follow a touch swipe's finger and, when the swipe turned the
/// month, spring in from the side it brought the new month from.
fn week_motion(swipe: super::navigation::Swipe, entering: f64) -> kivact_core::style::Style {
    let shift = swipe.shift();
    let base = style().translate_x(shift);
    if entering == 0.0 || matches!(swipe, super::navigation::Swipe::Dragging { .. }) {
        return base;
    }
    base.enter(style().translate_x(entering)).transition(
        kivact_core::style::TransitionProp::Paint,
        kivact_core::motion::Transition::spring(kivact_core::motion::Spring {
            stiffness: 246.7,
            damping: 31.4,
            mass: 1.0,
        }),
    )
}

/// One day: a band behind a button, as calendar.tsx draws a range.
fn day_cell(shared: &Rc<Shared>, month: CalendarMonth, day: CalendarDate) -> View {
    if day.month_of() == month {
        return day_node(shared, month, day).into_view();
    }
    let cell_size = shared.cell_size;
    let shared = shared.clone();
    show(shared.outside, move || day_node(&shared, month, day))
        .fallback(move || spacer(cell_size))
        .into_view()
}

fn day_node(shared: &Rc<Shared>, month: CalendarMonth, day: CalendarDate) -> Node {
    let swipe = shared.swipe;
    let cell = shared.grid.cell(month, day);
    let observed = cell.state();
    let range = shared.range;
    let marked = kivact_core::state::memo(move || {
        let mut state = observed.get();
        if let Some(range) = range.get()
            && day >= range.start()
            && day <= range.end()
        {
            state.selected = true;
            state.edge = day == range.start() || day == range.end();
            state.span = if range.start() == range.end() {
                kivact_interaction::CalendarSpan::None
            } else if day == range.start() {
                kivact_interaction::CalendarSpan::Start
            } else if day == range.end() {
                kivact_interaction::CalendarSpan::End
            } else {
                kivact_interaction::CalendarSpan::Middle
            };
        }
        state
    });
    let cell_size = shared.cell_size;
    let in_month = day.month_of() == month;
    let today = shared.today;
    let marks = shared.marks.clone();
    let weekend = shared.weekend.clone();
    let button = with_recipe_style(
        interactive_day(swipe)
            .with_node(|root| root.part(cell))
            .selected(move || marked.get().selected)
            .child(text(day.day().to_string()))
            .child(event_marks(shared, day, marked)),
        |r| r.calendar_recipes().day_recipe(),
        &shared.variants,
        move || crate::calendar_values::CalendarDayVisual {
            date: day,
            cell_size,
            day: marked.get(),
            today: day == today.get(),
            in_month,
            mark: marks(day).or_else(|| weekend(day.weekday())),
        },
    );
    let button = button.style(move || style().size(cell_size, cell_size));
    let band = with_recipe_style(
        node(),
        |r| r.calendar_recipes().range_recipe(),
        &shared.variants,
        move || marked.get(),
    );
    node().child(band).child(button).style(
        style()
            .position(Position::Relative)
            .size(shared.cell_size, shared.cell_size)
            .shrink(0),
    )
}

fn event_marks(
    shared: &Shared,
    day: CalendarDate,
    marked: kivact_core::state::Value<kivact_interaction::CalendarDayState>,
) -> Node {
    let events = shared.events;
    let variants = shared.variants.clone();
    let contents = kivact_core::view::dynamic(
        move || {
            events
                .get()
                .into_iter()
                .filter(|event| event.date == day)
                .take(3)
                .collect::<Vec<_>>()
        },
        move |events| {
            events
                .into_iter()
                .map(|event| {
                    with_recipe_style(
                        node(),
                        |r| r.calendar_recipes().event_recipe(),
                        &variants,
                        move || crate::calendar_tokens::CalendarEventVisual {
                            color: event.color,
                            selected: marked.get().edge,
                        },
                    )
                    .into_view()
                })
                .collect::<Vec<_>>()
        },
    );
    with_recipe_style(
        node().child(contents),
        |r| r.calendar_recipes().events_recipe(),
        &shared.variants,
        PartState::default,
    )
}

/// Render month/year paging and the theme-owned caption row.
fn month_caption(
    shared: &Rc<Shared>,
    names: &Rc<Names>,
    index: usize,
    month: CalendarMonth,
) -> Node {
    let rtl = kivact_core::view::use_direction().get() == kivact_core::view::ReadingDirection::Rtl;
    let (back, forward) = if rtl {
        ("chevron-right", "chevron-left")
    } else {
        ("chevron-left", "chevron-right")
    };
    let grid = shared.grid;
    let previous = if index == 0 {
        nav_button(back, names.previous, grid.previous_month())
    } else {
        spacer(shared.cell_size).into_view()
    };
    let next = if index + 1 == shared.count {
        nav_button(forward, names.next, grid.next_month())
    } else {
        spacer(shared.cell_size).into_view()
    };
    let previous_year = show(shared.year_navigation, move || {
        nav_button(
            "chevrons-left",
            kivact_core::i18n::message("calendar.previous_year", "Previous year"),
            grid.previous_year(),
        )
    });
    let next_year = show(shared.year_navigation, move || {
        nav_button(
            "chevrons-right",
            kivact_core::i18n::message("calendar.next_year", "Next year"),
            grid.next_year(),
        )
    });
    let caption = row((
        previous_year,
        previous,
        node().child(title(shared, names, index, month)).style(
            style()
                .grow(1)
                .direction(Direction::Row)
                .justify_content(Justify::Center)
                .align_items(Align::Center),
        ),
        next,
        next_year,
    ))
    .style(
        style()
            .direction(if rtl {
                Direction::RowReverse
            } else {
                Direction::Row
            })
            .align_items(Align::Center)
            .height(shared.cell_size)
            .width(pct(100)),
    );
    with_recipe_style(
        caption,
        |r| r.calendar_recipes().caption_recipe(),
        &shared.variants,
        PartState::default,
    )
}

fn interactive_day(
    swipe: kivact_core::state::MachineHandle<super::navigation::SwipeSnapshot>,
) -> Node {
    node()
        .on_pointer(move |event: kivact_core::view::PointerEvent| {
            let dragging = matches!(swipe.get(), super::navigation::Swipe::Dragging { .. });
            swipe.send(super::navigation::Event::Pointer(event));
            if matches!(swipe.get(), super::navigation::Swipe::Dragging { .. }) {
                event.capture();
                kivact_core::view::Handled::Yes
            } else if dragging {
                kivact_core::view::Handled::Yes
            } else {
                kivact_core::view::Handled::No
            }
        })
        .on_key(move |_| {
            swipe.send(super::navigation::Event::IndependentActivation);
            kivact_core::view::Handled::No
        })
        .a11y_action(kivact_core::view::A11yAction::Activate, move |_| {
            swipe.send(super::navigation::Event::IndependentActivation);
            kivact_core::view::Handled::No
        })
}
