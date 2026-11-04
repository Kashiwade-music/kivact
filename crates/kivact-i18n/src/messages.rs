//! Role: The texts components show, by key.

use crate::{PluralCategory, PluralForms};
use std::collections::BTreeMap;

/// The texts components show, by key: the App's replacements for the
/// default text a component carries (`"dialog.close"` is the label of a
/// dialog's close button).
///
/// A component names the keys it reads in its documentation. A key the
/// messages do not hold keeps the component's own default, so a
/// `Messages` lists only what the App changes.
///
/// ```
/// use kivact::i18n::Messages;
///
/// let ja = Messages::new()
///     .with("dialog.close", "閉じる")
///     .with("table.label", "データテーブル");
/// assert_eq!(ja.get("dialog.close"), Some("閉じる"));
/// assert_eq!(ja.get("palette.label"), None);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Messages {
    texts: BTreeMap<&'static str, MessageText>,
}

impl Messages {
    /// No texts: every component keeps its default.
    pub fn new() -> Self {
        Messages::default()
    }

    /// These messages with `key` set to `text` (replacing an earlier
    /// text of the same key).
    pub fn with(mut self, key: &'static str, text: impl Into<String>) -> Self {
        self.texts.insert(key, MessageText::Plain(text.into()));
        self
    }

    /// Set the cardinal plural forms for a message key.
    pub fn with_plural(mut self, key: &'static str, forms: PluralForms) -> Self {
        self.texts.insert(key, MessageText::Plural(forms));
        self
    }

    /// Read the requested plural form, falling back to `other`.
    pub fn get_plural(&self, key: &str, category: PluralCategory) -> Option<&str> {
        let MessageText::Plural(forms) = self.texts.get(key)? else {
            return None;
        };
        let text = match category {
            PluralCategory::Zero => &forms.zero,
            PluralCategory::One => &forms.one,
            PluralCategory::Two => &forms.two,
            PluralCategory::Few => &forms.few,
            PluralCategory::Many => &forms.many,
            PluralCategory::Other => return Some(&forms.other),
        };
        Some(text.as_deref().unwrap_or(&forms.other))
    }
    /// The text of `key`, if these messages hold one.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.texts.get(key).map(|text| match text {
            MessageText::Plain(text) => text.as_str(),
            MessageText::Plural(forms) => forms.other.as_str(),
        })
    }

    /// Whether these messages hold no text.
    pub fn is_empty(&self) -> bool {
        self.texts.is_empty()
    }

    /// These messages with every text of `over` set on top: a key `over`
    /// holds wins, every other key keeps the text of `self`.
    ///
    /// ```
    /// use kivact::i18n::Messages;
    ///
    /// let outer = Messages::new().with("a", "outer a").with("b", "outer b");
    /// let inner = Messages::new().with("b", "inner b");
    /// let layered = outer.layered(&inner);
    /// assert_eq!(layered.get("a"), Some("outer a"));
    /// assert_eq!(layered.get("b"), Some("inner b"));
    /// ```
    pub fn layered(&self, over: &Messages) -> Messages {
        let mut texts = self.texts.clone();
        texts.extend(over.texts.iter().map(|(k, v)| (*k, v.clone())));
        Messages { texts }
    }
}

kivact_runtime::constant_value!(Messages);

#[derive(Clone, Debug, PartialEq, Eq)]
enum MessageText {
    Plain(String),
    Plural(PluralForms),
}
