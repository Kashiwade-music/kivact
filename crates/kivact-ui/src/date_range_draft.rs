//! Role: Own typed range endpoint candidates while the public binding stays ordered.
use kivact_core::i18n::{CalendarDate, DateRange};
use kivact_core::state::{Machine, Step};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub(super) enum FieldValidity {
    Valid,
    StartInvalid,
    EndInvalid,
    BothInvalid,
}
#[derive(Clone, Copy, PartialEq)]
pub(super) enum RangeDraft {
    Following(Option<DateRange>),
    Editing {
        // xtask-allow(state-fields): independent fact - the typed start endpoint, absent while that field is empty
        start: Option<CalendarDate>,
        // xtask-allow(state-fields): independent fact - the typed end endpoint, absent while that field is empty
        end: Option<CalendarDate>,
        fields: FieldValidity,
    },
}
impl RangeDraft {
    pub(super) fn endpoints(self) -> (Option<CalendarDate>, Option<CalendarDate>) {
        match self {
            Self::Following(range) => (range.map(DateRange::start), range.map(DateRange::end)),
            Self::Editing { start, end, .. } => (start, end),
        }
    }
    pub(super) fn fields(self) -> FieldValidity {
        match self {
            Self::Following(_) => FieldValidity::Valid,
            Self::Editing { fields, .. } => fields,
        }
    }
    pub(super) fn order_invalid(self) -> bool {
        let (start, end) = self.endpoints();
        self.fields() == FieldValidity::Valid
            && matches!((start,end),(Some(start),Some(end)) if start>end)
    }
    fn candidate(self) -> Option<Option<DateRange>> {
        if self.fields() != FieldValidity::Valid {
            return None;
        }
        match self.endpoints() {
            (None, None) => Some(None),
            (Some(start), Some(end)) if start <= end => Some(Some(DateRange::new(start, end))),
            _ => None,
        }
    }
}
pub(super) enum Event {
    External(Option<DateRange>),
    Start(Option<CalendarDate>),
    End(Option<CalendarDate>),
    StartValid(bool),
    EndValid(bool),
}
pub(super) enum Effect {
    Publish(Option<DateRange>),
    Validity(bool),
}
pub(super) struct Policy {
    pub(super) accept: Rc<dyn Fn(DateRange) -> bool>,
}
impl Machine for Policy {
    type State = RangeDraft;
    type Event = Event;
    type Effect = Effect;
    fn transition(&self, state: &RangeDraft, event: Event) -> Step<RangeDraft, Event, Effect> {
        let (mut start, mut end) = state.endpoints();
        let mut fields = state.fields();
        let commit = match event {
            Event::External(value) => {
                return Step::to(RangeDraft::Following(value)).with(Effect::Validity(
                    value.is_none_or(|range| (self.accept)(range)),
                ));
            }
            Event::Start(value) => {
                fields = field_validity(fields, true, true);
                start = value;
                true
            }
            Event::End(value) => {
                fields = field_validity(fields, false, true);
                end = value;
                true
            }
            Event::StartValid(valid) => {
                fields = field_validity(fields, true, valid);
                false
            }
            Event::EndValid(valid) => {
                fields = field_validity(fields, false, valid);
                false
            }
        };
        let next = RangeDraft::Editing { start, end, fields };
        let accepted = next
            .candidate()
            .filter(|candidate| candidate.is_none_or(|range| (self.accept)(range)));
        let step = Step::to(next).with(Effect::Validity(accepted.is_some()));
        if commit && let Some(value) = accepted {
            return step.with(Effect::Publish(value));
        }
        step
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
