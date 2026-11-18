//! Role: The theme value, its extensions, and the colors resolved in it.
//!
//! Invariants:
//! - `Theme::with(T)` installs a [`ThemeExtension`] value and `get::<T>()`
//!   returns the shared `Arc<T>` (else `ThemeExtension::fallback(theme)`).
//!   Themes compare installed extensions by value.
//! - A theme with no extension allocates no table, and a clone of an
//!   extended theme shares it.
//! - A theme switches through the tracking of `use_theme()` in
//!   `.style(..)` closures and of `provide_theme`, without remounting.
//! - `Color::components_in(theme)` and `mix_in(other, amount, theme)`
//!   resolve tokens in the given theme; `components` and `mix` use the
//!   default theme. Mixing interpolates alpha-premultiplied Oklab, and the
//!   result stays in its space until it is painted.

use super::{Palette, Radii, Spacing, Typography};
use crate::color::{Color, ColorToken, Repr, rgb, rgba};
use crate::icon::IconSet;
use kivact_runtime::{Owner, Value};
use kivact_types::cell::SharedDeferred;
use kivact_types::{ColorInterpolation, ColorSpace, LineHeight, Rgba, SpaceColor, interpolate};
use std::any::{Any, TypeId};
use std::fmt;
use std::sync::{Arc, Mutex};

/// A value a component library adds to a [`Theme`]: its own colors, sizes
/// and fonts beside the core's palette and scales (tokens), or a table of
/// style functions for its components (recipes).
///
/// A library defines one type for each, installs a value with
/// [`Theme::with`] in its theme constructor, and reads it with
/// [`Theme::ext`] where it styles (a recipe, a style closure, a draw
/// callback). A read in a style closure tracks the theme, so switching the
/// theme restyles mounted components without a remount; the read shares the
/// installed value and copies nothing. Two themes are equal when their
/// installed extensions are equal.
///
/// The standard components read `kivact::ui::UiRecipes`, a table of style
/// functions with [`Default`] as its fallback; a component library may
/// define a table for its own components the same way. To change part of a
/// table a theme already has (a kit's look), clone the value that
/// [`Theme::ext`] returns, change it and install it again.
///
/// Recipe tables compare their functions by identity, so an unchanged table
/// never restyles. Identity means the same `Arc`: a table built anew is a
/// different table even when it holds the same functions. Build a kit's
/// table once and share it (a `static` [`OnceLock`](std::sync::OnceLock)
/// that the theme constructor clones). A theme built in a closure that
/// builds its table each time (`move || Theme::light().with(UiRecipes::default()
/// .button(..))`) restyles every component under it on each run of the
/// closure. An [`IconSet`] built anew is compared name by name instead:
/// no restyle, but a comparison of every entry.
///
/// ```
/// use kivact::style::{Color, Theme, ThemeExtension, rgb};
/// #[derive(Debug, PartialEq)]
/// struct Brand {
///     highlight: Color,
/// }
/// impl ThemeExtension for Brand {
///     fn fallback(_: &Theme) -> Self {
///         Brand { highlight: rgb(0xff8800) }
///     }
/// }
/// let theme = Theme::light().with(Brand { highlight: rgb(0x0088ff) });
/// assert_eq!(theme.ext::<Brand>().highlight, rgb(0x0088ff));
/// assert_eq!(Theme::light().ext::<Brand>().highlight, rgb(0xff8800));
/// ```
pub trait ThemeExtension: PartialEq + std::fmt::Debug + Send + Sync + 'static {
    /// The value a theme without this extension reads: the library's
    /// defaults, possibly derived from the theme's palette, so a component of
    /// the library still draws under another library's theme. A recipe
    /// table returns its [`Default`].
    fn fallback(theme: &Theme) -> Self;
}

/// A shared installed theme extension.
pub struct ThemeRef<T>(Arc<T>);
impl<T> Clone for ThemeRef<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T> std::ops::Deref for ThemeRef<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T: Clone> ThemeRef<T> {
    /// Copy the extension value.
    pub fn to_owned(&self) -> T {
        (*self.0).clone()
    }
}
impl From<kivact_types::ThemeHint> for ThemeMode {
    fn from(value: kivact_types::ThemeHint) -> Self {
        match value {
            kivact_types::ThemeHint::Light => Self::Light,
            kivact_types::ThemeHint::Dark => Self::Dark,
        }
    }
}
/// The current theme's installed extension.
#[track_caller]
pub fn use_tokens<T: ThemeExtension>() -> ThemeRef<T> {
    use_theme().with(Theme::ext::<T>)
}

/// One installed extension, keyed by its type.
#[derive(Clone)]
struct Entry {
    id: TypeId,
    value: Arc<dyn Any + Send + Sync>,
    eq: fn(&dyn Any, &dyn Any) -> bool,
    debug: fn(&dyn Any, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
}

impl Entry {
    fn new<T: PartialEq + std::fmt::Debug + Send + Sync + 'static>(value: T) -> Self {
        fn eq<T: PartialEq + 'static>(a: &dyn Any, b: &dyn Any) -> bool {
            a.downcast_ref::<T>() == b.downcast_ref::<T>()
        }
        fn debug<T: std::fmt::Debug + 'static>(
            value: &dyn Any,
            f: &mut std::fmt::Formatter<'_>,
        ) -> std::fmt::Result {
            match value.downcast_ref::<T>() {
                Some(value) => value.fmt(f),
                None => Ok(()),
            }
        }
        Entry {
            id: TypeId::of::<T>(),
            value: Arc::new(value),
            eq: eq::<T>,
            debug: debug::<T>,
        }
    }
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.debug)(self.value.as_ref(), f)
    }
}

/// The installed extensions; a theme without any allocates
/// nothing, and clones share the table.
#[derive(Clone, Debug, Default)]
struct Extensions(Option<Arc<Vec<Entry>>>);

impl PartialEq for Extensions {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.as_deref().map_or(&[][..], Vec::as_slice);
        let b = other.0.as_deref().map_or(&[][..], Vec::as_slice);
        a.len() == b.len()
            && a.iter().all(|entry| {
                b.iter()
                    .find(|other| other.id == entry.id)
                    .is_some_and(|other| {
                        Arc::ptr_eq(&entry.value, &other.value)
                            || (entry.eq)(entry.value.as_ref(), other.value.as_ref())
                    })
            })
    }
}

impl Extensions {
    fn insert(&mut self, entry: Entry) {
        let entries = Arc::make_mut(self.0.get_or_insert_with(|| Arc::new(Vec::new())));
        match entries.iter_mut().find(|e| e.id == entry.id) {
            Some(slot) => *slot = entry,
            None => entries.push(entry),
        }
    }

    fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let entry = self
            .0
            .as_ref()?
            .iter()
            .find(|e| e.id == TypeId::of::<T>())?;
        Arc::downcast::<T>(entry.value.clone()).ok()
    }
}

/// Whether a theme is light or dark. Every kit's theme constructor takes
/// it first (`theme(ThemeMode::Dark, ..)`), and a kit's tokens fall back to
/// the values of a theme's mode. Which mode to show (the system's, a
/// setting) is the application's choice.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

/// The values of a [`Theme`], shared by its clones.
#[derive(Clone, Debug, PartialEq)]
struct ThemeData {
    extensions: Extensions,
    mode: ThemeMode,
    colors: Palette,
    space: Spacing,
    radius: Radii,
    text: Typography,
    focus_ring: f64,
    focus_ring_offset: f64,
    disabled_opacity: f64,
    control_height: f64,
    overlay_margin: f64,
    icons: IconSet,
    fonts: FontFiles,
    fallbacks: Fallbacks,
}

/// A complete theme. Replace with `App::theme` or `kivact::style::provide_theme`.
///
/// Read its values with the accessors (`theme.space().md`) and change them
/// with the `with_*` builders, which keep everything else:
/// `Theme::light().with_focus_ring(3.0)`. A part of a table changes by
/// copying it: `theme.with_colors(Palette { accent, ..theme.colors().clone() })`.
///
/// A theme is a shared value: a clone costs a reference count, and two
/// clones compare equal at once.
#[derive(Clone, Debug)]
pub struct Theme(Arc<ThemeData>);

impl PartialEq for Theme {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
    }
}

// A theme is a `static` (`Theme::light`) and crosses to embed threads.
const _: () = {
    const fn shared<T: Send + Sync>() {}
    shared::<Theme>();
};

/// Font files a theme registers, as assets (a path, embedded bytes, owned
/// or shared bytes). `Asset` compares embedded and shared bytes by identity
/// (address and allocation), never byte by byte; the host registers each
/// asset once by that identity.
#[derive(Clone, Debug, Default, PartialEq)]
struct FontFiles(Vec<crate::asset::Asset>);

/// The fallback values of the extensions this theme lacks, built once per
/// theme value on the first read (ST-07). A cache: it is not part of the
/// theme's equality, and a changed copy of the theme starts empty.
#[derive(Default)]
struct Fallbacks(Mutex<Vec<(TypeId, Arc<dyn Any + Send + Sync>)>>);

impl Clone for Fallbacks {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for Fallbacks {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl fmt::Debug for Fallbacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Fallbacks")
    }
}

impl Fallbacks {
    fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let list = self
            .0
            .lock()
            .expect("invariant: the theme fallback lock is not poisoned");
        let found = list
            .iter()
            .find(|(id, _)| *id == TypeId::of::<T>())
            .map(|(_, value)| Arc::clone(value));
        drop(list);
        found.and_then(|value| Arc::downcast::<T>(value).ok())
    }

    /// Keep `value` unless another reader stored one first; the one kept.
    fn insert<T: Send + Sync + 'static>(&self, value: Arc<T>) -> Arc<T> {
        let mut list = self
            .0
            .lock()
            .expect("invariant: the theme fallback lock is not poisoned");
        if let Some((_, kept)) = list.iter().find(|(id, _)| *id == TypeId::of::<T>())
            && let Ok(kept) = Arc::downcast::<T>(Arc::clone(kept))
        {
            return kept;
        }
        list.push((
            TypeId::of::<T>(),
            Arc::clone(&value) as Arc<dyn Any + Send + Sync>,
        ));
        value
    }
}

impl Theme {
    /// The theme's values to change. Every write goes through here: it
    /// copies shared values (`Arc::make_mut`) and empties the fallback
    /// cache, so a fallback that reads what changes is built again,
    /// whether this theme shared its values or not.
    fn edit(&mut self) -> &mut ThemeData {
        let data = Arc::make_mut(&mut self.0);
        data.fallbacks = Fallbacks::default();
        data
    }

    /// Install an extension, replacing an earlier value of the same type.
    /// See [`ThemeExtension`].
    pub fn with<T: ThemeExtension>(mut self, value: T) -> Self {
        self.edit().extensions.insert(Entry::new(value));
        self
    }

    /// The extension of type `T` installed in this theme, or
    /// [`ThemeExtension::fallback`] when there is none. The installed value
    /// is shared, not copied; a fallback is built once per theme value.
    pub fn ext<T: ThemeExtension>(&self) -> ThemeRef<T> {
        if let Some(value) = self.0.extensions.get::<T>() {
            return ThemeRef(value);
        }
        if let Some(value) = self.0.fallbacks.get::<T>() {
            return ThemeRef(value);
        }
        // Built outside the lock: a fallback may read other extensions.
        let built = Arc::new(T::fallback(self));
        ThemeRef(self.0.fallbacks.insert(built))
    }

    /// Whether the palette is light or dark.
    pub fn mode(&self) -> ThemeMode {
        self.0.mode
    }

    /// The color table (the value of each [`ColorToken`](crate::ColorToken)).
    pub fn colors(&self) -> &Palette {
        &self.0.colors
    }

    /// The spacing scale.
    pub fn space(&self) -> &Spacing {
        &self.0.space
    }

    /// The corner radii.
    pub fn radius(&self) -> &Radii {
        &self.0.radius
    }

    /// The base type settings.
    pub fn text(&self) -> &Typography {
        &self.0.text
    }

    /// The width (px) of the ring a focusable element draws in
    /// [`ColorToken::Focus`](crate::ColorToken::Focus) while it has
    /// visible focus, unless its style has a focus look of its own (a
    /// `focus` or `focus_visible` state;
    /// `style().focus_visible(style().ring_none())` draws none).
    pub fn focus_ring(&self) -> f64 {
        self.0.focus_ring
    }

    /// The gap in logical pixels between the border box and the theme focus ring.
    pub fn focus_ring_offset(&self) -> f64 {
        self.0.focus_ring_offset
    }

    /// The opacity a disabled focusable element is drawn with, unless its
    /// style's `disabled` state sets an opacity of its own. Every element
    /// with an interaction part counts, so an item of a roving list that is
    /// not the tab stop (a disabled one never is) is dimmed too
    /// (`style().disabled(style().opacity(1.0))` keeps it opaque). A disabled
    /// focusable element also drops its hover and pressed looks.
    pub fn disabled_opacity(&self) -> f64 {
        self.0.disabled_opacity
    }

    /// Minimum height of interactive controls (px).
    pub fn control_height(&self) -> f64 {
        self.0.control_height
    }
    /// Viewport inset used to fit overlays, in logical pixels.
    pub fn overlay_margin(&self) -> f64 {
        self.0.overlay_margin
    }
    /// Set the viewport inset used to fit overlays, in logical pixels.
    #[must_use]
    pub fn with_overlay_margin(mut self, margin: f64) -> Self {
        self.edit().overlay_margin = margin;
        self
    }

    /// The images an [`Icon`](crate::Icon) name resolves to. The
    /// core's set has the marks the standard components draw; a component
    /// library adds or replaces names with [`Theme::with_icon`].
    pub fn icons(&self) -> &IconSet {
        &self.0.icons
    }

    /// The font files the theme registers, in the order added.
    pub fn fonts(&self) -> &[crate::asset::Asset] {
        &self.0.fonts.0
    }

    /// The theme with these font files added to the ones it has (a family
    /// in them is then usable by name in `font_family`). `App::theme` and
    /// `kivact::test::RenderOptions::with_theme` register them; a theme
    /// switched to later registers its own too.
    /// Any asset is a font file: `include_bytes!(..)`, a path, a `Vec<u8>`
    /// read at run time, shared bytes.
    #[must_use]
    pub fn with_fonts(
        mut self,
        fonts: impl IntoIterator<Item = impl Into<crate::asset::Asset>>,
    ) -> Theme {
        let added: Vec<crate::asset::Asset> = fonts.into_iter().map(Into::into).collect();
        self.edit().fonts.0.extend(added);
        self
    }

    /// The theme with `mode`. Only the label changes: the palette stays as it
    /// is (a kit's constructor sets both).
    #[must_use]
    pub fn with_mode(mut self, mode: ThemeMode) -> Self {
        self.edit().mode = mode;
        self
    }

    /// The theme with the color table `colors`.
    #[must_use]
    pub fn with_colors(mut self, colors: Palette) -> Self {
        self.edit().colors = colors;
        self
    }

    /// The theme with the spacing scale `space`.
    #[must_use]
    pub fn with_space(mut self, space: Spacing) -> Self {
        self.edit().space = space;
        self
    }

    /// The theme with the corner radii `radius`.
    #[must_use]
    pub fn with_radius(mut self, radius: Radii) -> Self {
        self.edit().radius = radius;
        self
    }

    /// The theme with the type settings `text`.
    #[must_use]
    pub fn with_text(mut self, text: Typography) -> Self {
        self.edit().text = text;
        self
    }

    /// The theme with the focus ring width `focus_ring` (px); see
    /// [`focus_ring`](Self::focus_ring).
    #[must_use]
    pub fn with_focus_ring(mut self, focus_ring: impl Into<f64>) -> Self {
        self.edit().focus_ring = focus_ring.into();
        self
    }

    /// Set the gap in logical pixels before the theme focus ring.
    pub fn with_focus_ring_offset(mut self, offset: impl Into<f64>) -> Self {
        self.edit().focus_ring_offset = offset.into();
        self
    }

    /// The theme with the disabled opacity `disabled_opacity`; see
    /// [`disabled_opacity`](Self::disabled_opacity).
    #[must_use]
    pub fn with_disabled_opacity(mut self, disabled_opacity: impl Into<f64>) -> Self {
        self.edit().disabled_opacity = disabled_opacity.into();
        self
    }

    /// The theme with the minimum control height `control_height` (px).
    #[must_use]
    pub fn with_control_height(mut self, control_height: impl Into<f64>) -> Self {
        self.edit().control_height = control_height.into();
        self
    }

    /// The theme with the icon set `icons`, replacing the whole set.
    #[must_use]
    pub fn with_icons(mut self, icons: IconSet) -> Self {
        self.edit().icons = icons;
        self
    }

    /// The theme with `name` drawn as `image`, replacing an earlier image of
    /// the name and keeping the other icons.
    #[must_use]
    pub fn with_icon(
        mut self,
        name: impl Into<String>,
        image: impl Into<crate::asset::Asset>,
    ) -> Self {
        let name: String = name.into();
        let icons = self.0.icons.clone().with(name, image);
        self.edit().icons = icons;
        self
    }

    /// Light business-tool theme. A clone of one shared value.
    pub fn light() -> Theme {
        static LIGHT: SharedDeferred<Theme> = SharedDeferred::new();
        LIGHT.get_or_init(Theme::light_theme).clone()
    }

    /// Dark theme with the same metrics. A clone of one shared value.
    pub fn dark() -> Theme {
        static DARK: SharedDeferred<Theme> = SharedDeferred::new();
        DARK.get_or_init(Theme::dark_theme).clone()
    }

    fn light_theme() -> Theme {
        Theme(Arc::new(ThemeData {
            extensions: Extensions::default(),
            mode: ThemeMode::Light,
            colors: Palette {
                surface: rgb(0xffffff),
                surface_alt: rgb(0xf3f4f6),
                text: rgb(0x111827),
                text_muted: rgb(0x6b7280),
                accent: rgb(0x2563eb),
                accent_text: rgb(0xffffff),
                border: rgb(0xd1d5db),
                focus: rgb(0x2563eb),
                danger: rgb(0xdc2626),
                success: rgb(0x16a34a),
                warning: rgb(0xd97706),
                overlay: rgba(0x000000, 0.35),
                shadow: rgba(0x000000, 0.2),
                selection: rgba(0x2563eb, 0.15),
                // Text at 5% over Surface is close to SurfaceAlt; a press doubles it.
                hover: rgba(0x111827, 0.05),
                pressed: rgba(0x111827, 0.10),
                // A shadow lifts a menu off a light page; no tint needed.
                raised: rgb(0xffffff),
            },
            space: Spacing {
                xs: 2.0,
                sm: 4.0,
                md: 8.0,
                lg: 12.0,
                xl: 16.0,
            },
            radius: Radii {
                sm: 3.0,
                md: 6.0,
                lg: 10.0,
            },
            text: Typography {
                family: None,
                size: 14.0,
                size_small: 12.0,
                size_large: 18.0,
                weight: 400,
                weight_strong: 600,
                line_height: LineHeight::Scale(1.35),
            },
            focus_ring: 2.0,
            focus_ring_offset: 0.0,
            disabled_opacity: 0.5,
            control_height: 28.0,
            overlay_margin: 0.0,
            icons: IconSet::standard(),
            fonts: FontFiles::default(),
            fallbacks: Fallbacks::default(),
        }))
    }

    fn dark_theme() -> Theme {
        Theme::light()
            .with_mode(ThemeMode::Dark)
            .with_colors(Palette {
                surface: rgb(0x111827),
                surface_alt: rgb(0x1f2937),
                text: rgb(0xf9fafb),
                text_muted: rgb(0x9ca3af),
                accent: rgb(0x3b82f6),
                accent_text: rgb(0xffffff),
                border: rgb(0x374151),
                focus: rgb(0x60a5fa),
                danger: rgb(0xf87171),
                success: rgb(0x4ade80),
                warning: rgb(0xfbbf24),
                overlay: rgba(0x000000, 0.5),
                shadow: rgba(0x000000, 0.5),
                selection: rgba(0x3b82f6, 0.3),
                // Text at 7.5% over Surface is close to SurfaceAlt; a press doubles it.
                hover: rgba(0xf9fafb, 0.075),
                pressed: rgba(0xf9fafb, 0.15),
                // Shadows vanish on a dark page, so a floating surface is lighter.
                raised: rgb(0x1f2937),
            })
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::light()
    }
}

/// Context entry for the theme of a subtree (a signal, a memo or a
/// constant source node; style effects read it tracked).
#[derive(Clone, Copy)]
struct ThemeCtx(Value<Theme>);

kivact_runtime::constant_value!(Theme);

/// Provide `theme` to `owner`'s subtree, replacing any inherited theme.
/// The value stays reactive: readers below `owner` track it.
///
/// The App's root and `kivact::style::provide_theme` use it;
/// [`use_theme`] and [`theme_of`] read the nearest one above the
/// caller.
pub fn install_theme(owner: Owner, theme: Value<Theme>) {
    owner.provide(ThemeCtx(theme));
}

/// The theme provided to `owner`'s subtree by the nearest [`install_theme`]
/// above it, if any (the App's root always provides one). The value is
/// not read here: reading it tracks the current computation.
///
pub fn theme_of(owner: Owner) -> Option<Value<Theme>> {
    owner.context::<ThemeCtx>().map(|c| c.0)
}

/// The theme in effect for the nearest owner (setup, handler, task or
/// effect): the one provided above it, else the default. Tracked when the
/// provided theme is reactive. A located `SetupOnly` misuse outside every
/// owner.
#[track_caller]
pub fn use_theme() -> Value<Theme> {
    let owner = Owner::ambient("use_theme", std::panic::Location::caller());
    match theme_of(owner) {
        Some(s) => s,
        None => owner.constant(Theme::default(), std::panic::Location::caller()),
    }
}

/// A color that resolves against the theme it is painted in: what every
/// color input of a style, a gradient or a [`Draw`](crate::draw::Draw) accepts.
///
/// [`Color`] is the identity and [`ColorToken`] reads the theme's
/// [`Palette`](crate::theme::Palette). A component kit names its own colors
/// with a `Color` enum that implements this trait by reading its tokens
/// (`theme.ext::<Tokens>().border`), so `style().background(basic::ColorName::Border)`
/// follows a theme switch without a closure around it.
///
/// The returned [`Color`] may itself be a token or carry an alpha; it is
/// resolved against the same theme. An implementor is a small value type
/// (a field-less enum): the process keeps one shared copy of each distinct
/// value, so a value that is built from a number (`Gray(f64)`) would grow
/// that table without bound.
pub trait ThemeColor: Copy + PartialEq + fmt::Debug + Send + Sync + 'static {
    /// The color this value stands for in `theme`.
    fn resolve(self, theme: &Theme) -> Color;
}

impl ThemeColor for Color {
    fn resolve(self, _: &Theme) -> Color {
        self
    }
}

impl ThemeColor for ColorToken {
    fn resolve(self, theme: &Theme) -> Color {
        theme.colors().get(self)
    }
}

/// A [`ThemeColor`] value behind a shared reference, so a [`Color`] stays
/// `Copy`: [`Color::of`] stores one per distinct value and the color holds
/// its index.
trait Themed: Send + Sync {
    fn resolve(&self, theme: &Theme) -> Color;
    fn value(&self) -> &dyn Any;
}

impl<T: ThemeColor> Themed for T {
    fn resolve(&self, theme: &Theme) -> Color {
        ThemeColor::resolve(*self, theme)
    }
    fn value(&self) -> &dyn Any {
        self
    }
}

/// A non-shrinking interning table with one entry per distinct `ThemeColor`
/// value. A Copy `Color` indexes it without carrying a theme.
static THEMED: Mutex<Vec<&'static dyn Themed>> = Mutex::new(Vec::new());

fn themed_index<T: ThemeColor>(value: T) -> u32 {
    let mut table = THEMED
        .lock()
        .expect("invariant: the THEMED interning lock is not poisoned");
    let known = table
        .iter()
        .position(|v| v.value().downcast_ref::<T>() == Some(&value));
    known.unwrap_or_else(|| {
        table.push(Box::leak(Box::new(value)));
        table.len() - 1
    }) as u32
}

fn themed_value(index: u32) -> &'static dyn Themed {
    let table = THEMED
        .lock()
        .expect("invariant: the THEMED interning lock is not poisoned");
    table[index as usize]
}

impl Color {
    /// The color as the engine paints it: a literal as it is, a token looked
    /// up in `theme`, with the color's alpha applied. A color in a CSS color
    /// space is gamut mapped into sRGB here, once.
    pub fn resolve(self, theme: &Theme) -> Rgba {
        match self.0 {
            Repr::Rgba(r, g, b, a) => Rgba { r, g, b, a },
            Repr::Space(c) => c.to_srgb(),
            Repr::Token(t, a) => theme.0.colors.get(t).resolve_concrete().with_alpha_mul(a),
            Repr::Themed(i, a) => themed_value(i)
                .resolve(theme)
                .resolve(theme)
                .with_alpha_mul(a),
        }
    }

    /// The color in the space it was authored in (before gamut mapping), as
    /// gradient stops and interpolation take it.
    pub fn resolve_space(self, theme: &Theme) -> SpaceColor {
        match self.0 {
            Repr::Rgba(r, g, b, a) => SpaceColor::from_rgba(Rgba { r, g, b, a }),
            Repr::Space(c) => c,
            Repr::Token(t, a) => {
                let mut c = theme.0.colors.get(t).resolve_space_concrete();
                c.alpha *= a;
                c
            }
            Repr::Themed(i, a) => {
                let mut c = themed_value(i).resolve(theme).resolve_space(theme);
                c.alpha *= a;
                c
            }
        }
    }

    fn resolve_concrete(self) -> Rgba {
        match self.0 {
            Repr::Rgba(r, g, b, a) => Rgba { r, g, b, a },
            Repr::Space(c) => c.to_srgb(),
            Repr::Token(..) | Repr::Themed(..) => Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        }
    }

    /// A palette entry without a theme: a literal, or opaque black.
    fn resolve_space_concrete(self) -> SpaceColor {
        match self.0 {
            Repr::Rgba(r, g, b, a) => SpaceColor::from_rgba(Rgba { r, g, b, a }),
            Repr::Space(c) => c,
            Repr::Token(..) | Repr::Themed(..) => SpaceColor::from_rgba(Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
        }
    }

    /// The color of any [`ThemeColor`]: resolved against the theme when it
    /// is painted. A [`Color`] and a [`ColorToken`] are themselves.
    pub fn of<C: ThemeColor>(color: C) -> Color {
        let any: &dyn Any = &color;
        if let Some(c) = any.downcast_ref::<Color>() {
            *c
        } else if let Some(t) = any.downcast_ref::<ColorToken>() {
            Color::token(*t)
        } else {
            Color(Repr::Themed(themed_index(color), 1.0))
        }
    }

    /// Concrete components (tokens resolved against the default theme).
    pub fn components(self) -> [f64; 4] {
        self.components_in(&Theme::default())
    }

    /// Components resolved against the supplied theme, in sRGB.
    pub fn components_in(self, theme: &Theme) -> [f64; 4] {
        let c = self.resolve(theme);
        [c.r, c.g, c.b, c.a]
    }

    /// Mix in Oklab (premultiplied); `amount = 0` is self, `1` is other.
    /// Theme tokens resolve against the default theme. Use `mix_in` in a theme style.
    /// The result keeps its space: gamut mapping happens when it is painted.
    pub fn mix(self, other: Color, amount: impl Into<f64>) -> Color {
        self.mix_in(other, amount, &Theme::default())
    }

    /// Mix after resolving both colors against the supplied theme.
    pub fn mix_in(self, other: Color, amount: impl Into<f64>, theme: &Theme) -> Color {
        let t = (amount.into()).clamp(0.0, 1.0);
        let a = self.resolve_space(theme);
        let b = other.resolve_space(theme);
        Color(Repr::Space(interpolate(
            a,
            b,
            t,
            ColorInterpolation::DEFAULT,
        )))
    }

    /// The WCAG relative luminance of the color, from 0 (black) to 1
    /// (white). A theme token resolves in the theme in effect; the alpha
    /// is ignored.
    pub fn relative_luminance(self) -> f64 {
        let c = self.resolve(&ambient_theme()).sanitize();
        let linear = |v: f64| {
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(c.r) + 0.7152 * linear(c.g) + 0.0722 * linear(c.b)
    }

    /// The WCAG contrast ratio of two colors, from 1 to 21.
    pub fn contrast_ratio(self, other: Color) -> f64 {
        let (a, b) = (self.relative_luminance(), other.relative_luminance());
        let (high, low) = if a >= b { (a, b) } else { (b, a) };
        (high + 0.05) / (low + 0.05)
    }
}

/// The theme a color resolves in where no theme is passed. S16 makes it
/// the theme in effect (`try_use_theme`).
fn ambient_theme() -> Theme {
    Theme::light()
}

/// The Oklab premultiplied components of a color (L·α, a·α, b·α, α) that
/// animation interpolates.
fn premultiplied_oklab(c: SpaceColor) -> [f64; 4] {
    let ok = c.convert(ColorSpace::Oklab);
    let [l, a, b] = ok.channels;
    let alpha = ok.alpha;
    [l * alpha, a * alpha, b * alpha, alpha]
}

impl kivact_runtime::Animatable for Color {
    const COMPONENTS: usize = 4;
    fn to_components(&self, out: &mut Vec<f64>) {
        let c = self.resolve_space(&Theme::default());
        out.extend_from_slice(&premultiplied_oklab(c));
    }
    fn from_components(c: &[f64]) -> Self {
        let g = |i: usize| c.get(i).copied().unwrap_or_default();
        let alpha = g(3).clamp(0.0, 1.0);
        let unpre = |v: f64| if alpha > 0.0 { v / alpha } else { 0.0 };
        Color(Repr::Space(SpaceColor::new(
            ColorSpace::Oklab,
            [unpre(g(0)), unpre(g(1)), unpre(g(2))],
            alpha,
        )))
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests for theme-resolved colors.
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Kit {
        Edge,
        Fill,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Other {
        Edge,
    }

    impl ThemeColor for Kit {
        fn resolve(self, theme: &Theme) -> Color {
            match self {
                Kit::Edge => ColorToken::Border.resolve(theme),
                Kit::Fill => rgb(0x102030),
            }
        }
    }

    impl ThemeColor for Other {
        fn resolve(self, _: &Theme) -> Color {
            Color::WHITE
        }
    }

    #[test]
    fn color_and_token_stay_themselves() {
        assert_eq!(Color::of(Color::BLACK), Color::BLACK);
        assert_eq!(
            Color::of(ColorToken::Accent),
            Color::token(ColorToken::Accent)
        );
    }

    #[test]
    fn a_theme_color_resolves_against_the_theme_it_is_painted_in() {
        let light = Theme::light();
        let dark = Theme::dark();
        let edge = Color::of(Kit::Edge);
        assert_eq!(
            edge.resolve(&light),
            Color::token(ColorToken::Border).resolve(&light)
        );
        assert_eq!(
            edge.resolve(&dark),
            Color::token(ColorToken::Border).resolve(&dark)
        );
        assert_ne!(edge.resolve(&light), edge.resolve(&dark));
        assert_eq!(
            Color::of(Kit::Fill).resolve(&dark),
            rgb(0x102030).resolve(&dark)
        );
    }

    #[test]
    fn equal_values_are_one_color_and_other_values_differ() {
        assert_eq!(Color::of(Kit::Edge), Color::of(Kit::Edge));
        assert_ne!(Color::of(Kit::Edge), Color::of(Kit::Fill));
        assert_ne!(Color::of(Kit::Edge), Color::of(Other::Edge));
    }

    #[test]
    fn alpha_multiplies_what_the_value_resolves_to() {
        let theme = Theme::light();
        let half = Color::of(Kit::Fill).with_alpha(0.5);
        assert_eq!(half.resolve(&theme).a, 0.5);
        assert_eq!(half.with_alpha(0.5).resolve(&theme).a, 0.25);
        assert_eq!(
            Color::of(Kit::Edge).with_alpha(0.5).resolve(&theme).a,
            0.5 * Color::token(ColorToken::Border).resolve(&theme).a
        );
    }

    #[derive(Debug, PartialEq)]
    struct Height(f64);

    impl ThemeExtension for Height {
        fn fallback(theme: &Theme) -> Self {
            Height(theme.control_height())
        }
    }

    #[test]
    fn clones_share_one_value_and_compare_at_once() {
        let a = Theme::light();
        let b = Theme::light();
        assert!(a == b);
        assert!(Arc::ptr_eq(&a.0, &b.0));
    }

    #[test]
    fn a_missing_extension_falls_back_once_per_theme() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static BUILT: AtomicU32 = AtomicU32::new(0);
        #[derive(Debug, PartialEq)]
        struct Probe(u32);
        impl ThemeExtension for Probe {
            fn fallback(_: &Theme) -> Self {
                BUILT.fetch_add(1, Ordering::Relaxed);
                Probe(1)
            }
        }
        let theme = Theme::light();
        assert_eq!((*theme.ext::<Probe>()).0, 1);
        assert_eq!((*theme.ext::<Probe>()).0, 1);
        assert_eq!(BUILT.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn a_builder_on_a_unique_theme_rebuilds_the_fallback() {
        let t = Theme::light().with_control_height(30);
        assert_eq!((*t.ext::<Height>()).0, 30.0);
        let t = t.with_control_height(40);
        assert_eq!((*t.ext::<Height>()).0, 40.0);
    }

    #[test]
    fn a_builder_on_a_shared_theme_rebuilds_the_fallback_and_keeps_the_clone() {
        let a = Theme::light().with_control_height(30);
        let _ = a.ext::<Height>();
        let b = a.clone().with_control_height(40);
        assert_eq!((*b.ext::<Height>()).0, 40.0);
        assert_eq!((*a.ext::<Height>()).0, 30.0);
    }

    #[test]
    fn installing_an_extension_rebuilds_a_fallback_that_reads_it() {
        #[derive(Debug, PartialEq)]
        struct Base(u32);
        impl ThemeExtension for Base {
            fn fallback(_: &Theme) -> Self {
                Base(1)
            }
        }
        #[derive(Debug, PartialEq)]
        struct Twice(u32);
        impl ThemeExtension for Twice {
            fn fallback(t: &Theme) -> Self {
                Twice((*t.ext::<Base>()).0 * 2)
            }
        }
        let t = Theme::light();
        assert_eq!((*t.ext::<Twice>()).0, 2);
        let t = t.with(Base(5));
        assert_eq!((*t.ext::<Twice>()).0, 10);
    }

    #[test]
    fn fonts_take_any_asset() {
        use crate::asset::Asset;
        let two = Theme::light()
            .with_fonts([Asset::from(&b"x"[..])])
            .with_fonts(vec![Asset::from(vec![1u8, 2])]);
        assert_eq!(two.fonts().len(), 2);
        assert!(
            Theme::light()
                .with_fonts(Vec::<Asset>::new())
                .fonts()
                .is_empty()
        );
    }
}
