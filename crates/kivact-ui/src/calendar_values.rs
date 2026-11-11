//! Role: Define passive calendar captions, event marks and day recipe projections.
use kivact_core::i18n::CalendarDate;

/// How the title of each month is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CalendarCaption {
    /// A text such as `October 2026`.
    #[default]
    Label,
    /// A month select and a year select.
    Dropdown,
}

kivact_core::state::constant_value!(CalendarCaption);

/// One colored event mark; the application's scheduling model owns its contents.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarEvent {
    pub date: CalendarDate,
    pub color: kivact_core::style::Color,
}

/// The logical state supplied to a calendar day recipe.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalendarDayVisual {
    pub cell_size: f64,
    /// Date represented by this recipe projection.
    pub date: CalendarDate,
    pub day: kivact_interaction::CalendarDayState,
    // xtask-allow(state-fields): independent: date == the clock's today; the grid shows today inside or outside the displayed month, with or without a tone
    pub today: bool,
    // xtask-allow(state-fields): independent: date lies in the displayed month; the grid's leading and trailing days are outside it whether or not they are today or toned
    pub in_month: bool,
    // xtask-allow(state-fields): independent: the tone from Calendar::marks, else Calendar::weekend, for this date; the caller's callbacks may tone any date, today or not, in the month or not
    pub mark: Option<kivact_interaction::Tone>,
}
