//! Role: Format and parse decimal numbers using ICU locale symbols and digits.

use crate::{FractionDigits, Grouping, Locale, NumberStyle};
use icu_decimal::DecimalFormatter;
use icu_decimal::input::Decimal;
use icu_decimal::options::{DecimalFormatterOptions, GroupingStrategy};
use icu_decimal::provider::{Baked, DecimalDigitsV1, DecimalSymbolsV1};
use icu_provider::prelude::{DataError, DataPayload, DataProvider, DataRequest, DataResponse};
use std::cell::RefCell;

impl Locale {
    /// Round, pad and group a decimal using the locale's symbols and digits.
    pub fn format_number(&self, value: f64, style: NumberStyle) -> String {
        if !value.is_finite() {
            return value.to_string();
        }
        let (min, max) = match style.fraction_digits {
            FractionDigits::Exact(n) => (n, n),
            FractionDigits::Range { min, max } => (min.min(max), max),
        };
        let mut decimal: Decimal = value
            .to_string()
            .parse()
            .expect("invariant: a finite float has a decimal representation");
        decimal.round(-i16::from(max));
        decimal.trim_end();
        decimal.pad_end(-i16::from(min));
        let mut options = DecimalFormatterOptions::default();
        options.grouping_strategy = Some(match style.grouping {
            Grouping::None => GroupingStrategy::Never,
            Grouping::Locale => GroupingStrategy::Auto,
        });
        self.decimal_formatter(options).format(&decimal).to_string()
    }

    /// Parse a finite decimal using the locale's symbols and numbering system.
    pub fn parse_number(&self, text: &str) -> Option<f64> {
        let symbols = NumberSymbols::new(self);
        symbols.parse(text)
    }

    fn decimal_formatter(&self, options: DecimalFormatterOptions) -> DecimalFormatter {
        DecimalFormatter::try_new(self.icu().into(), options)
            .expect("invariant: compiled ICU decimal data includes locale fallback")
    }
}

/// Symbols are resolved by ICU's formatter, including numbering-system overrides.
struct NumberSymbols {
    digits: [String; 10],
    decimal: String,
    group: String,
    minus_prefix: String,
    minus_suffix: String,
    plus_prefix: String,
    plus_suffix: String,
}

impl NumberSymbols {
    fn new(locale: &Locale) -> Self {
        let provider = SymbolCapture::default();
        DecimalFormatter::try_new_unstable(&provider, locale.icu().into(), Default::default())
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

    fn parse(&self, text: &str) -> Option<f64> {
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

/// Observe ICU's resolved data requests, preserving locale and numbering-system fallback.
#[derive(Default)]
struct SymbolCapture {
    // xtask-allow(state-fields): cache: the provider response, captured once when the formatter is built
    symbols: RefCell<Option<DataPayload<DecimalSymbolsV1>>>,
    // xtask-allow(state-fields): cache: the provider response, captured once when the formatter is built
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

/// Extract numeric date fields with the same resolved digits as number parsing.
pub(crate) fn date_fields(locale: &Locale, text: &str, pattern: &str) -> Option<Vec<String>> {
    let symbols = NumberSymbols::new(locale);
    let mut fields = Vec::new();
    let mut number = String::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some((digit, glyph)) = symbols
            .digits
            .iter()
            .enumerate()
            .find(|(_, glyph)| rest.starts_with(glyph.as_str()))
        {
            number.push(char::from(b'0' + digit as u8));
            rest = &rest[glyph.len()..];
        } else {
            let separator = rest.chars().next()?;
            if separator.is_numeric() || (separator.is_alphabetic() && !pattern.contains(separator))
            {
                return None;
            }
            if !number.is_empty() {
                fields.push(std::mem::take(&mut number));
            }
            rest = &rest[separator.len_utf8()..];
        }
    }
    if !number.is_empty() {
        fields.push(number);
    }
    Some(fields)
}
