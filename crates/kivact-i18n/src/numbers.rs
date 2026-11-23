//! Role: Format and parse decimal numbers using ICU locale symbols and digits.

use crate::shared::NumberSymbols;
use crate::{FractionDigits, Grouping, Locale, NumberStyle};
use icu_decimal::DecimalFormatter;
use icu_decimal::input::Decimal;
use icu_decimal::options::{DecimalFormatterOptions, GroupingStrategy};

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
        let cache = self.cache();
        let index = match style.grouping {
            Grouping::None => 0,
            Grouping::Locale => 1,
        };
        let formatter = cache.decimals[index].get_or_init(|| {
            let mut options = DecimalFormatterOptions::default();
            options.grouping_strategy = Some(match style.grouping {
                Grouping::None => GroupingStrategy::Never,
                Grouping::Locale => GroupingStrategy::Auto,
            });
            DecimalFormatter::try_new(cache.icu().clone().into(), options)
                .expect("invariant: compiled ICU decimal data includes locale fallback")
        });
        formatter.format(&decimal).to_string()
    }

    /// Parse a finite decimal using the locale's symbols and numbering system.
    pub fn parse_number(&self, text: &str) -> Option<f64> {
        let cache = self.cache();
        cache
            .symbols
            .get_or_init(|| NumberSymbols::new(cache.icu()))
            .parse(text)
    }
}

/// Extract numeric date fields with the same resolved digits as number parsing.
pub(crate) fn date_fields(locale: &Locale, text: &str, pattern: &str) -> Option<Vec<String>> {
    let cache = locale.cache();
    let symbols = cache
        .symbols
        .get_or_init(|| NumberSymbols::new(cache.icu()));
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
