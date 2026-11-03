//! Role: The theme value, its extensions, and the colors resolved in it.
//!
//! Invariants:
//! - `Theme::with(T)` installs a [`ThemeExtension`] value and `get::<T>()`
//!   returns the shared `Arc<T>` (else `ThemeExtension::fallback(theme)`).
//!   Themes compare installed extensions by value.
//! - A theme with no extension allocates no table, and a clone of an
//!   extended theme shares it.
//! - A theme switches through the tracking of `current_theme()` in
//!   `.style(..)` closures and of `provide_theme`, without remounting.
//! - `Color::components_in(theme)` and `mix_in(other, amount, theme)`
//!   resolve tokens in the given theme; `components` and `mix` use the
//!   default theme. Mixing interpolates alpha-premultiplied sRGB.

use super::{Palette, Radii, Spacing, Typography};
use crate::color::{Color, ColorToken, Repr, rgb, rgba};
use crate::icon::IconSet;
use kivact_reactive::{Owner, Value};
use kivact_wire::{LineHeight, Rgba};
use std::any::{Any, TypeId};
use std::fmt;
use std::sync::{Arc, Mutex};

/// A value a component library adds to a [`Theme`]: its own colors, sizes
/// and fonts beside the core's palette and scales (tokens), or a table of
/// style functions for its components (recipes).
///
/// A library defines one type for each, installs a value with
/// [`Theme::with`] in its theme constructor, and reads it with
/// [`Theme::get`] where it styles (a recipe, a style closure, a draw
/// callback). A read in a style closure tracks the theme, so switching the
/// theme restyles mounted components without a remount; the read shares the
/// installed value and copies nothing. Two themes are equal when their
/// installed extensions are equal.
///
/// The standard components read `kivact::ui::UiRecipes`, a table of style
/// functions with [`Default`] as its fallback; a component library may
/// define a table for its own components the same way. To change part of a
/// table a theme already has (a kit's look), clone the value that
/// [`Theme::get`] returns, change it and install it again.
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
/// assert_eq!(theme.get::<Brand>().highlight, rgb(0x0088ff));
/// assert_eq!(Theme::light().get::<Brand>().highlight, rgb(0xff8800));
/// ```
pub trait ThemeExtension: PartialEq + std::fmt::Debug + Send + Sync + 'static {
    /// The value a theme without this extension reads: the library's
    /// defaults, possibly derived from the theme's palette, so a component of
    /// the library still draws under another library's theme. A recipe
    /// table returns its [`Default`].
    fn fallback(theme: &Theme) -> Self;
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

/// A complete theme. Replace with `App::theme` or `kivact::view::provide_theme`.
///
/// Read its values with the accessors (`theme.space().md`) and change them
/// with the `with_*` builders, which keep everything else:
/// `Theme::light().with_focus_ring(3.0)`. A part of a table changes by
/// copying it: `theme.with_colors(Palette { accent, ..theme.colors().clone() })`.
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    extensions: Extensions,
    mode: ThemeMode,
    colors: Palette,
    space: Spacing,
    radius: Radii,
    text: Typography,
    focus_ring: f32,
    focus_ring_offset: f32,
    disabled_opacity: f32,
    control_height: f32,
    icons: IconSet,
}

impl Theme {
    /// Install an extension, replacing an earlier value of the same type.
    /// See [`ThemeExtension`].
    pub fn with<T: ThemeExtension>(mut self, value: T) -> Self {
        self.extensions.insert(Entry::new(value));
        self
    }

    /// The extension of type `T` installed in this theme, or
    /// [`ThemeExtension::fallback`] when there is none. The installed value
    /// is shared, not copied.
    pub fn get<T: ThemeExtension>(&self) -> Arc<T> {
        self.extensions
            .get::<T>()
            .unwrap_or_else(|| Arc::new(T::fallback(self)))
    }

    /// Whether the palette is light or dark.
    pub fn mode(&self) -> ThemeMode {
        self.mode
    }

    /// The color table (the value of each [`ColorToken`](crate::ColorToken)).
    pub fn colors(&self) -> &Palette {
        &self.colors
    }

    /// The spacing scale.
    pub fn space(&self) -> &Spacing {
        &self.space
    }

    /// The corner radii.
    pub fn radius(&self) -> &Radii {
        &self.radius
    }

    /// The base type settings.
    pub fn text(&self) -> &Typography {
        &self.text
    }

    /// The width (px) of the ring a focusable element draws in
    /// [`ColorToken::Focus`](crate::ColorToken::Focus) while it has
    /// visible focus, unless its style has a focus look of its own (a
    /// `focus` or `focus_visible` state;
    /// `style().focus_visible(style().ring_none())` draws none).
    pub fn focus_ring(&self) -> f32 {
        self.focus_ring
    }

    /// The gap in logical pixels between the border box and the theme focus ring.
    pub fn focus_ring_offset(&self) -> f32 {
        self.focus_ring_offset
    }

    /// The opacity a disabled focusable element is drawn with, unless its
    /// style's `disabled` state sets an opacity of its own. Every element
    /// with an interaction part counts, so an item of a roving list that is
    /// not the tab stop (a disabled one never is) is dimmed too
    /// (`style().disabled(style().opacity(1.0))` keeps it opaque). A disabled
    /// focusable element also drops its hover and pressed looks.
    pub fn disabled_opacity(&self) -> f32 {
        self.disabled_opacity
    }

    /// Minimum height of interactive controls (px).
    pub fn control_height(&self) -> f32 {
        self.control_height
    }

    /// The images an [`Icon`](crate::Icon) name resolves to. The
    /// core's set has the marks the standard components draw; a component
    /// library adds or replaces names with [`Theme::with_icon`].
    pub fn icons(&self) -> &IconSet {
        &self.icons
    }

    /// The theme with `mode`. Only the label changes: the palette stays as it
    /// is (a kit's constructor sets both).
    #[must_use]
    pub fn with_mode(mut self, mode: ThemeMode) -> Self {
        self.mode = mode;
        self
    }

    /// The theme with the color table `colors`.
    #[must_use]
    pub fn with_colors(mut self, colors: Palette) -> Self {
        self.colors = colors;
        self
    }

    /// The theme with the spacing scale `space`.
    #[must_use]
    pub fn with_space(mut self, space: Spacing) -> Self {
        self.space = space;
        self
    }

    /// The theme with the corner radii `radius`.
    #[must_use]
    pub fn with_radius(mut self, radius: Radii) -> Self {
        self.radius = radius;
        self
    }

    /// The theme with the type settings `text`.
    #[must_use]
    pub fn with_text(mut self, text: Typography) -> Self {
        self.text = text;
        self
    }

    /// The theme with the focus ring width `focus_ring` (px); see
    /// [`focus_ring`](Self::focus_ring).
    #[must_use]
    pub fn with_focus_ring(mut self, focus_ring: f32) -> Self {
        self.focus_ring = focus_ring;
        self
    }

    /// Set the gap in logical pixels before the theme focus ring.
    pub fn with_focus_ring_offset(mut self, offset: f32) -> Self {
        self.focus_ring_offset = offset;
        self
    }

    /// The theme with the disabled opacity `disabled_opacity`; see
    /// [`disabled_opacity`](Self::disabled_opacity).
    #[must_use]
    pub fn with_disabled_opacity(mut self, disabled_opacity: f32) -> Self {
        self.disabled_opacity = disabled_opacity;
        self
    }

    /// The theme with the minimum control height `control_height` (px).
    #[must_use]
    pub fn with_control_height(mut self, control_height: f32) -> Self {
        self.control_height = control_height;
        self
    }

    /// The theme with the icon set `icons`, replacing the whole set.
    #[must_use]
    pub fn with_icons(mut self, icons: IconSet) -> Self {
        self.icons = icons;
        self
    }

    /// The theme with `name` drawn as `image`, replacing an earlier image of
    /// the name and keeping the other icons.
    #[must_use]
    pub fn with_icon(
        mut self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        image: impl Into<crate::asset::Asset>,
    ) -> Self {
        self.icons = self.icons.with(name, image);
        self
    }

    /// Light business-tool theme.
    pub fn light() -> Theme {
        Theme {
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
            icons: IconSet::standard(),
        }
    }

    /// Dark theme with the same metrics.
    pub fn dark() -> Theme {
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

kivact_reactive::constant_value!(Theme);

impl Theme {
    /// Provide `theme` to `owner`'s subtree, replacing any inherited theme.
    /// The value stays reactive: readers below `owner` track it.
    ///
    /// The App's root and `kivact::view::provide_theme` use it;
    /// [`current_theme`] and [`Theme::of`] read the nearest one above the
    /// caller.
    pub fn install(owner: Owner, theme: Value<Theme>) {
        owner.provide(ThemeCtx(theme));
    }

    /// The theme provided to `owner`'s subtree by the nearest [`install`]
    /// above it, if any (the App's root always provides one). The value is
    /// not read here: reading it tracks the current computation.
    ///
    /// [`install`]: Theme::install
    pub fn of(owner: Owner) -> Option<Value<Theme>> {
        owner.context::<ThemeCtx>().map(|c| c.0)
    }
}

/// The theme in effect for the nearest owner (setup, handler, task or
/// effect): the one provided above it, else the default. Tracked when the
/// provided theme is reactive. A located `SetupOnly` misuse outside every
/// owner.
#[track_caller]
pub fn current_theme() -> Theme {
    let owner = Owner::ambient("current_theme", std::panic::Location::caller());
    match Theme::of(owner) {
        Some(s) => s.get(),
        None => Theme::default(),
    }
}

/// A color that resolves against the theme it is painted in: what every
/// color input of a style, a gradient or a [`Draw`](crate::draw::Draw) accepts.
///
/// [`Color`] is the identity and [`ColorToken`] reads the theme's
/// [`Palette`](crate::theme::Palette). A component kit names its own colors
/// with a `Color` enum that implements this trait by reading its tokens
/// (`theme.get::<Tokens>().border`), so `style().background(basic::Color::Border)`
/// follows a theme switch without a closure around it.
///
/// The returned [`Color`] may itself be a token or carry an alpha; it is
/// resolved against the same theme. An implementor is a small value type
/// (a field-less enum): the process keeps one shared copy of each distinct
/// value, so a value that is built from a number (`Gray(f32)`) would grow
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

/// Every distinct [`ThemeColor`] value stored so far; a `Repr::Themed`
/// indexes it. Values are never removed.
static THEMED: Mutex<Vec<&'static dyn Themed>> = Mutex::new(Vec::new());

fn themed_index<T: ThemeColor>(value: T) -> u32 {
    let mut table = THEMED
        .lock()
        .expect("THEMED lock poisoned: a panic already happened");
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
        .expect("THEMED lock poisoned: a panic already happened");
    table[index as usize]
}

impl Color {
    /// The color as the engine paints it: a literal as it is, a token looked
    /// up in `theme`, with the color's alpha applied.
    pub fn resolve(self, theme: &Theme) -> Rgba {
        match self.0 {
            Repr::Rgba(r, g, b, a) => Rgba { r, g, b, a },
            Repr::Token(t, a) => theme.colors.get(t).resolve_concrete().with_alpha_mul(a),
            Repr::Themed(i, a) => themed_value(i)
                .resolve(theme)
                .resolve(theme)
                .with_alpha_mul(a),
        }
    }

    fn resolve_concrete(self) -> Rgba {
        match self.0 {
            Repr::Rgba(r, g, b, a) => Rgba { r, g, b, a },
            Repr::Token(..) | Repr::Themed(..) => Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
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
    pub fn components(self) -> [f32; 4] {
        self.components_in(&Theme::default())
    }

    /// Components resolved against the supplied theme, in sRGB.
    pub fn components_in(self, theme: &Theme) -> [f32; 4] {
        let c = self.resolve(theme);
        [c.r, c.g, c.b, c.a]
    }

    /// Mix in premultiplied sRGB; `amount = 0` is self, `1` is other.
    /// Theme tokens resolve against the default theme. Use `mix_in` in a theme style.
    pub fn mix(self, other: Color, amount: impl Into<f64>) -> Color {
        self.mix_in(other, amount, &Theme::default())
    }

    /// Mix after resolving both colors against the supplied theme.
    pub fn mix_in(self, other: Color, amount: impl Into<f64>, theme: &Theme) -> Color {
        let t = (amount.into() as f32).clamp(0.0, 1.0);
        let a = self.components_in(theme);
        let b = other.components_in(theme);
        let alpha = a[3] + (b[3] - a[3]) * t;
        let component = |i| {
            if alpha > 0.0 {
                (a[i] * a[3] * (1.0 - t) + b[i] * b[3] * t) / alpha
            } else {
                0.0
            }
        };
        Color::from_rgba_f32(component(0), component(1), component(2), alpha)
    }
}

impl kivact_motion::Animatable for Color {
    const COMPONENTS: usize = 4;
    fn to_components(&self, out: &mut Vec<f32>) {
        out.extend_from_slice(&self.components());
    }
    fn from_components(c: &[f32]) -> Self {
        let g = |i: usize| c.get(i).copied().unwrap_or_default();
        Color::from_rgba_f32(g(0), g(1), g(2), g(3))
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
}
