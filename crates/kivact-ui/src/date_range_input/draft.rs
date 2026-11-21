//! Role: The draft of a date range: its two endpoints and their fields' validity.
use kivact_core::i18n::{CalendarDate, DateRange};

#[derive(Clone, Copy, PartialEq)]
pub(super) enum FieldValidity {
    Valid,
    StartInvalid,
    EndInvalid,
    BothInvalid,
}

/// The endpoints the two fields published and whether each field holds valid text.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct RangeFields {
    pub(super) start: Option<CalendarDate>,
    pub(super) end: Option<CalendarDate>,
    pub(super) fields: FieldValidity,
}

impl RangeFields {
    pub(super) fn following(range: &Option<DateRange>) -> Self {
        let range = *range;
        Self {
            start: range.map(DateRange::start),
            end: range.map(DateRange::end),
            fields: FieldValidity::Valid,
        }
    }

    pub(super) fn with_start(self, start: Option<CalendarDate>) -> Self {
        Self {
            start,
            fields: field_validity(self.fields, true, true),
            ..self
        }
    }

    pub(super) fn with_end(self, end: Option<CalendarDate>) -> Self {
        Self {
            end,
            fields: field_validity(self.fields, false, true),
            ..self
        }
    }

    pub(super) fn start_valid(self, valid: bool) -> Self {
        Self {
            fields: field_validity(self.fields, true, valid),
            ..self
        }
    }

    pub(super) fn end_valid(self, valid: bool) -> Self {
        Self {
            fields: field_validity(self.fields, false, valid),
            ..self
        }
    }

    pub(super) fn order_invalid(self) -> bool {
        self.fields == FieldValidity::Valid
            && matches!((self.start, self.end), (Some(start), Some(end)) if start > end)
    }

    /// The range these fields stand for, `None` when they stand for none.
    pub(super) fn candidate(self) -> Option<Option<DateRange>> {
        if self.fields != FieldValidity::Valid {
            return None;
        }
        match (self.start, self.end) {
            (None, None) => Some(None),
            (Some(start), Some(end)) if start <= end => Some(Some(DateRange::new(start, end))),
            _ => None,
        }
    }
}

fn field_validity(state: FieldValidity, start: bool, valid: bool) -> FieldValidity {
    use FieldValidity::{BothInvalid, EndInvalid, StartInvalid, Valid};
    match (state, start, valid) {
        (Valid | StartInvalid, true, true) | (Valid | EndInvalid, false, true) => Valid,
        (EndInvalid | BothInvalid, true, true) => EndInvalid,
        (StartInvalid | BothInvalid, false, true) => StartInvalid,
        (Valid | StartInvalid, true, false) => StartInvalid,
        (Valid | EndInvalid, false, false) => EndInvalid,
        (_, _, false) => BothInvalid,
    }
}
