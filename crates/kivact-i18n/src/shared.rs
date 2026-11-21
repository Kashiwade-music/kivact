//! Role: Shared value types and number symbols; refers to no other module of this crate.

use icu_decimal::DecimalFormatter;
use icu_decimal::provider::{Baked, DecimalDigitsV1, DecimalSymbolsV1};
use icu_provider::prelude::{DataError, DataPayload, DataProvider, DataRequest, DataResponse};
use std::cell::RefCell;

/// A day of the week ([`Locale::first_day_of_week`](crate::Locale::first_day_of_week)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Weekday {
    /// Monday.
    Monday,
    /// Tuesday.
    Tuesday,
    /// Wednesday.
    Wednesday,
    /// Thursday.
    Thursday,
    /// Friday.
    Friday,
    /// Saturday.
    Saturday,
    /// Sunday.
    Sunday,
}

impl Weekday {
    const ORDER: [Weekday; 7] = [
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
        Weekday::Sunday,
    ];

    fn index(self) -> usize {
        Self::ORDER.iter().position(|day| *day == self).unwrap_or(0)
    }

    /// The day `days` after this one, wrapping around the week.
    pub fn days_after(self, days: usize) -> Weekday {
        Self::ORDER[(self.index() + days % 7) % 7]
    }

    /// How many days after `first` this day comes, 0 to 6: its column in a
    /// week that starts on `first`.
    pub fn days_since(self, first: Weekday) -> usize {
        (self.index() + 7 - first.index()) % 7
    }
}

/// The locale's default hour cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HourCycle {
    /// Twelve-hour clock.
    H12,
    /// Twenty-four-hour clock starting at zero.
    H23,
}

/// Width of a stand-alone calendar name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameWidth {
    /// Narrow name, which may be ambiguous.
    Narrow,
    /// Abbreviated name.
    Short,
    /// Full name.
    Long,
}

/// Symbols are resolved by ICU's formatter, including numbering-system overrides.
pub(crate) struct NumberSymbols {
    pub(crate) digits: [String; 10],
    decimal: String,
    group: String,
    minus_prefix: String,
    minus_suffix: String,
    plus_prefix: String,
    plus_suffix: String,
}

impl NumberSymbols {
    pub(crate) fn new(locale: &icu_locale_core::Locale) -> Self {
        let provider = SymbolCapture::default();
        DecimalFormatter::try_new_unstable(&provider, locale.clone().into(), Default::default())
            .expect("invariant: compiled ICU decimal data includes locale fallback");
        let symbols = provider
            .symbols
            .into_inner()
            .expect("invariant: successful formatter construction loaded symbols");
        let digits = provider
            .digits
            .into_inner()
            .expect("invariant: successful formatter construction loaded digits");
        let symbols = symbols.get();
        Self {
            digits: digits.get().map(|digit| digit.to_string()),
            decimal: symbols.decimal_separator().to_owned(),
            group: symbols.grouping_separator().to_owned(),
            minus_prefix: symbols.minus_sign_prefix().to_owned(),
            minus_suffix: symbols.minus_sign_suffix().to_owned(),
            plus_prefix: symbols.plus_sign_prefix().to_owned(),
            plus_suffix: symbols.plus_sign_suffix().to_owned(),
        }
    }

    pub(crate) fn parse(&self, text: &str) -> Option<f64> {
        let mut rest = text.trim();
        let negative = (!self.minus_prefix.is_empty() || !self.minus_suffix.is_empty())
            && rest.starts_with(&self.minus_prefix)
            && rest.ends_with(&self.minus_suffix);
        if negative {
            rest = rest
                .strip_prefix(&self.minus_prefix)?
                .strip_suffix(&self.minus_suffix)?;
        } else if (!self.plus_prefix.is_empty() || !self.plus_suffix.is_empty())
            && rest.starts_with(&self.plus_prefix)
            && rest.ends_with(&self.plus_suffix)
        {
            rest = rest
                .strip_prefix(&self.plus_prefix)?
                .strip_suffix(&self.plus_suffix)?;
        }
        let mut normalized = if negative {
            "-".to_owned()
        } else {
            String::new()
        };
        let mut fraction = false;
        let mut previous_digit = false;
        let mut any_digit = false;
        while !rest.is_empty() {
            if let Some((digit, glyph)) = self
                .digits
                .iter()
                .enumerate()
                .find(|(_, glyph)| rest.starts_with(glyph.as_str()))
            {
                normalized.push(char::from(b'0' + digit as u8));
                rest = &rest[glyph.len()..];
                previous_digit = true;
                any_digit = true;
            } else if !fraction && rest.starts_with(&self.decimal) {
                normalized.push('.');
                rest = &rest[self.decimal.len()..];
                fraction = true;
                previous_digit = false;
            } else if !fraction
                && !self.group.is_empty()
                && previous_digit
                && rest.starts_with(&self.group)
            {
                rest = &rest[self.group.len()..];
                if !self
                    .digits
                    .iter()
                    .any(|glyph| rest.starts_with(glyph.as_str()))
                {
                    return None;
                }
                previous_digit = false;
            } else {
                return None;
            }
        }
        let result: f64 = normalized.parse().ok()?;
        (any_digit && result.is_finite()).then_some(result)
    }
}

// xtask-allow(state): independent: symbols and digits are filled by the two DataProvider::load impls, one per ICU key, in whichever order ICU asks; NumberSymbols::new takes both once the formatter is built
/// Observe ICU's resolved data requests, preserving locale and numbering-system fallback.
#[derive(Default)]
struct SymbolCapture {
    symbols: RefCell<Option<DataPayload<DecimalSymbolsV1>>>,
    digits: RefCell<Option<DataPayload<DecimalDigitsV1>>>,
}

impl DataProvider<DecimalSymbolsV1> for SymbolCapture {
    fn load(&self, request: DataRequest<'_>) -> Result<DataResponse<DecimalSymbolsV1>, DataError> {
        let response = Baked.load(request)?;
        self.symbols.replace(Some(response.payload.clone()));
        Ok(response)
    }
}

impl DataProvider<DecimalDigitsV1> for SymbolCapture {
    fn load(&self, request: DataRequest<'_>) -> Result<DataResponse<DecimalDigitsV1>, DataError> {
        let response = Baked.load(request)?;
        self.digits.replace(Some(response.payload.clone()));
        Ok(response)
    }
}

kivact_runtime::constant_value!(Weekday, HourCycle, NameWidth);
