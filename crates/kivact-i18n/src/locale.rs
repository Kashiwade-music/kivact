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

    /// The region subtag (`"JP"` of `"ja-JP"`, `"419"` of `"es-419"`), if
    /// the tag names one.
    fn region(&self) -> Option<&str> {
        // Script (four letters), region (two letters or three digits),
        // variants and extensions follow the language in this order; a
        // single-character subtag starts an extension, which has no region.
        self.tag
            .split(['-', '_'])
            .skip(1)
            .take_while(|subtag| subtag.len() > 1)
            .find(|subtag| {
                (subtag.len() == 2 && subtag.bytes().all(|b| b.is_ascii_alphabetic()))
                    || (subtag.len() == 3 && subtag.bytes().all(|b| b.is_ascii_digit()))
            })
    }

    /// The day a calendar week starts on here (a calendar's first
    /// column).
    ///
    /// It follows the Unicode CLDR's week data by region: Sunday in
    /// Japan, the United States, Korea, India, Brazil and others;
    /// Saturday in much of the Middle East; Friday in the Maldives;
    /// Monday elsewhere, which covers Europe and every region this table
    /// does not list. A tag without a region uses its language's most
    /// likely one (`"en"` as `"en-US"`, `"ja"` as `"ja-JP"`).
    ///
    /// ```
    /// use kivact::i18n::{Locale, Weekday};
    ///
    /// assert_eq!(Locale::new("en-US").first_day_of_week(), Weekday::Sunday);
    /// assert_eq!(Locale::new("en-GB").first_day_of_week(), Weekday::Monday);
    /// assert_eq!(Locale::new("en").first_day_of_week(), Weekday::Sunday);
    /// assert_eq!(Locale::new("ar-EG").first_day_of_week(), Weekday::Saturday);
    /// ```
    pub fn first_day_of_week(&self) -> Weekday {
        let region = self
            .region()
            .map(str::to_ascii_uppercase)
            .or_else(|| likely_region(&self.language().to_ascii_lowercase()).map(str::to_owned));
        match region.as_deref() {
            Some(region) if SUNDAY_START.contains(&region) => Weekday::Sunday,
            Some(region) if SATURDAY_START.contains(&region) => Weekday::Saturday,
            Some("MV") => Weekday::Friday,
            _ => Weekday::Monday,
        }
    }
}

impl Default for Locale {
    /// `"en-US"`: what an App speaks when it names no locale and the
    /// system names none either.
    fn default() -> Self {
        Locale::new("en-US")
    }
}

kivact_reactive::constant_value!(Locale, Weekday);

/// Regions whose weeks start on Sunday (CLDR `weekData`, `firstDay`).
const SUNDAY_START: &[&str] = &[
    "AG", "AS", "AU", "BD", "BR", "BS", "BT", "BW", "BZ", "CA", "CO", "DM", "DO", "ET", "GT", "GU",
    "HK", "HN", "ID", "IL", "IN", "JM", "JP", "KE", "KH", "KR", "LA", "MH", "MM", "MO", "MT", "MX",
    "MZ", "NI", "NP", "PA", "PE", "PH", "PK", "PR", "PT", "PY", "SA", "SG", "SV", "TH", "TT", "TW",
    "UM", "US", "VE", "VI", "WS", "YE", "ZA", "ZW",
];

/// Regions whose weeks start on Saturday.
const SATURDAY_START: &[&str] = &[
    "AE", "AF", "BH", "DJ", "DZ", "EG", "IQ", "IR", "JO", "KW", "LY", "OM", "QA", "SD", "SY",
];

/// The region a language is mostly spoken in, for the languages whose week
/// does not start on Monday there. Others need no entry.
fn likely_region(language: &str) -> Option<&'static str> {
    Some(match language {
        "en" => "US",
        "ja" => "JP",
        "ko" => "KR",
        "he" => "IL",
        "ar" => "EG",
        "fa" => "IR",
        "hi" | "ta" | "te" | "mr" => "IN",
        "th" => "TH",
        "id" => "ID",
        "pt" => "BR",
        "bn" => "BD",
        "ur" => "PK",
        "fil" => "PH",
        _ => return None,
    })
}
