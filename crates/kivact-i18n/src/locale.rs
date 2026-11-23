//! Role: A locale (a BCP 47 language tag) and the regional conventions it carries.

use crate::shared::Weekday;

/// The language and region an App speaks, as a BCP 47 language tag such as
/// `"ja-JP"` or `"en-GB"`. It selects nothing by itself: components read
/// it (`kivact::i18n::use_locale`) to format what they show, and the App
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
    tag: std::sync::Arc<str>,
}

impl Locale {
    /// A locale named by the BCP 47 tag `tag`.
    pub fn new(tag: impl Into<String>) -> Self {
        Locale {
            tag: std::sync::Arc::from(tag.into()),
        }
    }

    /// The tag as given, e.g. `"ja-JP"`.
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The language subtag, e.g. `"ja"` for `"ja-JP"`.
    pub fn language(&self) -> &str {
        self.tag.split(['-', '_']).next().unwrap_or_default()
    }

    /// This locale's ICU objects on this thread.
    pub(crate) fn cache(&self) -> std::rc::Rc<crate::cache::LocaleCache> {
        crate::cache::locale_cache(&self.tag)
    }

    /// The parsed ICU locale.
    pub(crate) fn icu(&self) -> icu_locale_core::Locale {
        self.cache().icu().clone()
    }

    /// The locale's first weekday, including Unicode locale overrides.
    pub fn first_day_of_week(&self) -> Weekday {
        let cache = self.cache();
        *cache.first_day.get_or_init(|| {
            let info = icu_calendar::week::WeekInformation::try_new(cache.icu().clone().into())
                .expect("invariant: compiled ICU week data includes locale fallback");
            Weekday::Monday.days_after((info.first_weekday as usize + 6) % 7)
        })
    }
}
impl Default for Locale {
    /// `"en-US"`: what an App speaks when it names no locale and the
    /// system names none either.
    fn default() -> Self {
        Locale::new("en-US")
    }
}

kivact_runtime::constant_value!(Locale);
