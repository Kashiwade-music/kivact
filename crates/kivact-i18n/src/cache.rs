//! Role: Per-thread caches of each locale's ICU objects, built on first use.
//!
//! A `Locale` is a tag; what ICU needs to format for it (the parsed
//! locale, the formatters, the collator, the plural rules, the names) is
//! built the first time it is asked for and kept in a per-thread cache of
//! the last `CACHED_LOCALES` tags. ICU objects are not shared between
//! threads, so a `Locale` stays `Send + Sync`.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::sync::Arc;

use icu_calendar::Gregorian;
use icu_datetime::fieldsets::{T, YMD};
use icu_datetime::{FixedCalendarDateTimeFormatter, NoCalendarFormatter};
use icu_decimal::DecimalFormatter;
use kivact_types::cell::Deferred;

use crate::shared::{HourCycle, NameWidth, NumberSymbols, Weekday};

/// How many locales one thread keeps (an App uses one or two).
const CACHED_LOCALES: usize = 16;

/// A display name kept by `LocaleCache::names`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum NameKey {
    Month(NameWidth, u8),
    Weekday(NameWidth, Weekday),
}

/// The string comparison of a locale's collation.
type Collate = Box<dyn Fn(&str, &str) -> Ordering>;

/// The cardinal plural category of a number in a locale.
type Plural = Box<dyn Fn(&icu_decimal::input::Decimal) -> icu_plurals::PluralCategory>;

/// The ICU objects of one locale, each built on first use.
pub(crate) struct LocaleCache {
    pub(crate) tag: Arc<str>,
    icu: Deferred<icu_locale_core::Locale>,
    pub(crate) first_day: Deferred<Weekday>,
    pub(crate) hour_cycle: Deferred<HourCycle>,
    pub(crate) collator: Deferred<Collate>,
    pub(crate) plurals: Deferred<Plural>,
    pub(crate) symbols: Deferred<NumberSymbols>,
    /// Decimal formatters by grouping: `[Grouping::None, Grouping::Locale]`.
    pub(crate) decimals: [Deferred<DecimalFormatter>; 2],
    /// Date formatters by length: `[short, medium, long]`.
    pub(crate) dates: [Deferred<FixedCalendarDateTimeFormatter<Gregorian, YMD>>; 3],
    /// Time formatters: `[minutes, seconds]`.
    pub(crate) times: [Deferred<NoCalendarFormatter<T>>; 2],
    /// Month and weekday names already formatted.
    names: RefCell<Vec<(NameKey, String)>>,
}

impl LocaleCache {
    fn new(tag: Arc<str>) -> Self {
        LocaleCache {
            tag,
            icu: Deferred::new(),
            first_day: Deferred::new(),
            hour_cycle: Deferred::new(),
            collator: Deferred::new(),
            plurals: Deferred::new(),
            symbols: Deferred::new(),
            decimals: [Deferred::new(), Deferred::new()],
            dates: [Deferred::new(), Deferred::new(), Deferred::new()],
            times: [Deferred::new(), Deferred::new()],
            names: RefCell::new(Vec::new()),
        }
    }

    /// The parsed locale (a tag ICU cannot parse is `UNKNOWN`).
    pub(crate) fn icu(&self) -> &icu_locale_core::Locale {
        self.icu.get_or_init(|| {
            self.tag
                .replace('_', "-")
                .parse()
                .unwrap_or(icu_locale_core::Locale::UNKNOWN)
        })
    }

    /// The name `key` formats to, computed by `make` the first time.
    pub(crate) fn name(&self, key: NameKey, make: impl FnOnce() -> String) -> String {
        if let Some((_, name)) = self.names.borrow().iter().find(|(k, _)| *k == key) {
            return name.clone();
        }
        let name = make();
        self.names.borrow_mut().push((key, name.clone()));
        name
    }
}

thread_local! {
    static CACHES: RefCell<Vec<Rc<LocaleCache>>> = const { RefCell::new(Vec::new()) };
}

/// The cache of `tag` on this thread; the oldest of `CACHED_LOCALES` is
/// dropped when a new tag arrives.
pub(crate) fn locale_cache(tag: &Arc<str>) -> Rc<LocaleCache> {
    CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        if let Some(found) = caches.iter().find(|c| *c.tag == **tag) {
            return Rc::clone(found);
        }
        if caches.len() == CACHED_LOCALES {
            caches.remove(0);
        }
        let cache = Rc::new(LocaleCache::new(Arc::clone(tag)));
        caches.push(Rc::clone(&cache));
        cache
    })
}

#[cfg(test)]
mod tests {
    //! Role: One locale's ICU objects are built once per thread.

    use std::rc::Rc;
    use std::sync::Arc;

    #[test]
    fn the_same_tag_reuses_one_cache() {
        let a = super::locale_cache(&Arc::from("ja-JP"));
        let b = super::locale_cache(&Arc::from("ja-JP"));
        assert!(Rc::ptr_eq(&a, &b));
    }
}
