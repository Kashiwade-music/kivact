//! Role: Describe typed table filter editors and validate their values.
use kivact_interaction::FilterValue;

/// Placement and density of a table's typed filter controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterUi {
    /// A header trigger opens a draft editor with Reset and Apply.
    #[default]
    Popover,
    /// A control under the heading applies changes as they are entered.
    Inline,
    /// A compact inline control.
    InlineCompact,
}

/// The editor and accepted values for a typed column filter.
#[derive(Clone, Debug, PartialEq)]
pub enum FilterField {
    /// Text, with a blank value clearing the filter.
    Text,
    /// A finite number with optional inclusive bounds.
    Number {
        // xtask-allow(state-fields): option: absent means the filter has no bound
        min: Option<f64>,
        // xtask-allow(state-fields): option: absent means the filter has no bound
        max: Option<f64>,
    },
    /// One of the (key, label) options.
    Choice(Vec<(String, String)>),
    /// Any of the (key, label) options.
    Choices(Vec<(String, String)>),
    /// A calendar date entered as YYYY-MM-DD.
    Date,
    /// A local time entered as HH:MM.
    Time,
    /// A list of removable text tokens.
    Tokens,
}

impl FilterField {
    pub(crate) fn accepts(&self, value: &FilterValue) -> bool {
        match (self, value) {
            (Self::Text, FilterValue::Text(_))
            | (Self::Date, FilterValue::Date(_))
            | (Self::Time, FilterValue::Time { .. })
            | (Self::Tokens, FilterValue::Tokens(_)) => !value.is_empty(),
            (Self::Number { min, max }, FilterValue::Number(number)) => {
                number.is_finite()
                    && min.is_none_or(|min| *number >= min)
                    && max.is_none_or(|max| *number <= max)
            }
            (Self::Choice(options), FilterValue::Choice(key)) => {
                options.iter().any(|(id, _)| id == key)
            }
            (Self::Choices(options), FilterValue::Choices(keys)) => {
                !keys.is_empty()
                    && keys
                        .iter()
                        .all(|key| options.iter().any(|(id, _)| id == key))
            }
            _ => false,
        }
    }
}
