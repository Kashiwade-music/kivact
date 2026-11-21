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
//! - `Color::components(theme)` and `mix(other, amount, theme)` resolve
//!   variables in the given theme; animation and luminance resolve them in
//!   the theme in effect (`try_use_theme`). Mixing interpolates
//!   alpha-premultiplied Oklab, and the result stays in its space until it
//!   is painted.

use super::vars::{ThemeValue, VarTable, lookup};
use crate::color::{
    ACCENT_TEXT_VAR, ACCENT_VAR, BORDER_VAR, DANGER_VAR, FOCUS_VAR, HOVER_VAR, OVERLAY_VAR,
    PRESSED_VAR, RAISED_VAR, SELECTION_VAR, SHADOW_VAR, SUCCESS_VAR, SURFACE_ALT_VAR, SURFACE_VAR,
    TEXT_MUTED_VAR, TEXT_VAR, WARNING_VAR,
};
use crate::color_value::{Color, Repr, rgb, rgba};
use crate::icon::IconSet;
use kivact_runtime::driver::EffectKind;
use kivact_runtime::{ErrorKind, Owner, UiError, Value};
use kivact_types::cell::{ReportOnceSet, SharedDeferred};
use kivact_types::{
    ColorInterpolation, ColorSpace, Rgba, SharedTypeMap, SpaceColor, ThemeVar, ThemeVarId,
    VarValue, interpolate,
};
use std::any::{Any, TypeId};
use std::fmt;
use std::sync::{Arc, Mutex};

/// A value a component library adds to a [`Theme`]: its own tables beside
/// the theme's variables (recipes, policies), or a table of style functions
/// for its components (recipes).
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
pub use kivact_types::ThemeMode;

/// The values of a [`Theme`], shared by its clones.
#[derive(Clone, Debug, PartialEq)]
struct ThemeData {
    extensions: SharedTypeMap,
    mode: ThemeMode,
    vars: VarTable,
    focus_ring: f64,
    focus_ring_offset: f64,
    icons: IconSet,
    fonts: FontFiles,
    fallbacks: Fallbacks,
}

/// A complete theme. Replace with `App::theme` or `kivact::style::provide_theme`.
///
/// Read its theme variables with [`get`](Self::get) (`theme.get(&space::MD_VAR)`)
/// and set them with [`set`](Self::set), which keeps everything else:
/// `Theme::light().set(&color::ACCENT_VAR, rgb(0xff0000))`. The `with_*`
/// builders change the other fields: `Theme::light().with_focus_ring(3.0)`.
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
        let data = self.edit();
        data.extensions = std::mem::take(&mut data.extensions).with(value);
        self
    }

    /// The extension of type `T` installed in this theme, or
    /// [`ThemeExtension::fallback`] when there is none. The installed value
    /// is shared, not copied; a fallback is built once per theme value.
    pub fn ext<T: ThemeExtension>(&self) -> ThemeRef<T> {
        if let Some(value) = self.0.extensions.get_shared::<T>() {
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

    /// The value of `var` in this theme: the value it sets, else the
    /// variable's fallback, aliases followed.
    pub fn get<T: ThemeValue>(&self, var: &'static ThemeVar<T>) -> T {
        lookup(self, var)
    }

    /// The theme with `var` set to `value`: a value, or another variable
    /// of the same type (an alias, CSS `--accent: var(--primary)`:
    /// `.set(&color::ACCENT_VAR, &PRIMARY_VAR)`, `.set(&FIELD_RADIUS_VAR, &radius::MD_VAR)`),
    /// followed up to 8 steps; a cycle uses the fallback and is reported
    /// once (`ErrorKind::Unsupported`) where the theme is provided.
    #[must_use]
    pub fn set<T: ThemeValue>(
        mut self,
        var: &'static ThemeVar<T>,
        value: impl Into<VarValue<T>>,
    ) -> Theme {
        self.edit()
            .vars
            .insert(ThemeVarId::of(var), var.name(), T::into_slot(value.into()));
        self
    }

    /// The variable's slot in this theme, if the theme sets it.
    pub(crate) fn slot(&self, id: ThemeVarId) -> Option<&super::vars::VarSlot> {
        self.0.vars.get(id)
    }

    /// The names of the variables of this theme whose aliases do not
    /// end (reported by `install_theme`).
    #[doc(hidden)]
    pub fn var_problems(&self) -> Vec<&'static str> {
        self.0.vars.problems(self)
    }

    /// The width (px) of the ring a focusable element draws in
    /// [`color::FOCUS`](crate::color::FOCUS) while it has
    /// visible focus, unless its style has a focus look of its own (a
    /// `focus` or `focus_visible` state;
    /// `style().focus_visible(style().outline_none())` draws none).
    pub fn focus_ring(&self) -> f64 {
        self.0.focus_ring
    }

    /// The gap in logical pixels between the border box and the theme focus ring.
    pub fn focus_ring_offset(&self) -> f64 {
        self.0.focus_ring_offset
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
            extensions: SharedTypeMap::new(),
            mode: ThemeMode::Light,
            vars: VarTable::default(),
            focus_ring: 2.0,
            focus_ring_offset: 0.0,
            icons: IconSet::standard(),
            fonts: FontFiles::default(),
            fallbacks: Fallbacks::default(),
        }))
    }

    fn dark_theme() -> Theme {
        Theme::light()
            .with_mode(ThemeMode::Dark)
            .set(&SURFACE_VAR, rgb(0x111827))
            .set(&SURFACE_ALT_VAR, rgb(0x1f2937))
            .set(&TEXT_VAR, rgb(0xf9fafb))
            .set(&TEXT_MUTED_VAR, rgb(0x9ca3af))
            .set(&ACCENT_VAR, rgb(0x3b82f6))
            .set(&ACCENT_TEXT_VAR, rgb(0xffffff))
            .set(&BORDER_VAR, rgb(0x374151))
            .set(&FOCUS_VAR, rgb(0x60a5fa))
            .set(&DANGER_VAR, rgb(0xf87171))
            .set(&SUCCESS_VAR, rgb(0x4ade80))
            .set(&WARNING_VAR, rgb(0xfbbf24))
            .set(&OVERLAY_VAR, rgba(0x000000, 0.5))
            .set(&SHADOW_VAR, rgba(0x000000, 0.5))
            .set(&SELECTION_VAR, rgba(0x3b82f6, 0.3))
            // Text at 7.5% over Surface is close to SurfaceAlt; a press doubles it.
            .set(&HOVER_VAR, rgba(0xf9fafb, 0.075))
            .set(&PRESSED_VAR, rgba(0xf9fafb, 0.15))
            // Shadows vanish on a dark page, so a floating surface is lighter.
            .set(&RAISED_VAR, rgb(0x1f2937))
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
#[track_caller]
pub fn install_theme(owner: Owner, theme: Value<Theme>) {
    owner.provide(ThemeCtx(theme));
    let at = std::panic::Location::caller();
    let reported = ReportOnceSet::new();
    owner.effect(
        EffectKind::Render,
        move || {
            for name in theme.with(Theme::var_problems) {
                if reported.note(name) {
                    owner.report(UiError::new(
                        ErrorKind::Unsupported,
                        format!(
                            "theme variable `{name}` is an alias that does not end within 8 steps (a cycle?); its fallback is used"
                        ),
                        Some(at),
                    ));
                }
            }
        },
        at,
    );
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
    let owner = Owner::running("use_theme", std::panic::Location::caller());
    match theme_of(owner) {
        Some(s) => s,
        None => owner.constant(Theme::default(), std::panic::Location::caller()),
    }
}

impl Color {
    /// The color as the engine paints it: a literal as it is, a token looked
    /// up in `theme`, with the color's alpha applied. A color in a CSS color
    /// space is gamut mapped into sRGB here, once.
    pub(crate) fn resolve(self, theme: &Theme) -> Rgba {
        match self.0 {
            Repr::Rgba(r, g, b, a) => Rgba { r, g, b, a },
            Repr::Space(c) => c.to_srgb(),
            Repr::Var(v, a) => theme.get(v).resolve(theme).with_alpha_mul(a),
        }
    }

    /// The color in the space it was authored in (before gamut mapping), as
    /// gradient stops and interpolation take it.
    pub(crate) fn resolve_space(self, theme: &Theme) -> SpaceColor {
        match self.0 {
            Repr::Rgba(r, g, b, a) => SpaceColor::from_rgba(Rgba { r, g, b, a }),
            Repr::Space(c) => c,
            Repr::Var(v, a) => {
                let mut c = theme.get(v).resolve_space(theme);
                c.alpha *= a;
                c
            }
        }
    }

    /// Components resolved against the supplied theme, in sRGB.
    pub fn components(self, theme: &Theme) -> [f64; 4] {
        let c = self.resolve(theme);
        [c.r, c.g, c.b, c.a]
    }

    /// Mix in Oklab (premultiplied) after resolving both colors in `theme`;
    /// `amount = 0` is self, `1` is other. The result keeps its space until
    /// it is painted.
    pub fn mix(self, other: Color, amount: impl Into<f64>, theme: &Theme) -> Color {
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

/// The theme a color resolves in where no theme is passed: the theme in
/// effect, else the light theme.
fn ambient_theme() -> Theme {
    crate::internal::try_use_theme().unwrap_or_else(Theme::light)
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
        let c = self.resolve_space(&ambient_theme());
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

    use crate::color::ACCENT;
    use crate::font;
    use crate::radius;
    use crate::theme::FontFamily;
    use kivact_types::LineHeight;

    static PRIMARY_VAR: ThemeVar<Color> = ThemeVar::new("test.primary", rgb(0x00ff00));
    static A_VAR: ThemeVar<Color> = ThemeVar::new("test.a", rgb(0x010203));
    static B_VAR: ThemeVar<Color> = ThemeVar::new("test.b", rgb(0x040506));
    static NA: ThemeVar<f64> = ThemeVar::new("test.na", 7.0);
    static NB: ThemeVar<f64> = ThemeVar::alias("test.nb", &NA);
    static FIELD_VAR: ThemeVar<f64> = ThemeVar::alias("test.field", &radius::MD_VAR);
    static WEIGHT_ALIAS_VAR: ThemeVar<u16> =
        ThemeVar::alias("test.weight", &font::WEIGHT_STRONG_VAR);
    static FAMILY_ALIAS_VAR: ThemeVar<FontFamily> =
        ThemeVar::new("test.family", FontFamily::SYSTEM);
    static LINE_HEIGHT_ALIAS_VAR: ThemeVar<LineHeight> =
        ThemeVar::new("test.line_height", LineHeight::Normal);

    #[test]
    fn a_variable_takes_the_theme_value_or_its_fallback() {
        assert_eq!(Theme::light().get(&ACCENT_VAR), rgb(0x2563eb));
        let theme = Theme::light().set(&ACCENT_VAR, rgb(0xff0000));
        assert_eq!(theme.get(&ACCENT_VAR), rgb(0xff0000));
    }

    #[test]
    fn an_alias_resolves_through_another_variable() {
        let theme = Theme::light().set(&ACCENT_VAR, Color::var(&PRIMARY_VAR));
        assert_eq!(
            theme.get(&ACCENT_VAR).resolve(&theme),
            rgb(0x00ff00).resolve(&theme)
        );
    }

    #[test]
    fn alpha_multiplies_through_an_alias() {
        let theme = Theme::light().set(&ACCENT_VAR, Color::var(&PRIMARY_VAR).with_alpha(0.5));
        assert_eq!(theme.get(&ACCENT_VAR).resolve(&theme).a, 0.5);
        assert_eq!(ACCENT.resolve(&theme).a, 0.5);
    }

    #[test]
    fn a_cycle_takes_the_fallback_and_is_named() {
        let theme = Theme::light().set(&A_VAR, &B_VAR).set(&B_VAR, &A_VAR);
        assert_eq!(theme.get(&A_VAR), rgb(0x010203));
        let problems = theme.var_problems();
        assert!(problems.contains(&"test.a"));
        assert!(problems.contains(&"test.b"));
    }

    #[test]
    fn a_number_cycle_takes_the_fallback_and_is_named() {
        let theme = Theme::light().set(&NA, &NB).set(&NB, &NA);
        assert_eq!(theme.get(&NA), 7.0);
        let problems = theme.var_problems();
        assert!(problems.contains(&"test.na"));
        assert!(problems.contains(&"test.nb"));
    }

    #[test]
    fn every_type_aliases_as_a_value_and_as_a_fallback() {
        assert_eq!(Theme::light().get(&FIELD_VAR), 6.0);
        let theme = Theme::light().set(&radius::MD_VAR, 20);
        assert_eq!(theme.get(&FIELD_VAR), 20.0);
        assert_eq!(theme.set(&FIELD_VAR, 4).get(&FIELD_VAR), 4.0);
        assert_eq!(Theme::light().get(&WEIGHT_ALIAS_VAR), 600);
        let theme = Theme::light()
            .set(&FAMILY_ALIAS_VAR, &font::FAMILY_VAR)
            .set(&font::FAMILY_VAR, FontFamily::from("Inter"));
        assert_eq!(theme.get(&FAMILY_ALIAS_VAR).as_str(), Some("Inter"));
        let theme = Theme::light().set(&LINE_HEIGHT_ALIAS_VAR, &font::LINE_HEIGHT_VAR);
        assert_eq!(theme.get(&LINE_HEIGHT_ALIAS_VAR), LineHeight::Scale(1.35));
    }

    #[test]
    fn dark_sets_the_core_colors() {
        assert_eq!(Theme::dark().get(&crate::color::SURFACE_VAR), rgb(0x111827));
    }

    #[test]
    fn components_and_mix_resolve_in_the_given_theme() {
        let dark = Theme::dark();
        let surface = crate::color::SURFACE.components(&dark);
        assert_eq!(surface, rgb(0x111827).components(&dark));
        // Mixing goes through Oklab and back, so the components agree to
        // float rounding only.
        let mixed = crate::color::SURFACE.mix(crate::color::TEXT, 0.0, &dark);
        for (got, want) in mixed.components(&dark).into_iter().zip(surface) {
            assert!((got - want).abs() < 1e-9, "{got} != {want}");
        }
    }

    #[derive(Debug, PartialEq)]
    struct Height(f64);

    impl ThemeExtension for Height {
        fn fallback(theme: &Theme) -> Self {
            Height(theme.get(&crate::control::HEIGHT_VAR))
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
        let t = Theme::light().set(&crate::control::HEIGHT_VAR, 30);
        assert_eq!((*t.ext::<Height>()).0, 30.0);
        let t = t.set(&crate::control::HEIGHT_VAR, 40);
        assert_eq!((*t.ext::<Height>()).0, 40.0);
    }

    #[test]
    fn a_builder_on_a_shared_theme_rebuilds_the_fallback_and_keeps_the_clone() {
        let a = Theme::light().set(&crate::control::HEIGHT_VAR, 30);
        let _ = a.ext::<Height>();
        let b = a.clone().set(&crate::control::HEIGHT_VAR, 40);
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
