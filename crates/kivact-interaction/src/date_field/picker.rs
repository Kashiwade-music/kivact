//! Role: The date picker state: open or closed, its panel and where focus returns.

use kivact_core::state::{IntoValue, Signal, Value, binding, memo, signal};
use kivact_core::view::NodeRef;

use super::types::{DatePanel, DatePicker};

#[derive(Clone, Copy)]
pub struct DatePickerApi {
    state: Signal<DatePicker>,
    open: Value<bool>,
}

/// A picker's open state, its panel and where the focus returns. Setup only.
#[track_caller]
pub fn use_date_picker() -> DatePickerApi {
    let state = signal(DatePicker::Closed);
    DatePickerApi {
        state,
        open: memo(move || matches!(state.get(), DatePicker::Open { .. })),
    }
}

impl DatePickerApi {
    pub fn show(self, panel: DatePanel, invoker: NodeRef) {
        self.state.set(DatePicker::Open { panel, invoker });
    }

    /// Show `panel` while open; nothing while closed.
    pub fn show_panel(self, panel: DatePanel) {
        if let DatePicker::Open { invoker, .. } = self.state.get_untracked() {
            self.state.set(DatePicker::Open { panel, invoker });
        }
    }

    pub fn hide(self) {
        self.state.set(DatePicker::Closed);
    }

    pub fn state(self) -> Value<DatePicker> {
        self.state.into_value()
    }

    pub fn is_open(self) -> Value<bool> {
        self.open
    }

    /// The open state as a signal: opening shows `panel` from `invoker`. Setup only.
    #[track_caller]
    pub fn open(self, panel: DatePanel, invoker: NodeRef) -> Signal<bool> {
        binding(self.open, move |open| {
            if open {
                self.show(panel, invoker);
            } else {
                self.hide();
            }
        })
    }

    /// Where the focus returns on close: the invoker, else `fallback`. Setup only.
    #[track_caller]
    pub fn return_focus(self, fallback: NodeRef) -> Value<NodeRef> {
        let state = self.state;
        memo(move || match state.get() {
            DatePicker::Open { invoker, .. } => invoker,
            DatePicker::Closed => fallback,
        })
    }
}
