//! Role: Semantic actions shared by Arrangement and range controls.
use kivact::custom::{Part, PartState, part};
use kivact::draw::RegionId;
use kivact::state::{Callback, Value};
use kivact::view::{A11yAction, A11yEvent, Role};
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Kind {
    Item,
    Trim,
    TrackSwitch,
    Automation,
    LoopEndpoint,
    AddTrack,
    LoopBody,
    LoopCreate,
    ArrangementPlayhead,
    RangeBody,
}
#[derive(Clone, PartialEq)]
pub(crate) struct RegionInfo {
    pub(crate) id: RegionId,
    pub(crate) kind: Kind,
    pub(crate) name: String,
    pub(crate) value: String,
    // xtask-allow(state-fields): Independent accessibility output facts, recomputed from domain data.
    pub(crate) checked: bool,
    // xtask-allow(state-fields): Independent accessibility output facts, recomputed from domain data.
    pub(crate) selected: bool,
    pub(crate) min: f64,
    pub(crate) max: f64,
    pub(crate) now: f64,
}

impl RegionInfo {
    pub(crate) fn new(id: RegionId, kind: Kind, name: String) -> Self {
        Self {
            id,
            kind,
            name,
            value: String::new(),
            checked: false,
            selected: false,
            min: 0.0,
            max: 0.0,
            now: 0.0,
        }
    }
}

pub(crate) fn attachment(
    kind: Kind,
    data: Value<RegionInfo>,
    action: Callback<A11yEvent>,
) -> Part<PartState> {
    let role = match kind {
        Kind::LoopCreate => Role::Group,
        Kind::Automation | Kind::LoopEndpoint | Kind::ArrangementPlayhead => Role::Slider,
        _ => Role::Button,
    };
    let mut result = {
        let input_part = part(|input| input);
        kivact::custom::Part::new(input_part.state(), move |node| {
            node.part(input_part)
                .role(role)
                .name(move || data.get().name)
        })
    };
    if kind == Kind::Item {
        result = kivact::custom::Part::new(result.state(), move |node| {
            node.part(result).value_text(move || data.get().value)
        });
    }
    if kind == Kind::TrackSwitch {
        result = kivact::custom::Part::new(result.state(), move |node| {
            node.part(result).checked(move || data.get().checked)
        });
    }
    if matches!(kind, Kind::Automation | Kind::LoopEndpoint) {
        result = kivact::custom::Part::new(result.state(), move |node| {
            node.part(result)
                .value_min(move || data.get().min)
                .value_max(move || data.get().max)
        });
    }
    if matches!(
        kind,
        Kind::Automation | Kind::LoopEndpoint | Kind::ArrangementPlayhead
    ) {
        result = kivact::custom::Part::new(result.state(), move |node| {
            node.part(result).value_now(move || data.get().now)
        });
    }
    let actions: &[A11yAction] = match kind {
        Kind::LoopCreate => &[],
        Kind::Item => &[A11yAction::Select, A11yAction::Activate],
        Kind::Automation => &[A11yAction::SetValue, A11yAction::Select],
        Kind::LoopEndpoint | Kind::ArrangementPlayhead => &[A11yAction::SetValue],
        Kind::RangeBody => &[A11yAction::Select],
        _ => &[A11yAction::Activate],
    };
    for &advertised in actions {
        result = kivact::custom::Part::new(result.state(), move |node| {
            node.part(result)
                .a11y_action(advertised, move |event| action.call(event))
        });
    }
    result
}
pub(crate) fn bounded_range((a, b): (f64, f64), length: f64, min_duration: f64) -> (f64, f64) {
    let minimum = min_duration.min(length);
    let a = if a.is_finite() {
        a.clamp(0.0, (length - minimum).max(0.0))
    } else {
        0.0
    };
    let b = if b.is_finite() {
        b.clamp((a + minimum).min(length), length)
    } else {
        16.0_f64.max(a + minimum).min(length)
    };
    (a, b)
}
