//! Role: A locale (a BCP 47 language tag) and the regional conventions it carries.

/// A day of the week ([`Locale::first_day_of_week`]).
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
        Self::ORDER[(self.index() + days) % 7]
    }

    /// How many days after `first` this day comes, 0 to 6: its column in a
    /// week that starts on `first`.
    pub fn days_since(self, first: Weekday) -> usize {
        (self.index() + 7 - first.index()) % 7
    }
}

/// The language and region an App speaks, as a BCP 47 language tag such as
/// `"ja-JP"` or `"en-GB"`. It selects nothing by itself: components read
/// it ([`locale`](super::locale)) to format what they show, and the App
/// uses it to choose its own translations.
///
/// The tag is kept as given. Subtags may be separated by `-` or `_`.
///
/// ```
/// use kivact::i18n::{Locale, Weekday};
///
/// let locale = Locale::new("ja-JP");
/// assert_eq!(locale.tag(), "ja-JP");
/// assert_eq!(locale.language(), "ja");
/// assert_eq!(locale.first_day_of_week(), Weekday::Sunday);
/// assert_eq!(Locale::new("de-DE").first_day_of_week(), Weekday::Monday);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locale {
    tag: String,
}

impl Locale {
    /// A locale named by the BCP 47 tag `tag`.
    pub fn new(tag: impl Into<String>) -> Self {
        Locale { tag: tag.into() }
    }

    /// The tag as given, e.g. `"ja-JP"`.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The language subtag, e.g. `"ja"` for `"ja-JP"`.
    pub fn language(&self) -> &str {
        self.tag.split(['-', '_']).next().unwrap_or_default()
    }

    pub(crate) fn icu(&self) -> icu_locale_core::Locale {
        self.tag
            .replace('_', "-")
            .parse()
            .unwrap_or(icu_locale_core::Locale::UNKNOWN)
    }

    /// The locale's first weekday, including Unicode locale overrides.
    pub fn first_day_of_week(&self) -> Weekday {
        let info = icu_calendar::week::WeekInformation::try_new(self.icu().into())
            .expect("invariant: compiled ICU week data includes locale fallback");
        Weekday::Monday.days_after((info.first_weekday as usize + 6) % 7)
    }
}
impl Default for Locale {
    /// `"en-US"`: what an App speaks when it names no locale and the
    /// system names none either.
    fn default() -> Self {
        Locale::new("en-US")
    }
}

kivact_runtime::constant_value!(Locale, Weekday);
