//! Role: Name an icon by meaning or by its image, as plain data.

use std::borrow::Cow;

use kivact_runtime::{IntoValue, Prop as Input};

use crate::asset::Asset;

/// An icon as data: a name the theme's icon set resolves (`"check"`,
/// `"chevron-right"`), or an explicit image [`Asset`].
///
/// List items and menu entries carry one so that a component can draw it
/// beside the label without the app building a view per row. A name keeps
/// the choice of image with the theme, so a kit draws its own glyph for
/// `"check"`; an asset draws that image wherever it appears. Two icons are
/// equal when they have the same name or the same asset.
///
/// ```
/// use kivact::view::Asset;
/// use kivact::style::Icon;
/// let check = Icon::named("check");
/// assert_eq!(check.name(), Some("check"));
/// assert_eq!(Icon::from("check"), check);
/// static LOGO: &[u8] = b"<svg/>";
/// let logo = Icon::from(Asset::from(LOGO));
/// assert!(logo.asset().is_some());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Icon(InputProp);

#[derive(Clone, Debug, PartialEq)]
enum InputProp {
    Named(Cow<'static, str>),
    Asset(Asset),
}

impl Icon {
    /// An icon the theme's icon set resolves by `name`.
    pub fn named(name: impl Into<String>) -> Self {
        Icon(InputProp::Named(Cow::Owned(name.into())))
    }

    /// The name, for an icon made by [`named`](Self::named).
    pub fn name(&self) -> Option<&str> {
        match &self.0 {
            InputProp::Named(name) => Some(name),
            InputProp::Asset(_) => None,
        }
    }

    /// The image, for an icon made from an [`Asset`].
    pub fn asset(&self) -> Option<&Asset> {
        match &self.0 {
            InputProp::Named(_) => None,
            InputProp::Asset(asset) => Some(asset),
        }
    }
}

impl From<Asset> for Icon {
    /// An icon drawn from this image.
    fn from(asset: Asset) -> Self {
        Icon(InputProp::Asset(asset))
    }
}

impl From<&'static str> for Icon {
    /// An icon the theme resolves by this name (not a file path: use
    /// `Icon::from(Asset::from(path))` for an image file).
    fn from(name: &'static str) -> Self {
        Icon::named(name)
    }
}

kivact_runtime::constant_value!(Icon);

/// A constant icon from what [`Icon`] converts from: a name the theme
/// resolves (`icon("check")`) or an image [`Asset`]. The conversion happens
/// once, when the property is set.
macro_rules! icon_into_value {
    ($($t:ty),*) => {$(
        // Kept out of other properties' diagnostics: a name or an asset is
        // only an icon here.
        #[diagnostic::do_not_recommend]
        impl IntoValue<Icon> for $t {
            fn into_prop(self) -> Input<Icon> {
                Input::constant(Icon::from(self))
            }
        }
    )*};
}

icon_into_value!(&'static str, Asset);

/// The images a theme draws for [`Icon`] names, in
/// [`Theme::icons`](crate::Theme::icons).
///
/// A component that draws an icon (`kivact::view::icon`, the
/// marks and item icons of the standard components) looks its name up in the
/// theme in effect, so a component library's theme gives every name its own
/// glyph. The [`standard`](Self::standard) set has the marks the standard
/// components draw; a library's theme adds its names on top with
/// [`with`](Self::with) or [`Extend`]. Clones share the table; two sets are
/// equal when they map the same names to the same assets.
///
/// ```
/// use kivact::style::{Icon, IconSet};
/// static STAR: &[u8] = b"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'/>";
/// let icons = IconSet::standard().with("star", STAR);
/// assert!(icons.resolve(&Icon::named("star")).is_some());
/// assert!(icons.resolve(&Icon::named("check")).is_some());
/// assert!(icons.resolve(&Icon::named("unknown")).is_none());
/// ```
#[derive(Clone, Debug, Default)]
pub struct IconSet(std::sync::Arc<std::collections::BTreeMap<Cow<'static, str>, Asset>>);

impl PartialEq for IconSet {
    fn eq(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

/// The marks of the standard set, 24 px outlines drawn in the icon's color.
static ARROW_DOWN: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 5v14m-7-7 7 7 7-7"/></svg>"##;
static COPY: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>"##;
static CHECK: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12.5l4.5 4.5L19 7"/></svg>"##;
static DOT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4.5" fill="#000"/></svg>"##;
static MINUS: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round"><path d="M6 12h12"/></svg>"##;
static CHEVRON_DOWN: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 9l6 6 6-6"/></svg>"##;
static CHEVRON_RIGHT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 6l6 6-6 6"/></svg>"##;
static CHEVRON_LEFT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 6l-6 6 6 6"/></svg>"##;
static CHEVRONS_LEFT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2"><path d="M11 6l-6 6 6 6M19 6l-6 6 6 6"/></svg>"##;
static CHEVRONS_RIGHT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2"><path d="M5 6l6 6-6 6M13 6l6 6-6 6"/></svg>"##;
static CALENDAR: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="5" width="18" height="16" rx="2"/><path d="M16 3v4M8 3v4M3 11h18"/></svg>"##;
static PLUS: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>"##;
static TOAST_INFO: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8h.01"/></svg>"##;
static TOAST_SUCCESS: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M8 12.5l3 3 5-6"/></svg>"##;
static TOAST_WARNING: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 4l9 16H3z"/><path d="M12 10v4M12 17h.01"/></svg>"##;
static TOAST_DANGER: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M9 9l6 6M15 9l-6 6"/></svg>"##;
static TOAST_ACCENT: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 4l2.4 5 5.6.7-4.1 3.8 1.1 5.5-5-2.8-5 2.8 1.1-5.5L4 9.7l5.6-.7z"/></svg>"##;
static X: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#000" stroke-width="2" stroke-linecap="round"><path d="M6 6l12 12M18 6L6 18"/></svg>"##;

impl IconSet {
    /// An empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// The core's marks, under the names the standard components draw:
    /// `check` (a checked item), `dot` (a chosen radio button),
    /// `menu-radio` (a chosen radio item of a menu, the same dot; a theme
    /// that marks it with a check gives it its own image), `minus` (a mixed
    /// state), `chevron-down` (a select's trigger), `chevron-right` (a
    /// submenu), `x` (a close button), `plus` (an increase button) and the
    /// icons of a toast's tone: `toast-info`, `toast-success`,
    /// `toast-warning`, `toast-danger` and `toast-accent`.
    pub fn standard() -> Self {
        [
            ("check", CHECK),
            ("copy", COPY),
            ("arrow-down", ARROW_DOWN),
            ("dot", DOT),
            ("menu-radio", DOT),
            ("minus", MINUS),
            ("chevron-down", CHEVRON_DOWN),
            ("chevron-right", CHEVRON_RIGHT),
            ("chevron-left", CHEVRON_LEFT),
            ("chevrons-left", CHEVRONS_LEFT),
            ("chevrons-right", CHEVRONS_RIGHT),
            ("calendar", CALENDAR),
            ("x", X),
            ("plus", PLUS),
            ("toast-info", TOAST_INFO),
            ("toast-success", TOAST_SUCCESS),
            ("toast-warning", TOAST_WARNING),
            ("toast-danger", TOAST_DANGER),
            ("toast-accent", TOAST_ACCENT),
        ]
        .into_iter()
        .collect()
    }

    /// The set with `name` drawn as `image`, replacing an earlier image of
    /// the name.
    pub fn with(mut self, name: impl Into<Cow<'static, str>>, image: impl Into<Asset>) -> Self {
        std::sync::Arc::make_mut(&mut self.0).insert(name.into(), image.into());
        self
    }

    /// The image of `name`, if the set has one.
    pub fn get(&self, name: &str) -> Option<&Asset> {
        self.0.get(name)
    }

    /// The image `icon` draws: its own asset, or the image of its name in
    /// this set (`None` for a name the set does not have).
    pub fn resolve(&self, icon: &Icon) -> Option<Asset> {
        match &icon.0 {
            InputProp::Asset(asset) => Some(asset.clone()),
            InputProp::Named(name) => self.get(name).cloned(),
        }
    }

    /// The names in the set, in order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.0.keys().map(|name| name.as_ref())
    }
}

impl<N: Into<Cow<'static, str>>, A: Into<Asset>> Extend<(N, A)> for IconSet {
    /// Add every name, each replacing an earlier image of the same name.
    fn extend<I: IntoIterator<Item = (N, A)>>(&mut self, iter: I) {
        let map = std::sync::Arc::make_mut(&mut self.0);
        for (name, image) in iter {
            map.insert(name.into(), image.into());
        }
    }
}

impl<N: Into<Cow<'static, str>>, A: Into<Asset>> FromIterator<(N, A)> for IconSet {
    fn from_iter<I: IntoIterator<Item = (N, A)>>(iter: I) -> Self {
        let mut set = IconSet::new();
        set.extend(iter);
        set
    }
}
