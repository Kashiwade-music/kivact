//! Role: Virtualized, keyboard-navigable list built from public parts.
//!
//! `LazyList`: a virtualized, keyboard-navigable list built from public
//! parts only — it does **not** use the core `virtual_list`.
//!
//! * The viewport height comes from `NodeRef::rect()` (host layout).
//! * Scrolling is owned by the component: `on_wheel` and the keyboard move a
//!   `Signal<f32>` offset; a content box is shifted by `top: -offset`.
//! * Only the visible window (plus one row) is mounted with a keyed
//!   `for_each`; each row is absolutely positioned at `index * row_height`.
//! * A scrollbar is drawn with `draw_over`; its thumb is a `Draw::region`
//!   that can be dragged.
//! * Selection uses a `Selector`, so moving it touches two rows only.
//!
//! ```
//! use kivact::prelude::*;
//! use kivact_external_component::lazy_list;
//!
//! use kivact::signal;
//!
//! let screen = kivact::test::render(|| {
//!     let items = signal((0..10_000u32).collect::<Vec<_>>());
//!     lazy_list("Numbers", items, |n| *n, |_, n: Value<u32>| {
//!         text(move || format!("#{}", n.get()))
//!     })
//!     .row_height(20.0)
//!     .style(style().height(200))
//! });
//! screen.get_by_text("#0");
//! assert!(screen.query_by_text("#500").is_none(), "not mounted");
//! screen.get_by_role(Role::List, "Numbers").scroll(0.0, 10_000.0);
//! screen.get_by_text("#500");
//! assert!(screen.query_by_text("#0").is_none(), "scrolled out");
//! ```

use kivact::primitive::NodeExt;
use kivact::style::current_theme;
use kivact::view::View;
use std::hash::Hash;
use std::panic::Location;
use std::rc::Rc;

use kivact::Callback;
use kivact::Selector;
use kivact::draw::{Draw, Rect, RegionId, Size};
use kivact::prelude::*;
use kivact::style::Color;
use kivact::style::Position;
use kivact::view::{Handled, KeyEvent, PointerEvent, selector};
use kivact::view::{Modifiers, PointerPhase, WheelEvent};

use crate::skin::{self, look};

type RowFn<K, T> = Rc<dyn Fn(K, Value<T>) -> View>;
type NameFn<T> = Rc<dyn Fn(&T) -> String>;

/// Builder of a lazy list. Built by [`lazy_list`].
pub struct LazyList<T: 'static, K: 'static> {
    root: Node,
    items: Value<Vec<T>>,
    key: Rc<dyn Fn(&T) -> K>,
    row: RowFn<K, T>,
    row_height: f32,
    selection: Option<Signal<Option<K>>>,
    on_activate: Option<Callback<K>>,
    item_name: Option<NameFn<T>>,
    at: &'static Location<'static>,
}

/// A virtualized list over `items`, keyed by `key`; `row` builds the content
/// of one row (mounted only while visible). Its nodes are located at the
/// call of `lazy_list`.
#[track_caller]
pub fn lazy_list<T, K, V>(
    label: impl IntoText,
    items: impl IntoValue<Vec<T>>,
    key: impl Fn(&T) -> K + 'static,
    row: impl Fn(K, Value<T>) -> V + 'static,
) -> LazyList<T, K>
where
    T: Clone + PartialEq + 'static,
    K: Eq + Hash + Clone + 'static,
    V: IntoView,
{
    LazyList {
        root: node()
            .role(Role::List)
            .name(label)
            .focusable(true)
            .style(
                style()
                    .position(Position::Relative)
                    .overflow(kivact::style::Overflow::Hidden)
                    .min_height(DEFAULT_ROW_HEIGHT),
            )
            .style(move || skin::focus_ring(&current_theme())),
        items: items.into_value(),
        key: Rc::new(key),
        row: Rc::new(move |k, v| row(k, v).into_view()),
        row_height: DEFAULT_ROW_HEIGHT,
        selection: None,
        on_activate: None,
        item_name: None,
        at: Location::caller(),
    }
}

impl<T: 'static, K: 'static> LazyList<T, K> {
    /// Fixed row height in logical px (default 28). The list is at least
    /// one row high.
    pub fn row_height(mut self, h: f32) -> Self {
        self.row_height = h.max(1.0);
        let h = self.row_height;
        self.style(style().min_height(h))
    }

    /// Selected key: click selects, ↑/↓/PageUp/PageDown/Home/End move it.
    /// Named `selection` so that `NodeBuilder::selected` (the semantic
    /// selected state of the root) stays reachable.
    pub fn selection(mut self, s: Signal<Option<K>>) -> Self {
        self.selection = Some(s);
        self
    }

    /// Enter on the selected row, or a double click.
    pub fn on_activate(mut self, cb: Callback<K>) -> Self {
        self.on_activate = Some(cb);
        self
    }

    /// Accessible name of each row (default: none, the row's text is read).
    pub fn item_name(mut self, f: impl Fn(&T) -> String + 'static) -> Self {
        self.item_name = Some(Rc::new(f));
        self
    }
}

impl<T: 'static, K: 'static> NodeBuilder for LazyList<T, K> {
    fn node_mut(&mut self) -> &mut Node {
        &mut self.root
    }
}

impl<T, K> IntoView for LazyList<T, K>
where
    T: Clone + PartialEq + 'static,
    K: Eq + Hash + Clone + 'static,
{
    fn into_view(self) -> View {
        lazy_list_view(self).into_view()
    }
}

const BAR_W: f32 = 8.0;
const DEFAULT_ROW_HEIGHT: f32 = 28.0;
const THUMB: RegionId = RegionId(1);
/// Height assumed before the host reports the first layout.
const FALLBACK_VIEWPORT: f32 = 480.0;

fn lazy_list_view<T, K>(p: LazyList<T, K>) -> impl IntoView
where
    T: Clone + PartialEq + 'static,
    K: Eq + Hash + Clone + 'static,
{
    let look = look();
    let LazyList {
        root,
        items,
        key,
        row,
        row_height: h,
        selection,
        on_activate,
        item_name,
        at,
    } = p;
    let selected = selection.unwrap_or_else(|| signal(None));
    let sel: Selector<K> = selector(selected);
    let offset = signal(0.0f32);
    let list_ref = node_ref();
    let rect = list_ref.rect();
    let viewport = memo(move || {
        rect.get()
            .map(|r| r.height)
            .filter(|h| *h > 0.0)
            .unwrap_or(FALLBACK_VIEWPORT)
    });
    let len = memo(move || items.with(|v| v.len()));
    let max_offset = memo(move || (len.get() as f32 * h - viewport.get()).max(0.0));
    // Clamp whenever the content shrinks (filtering) or the viewport grows.
    let shown = memo(move || offset.get().clamp(0.0, max_offset.get()));
    let scroll_to = move |y: f32| offset.set(y.clamp(0.0, max_offset.get_untracked()));
    let window = memo(move || {
        let off = shown.get();
        let first = (off / h).floor() as usize;
        let last = ((off + viewport.get()) / h).ceil() as usize + 1;
        items.with(|v| {
            let end = last.min(v.len());
            let start = first.min(end);
            v[start..end]
                .iter()
                .enumerate()
                .map(|(i, t)| (start + i, t.clone()))
                .collect::<Vec<(usize, T)>>()
        })
    });
    // Index of the selected key (O(N) only when the selection or items change).
    let key_idx = key.clone();
    let selected_index = memo(move || {
        let k = selected.get()?;
        items.with(|v| v.iter().position(|t| key_idx(t) == k))
    });
    let key_at = {
        let key = key.clone();
        move |i: usize| items.with_untracked(|v| v.get(i).map(|t| key(t)))
    };
    let reveal = move |i: usize| {
        let top = i as f32 * h;
        let off = offset.get_untracked();
        let vh = viewport.get_untracked();
        if top < off {
            scroll_to(top);
        } else if top + h > off + vh {
            scroll_to(top + h - vh);
        }
    };
    let key_at2 = key_at.clone();
    let move_selection = move |to: usize| {
        let n = len.get_untracked();
        if n == 0 {
            return;
        }
        let i = to.min(n - 1);
        selected.set(key_at2(i));
        reveal(i);
    };
    let page = move || ((viewport.get_untracked() / h).floor() as usize).max(1);
    let key_row = key.clone();
    let rows = for_each(
        window,
        move |(_, t): &(usize, T)| key_row(t),
        move |k: K, slot: Value<(usize, T)>| {
            let item = memo(move || slot.with(|(_, t)| t.clone()));
            let k_sel = k.clone();
            let k_press = k.clone();
            let name = item_name.clone();
            let mut b = node()
                .at(at)
                .role(Role::ListItem)
                .selected(move || sel.is(&k_sel))
                .on_press(move || selected.set(Some(k_press.clone())))
                .style(move || {
                    let t = &current_theme();
                    skin::list_row(look(), t)
                })
                .style(move || {
                    style()
                        .position(Position::Absolute)
                        .left(0)
                        .right(BAR_W)
                        .top(slot.with(|(i, _)| *i as f32 * h))
                        .height(h)
                });
            if let Some(f) = name {
                b = b.name(move || item.with(|t| f(t)));
            }
            b.child(row(k, item))
        },
    );
    let content = node()
        .at(at)
        .style(move || {
            style()
                .position(Position::Absolute)
                .left(0)
                .right(0)
                .top(-shown.get())
                .height(len.get() as f32 * h)
        })
        .child(rows);
    let dragging = var(None::<(f32, f32)>);
    let thumb = move |size: Size| -> Option<Rect> {
        let total = len.get() as f32 * h;
        if total <= size.height || size.height <= 0.0 {
            return None;
        }
        let th = (size.height * size.height / total).max(24.0);
        let y = shown.get() / (total - size.height) * (size.height - th);
        Some(Rect::new(size.width - BAR_W, y, BAR_W, th))
    };
    let activate = on_activate;
    let key_act = key_at.clone();
    let key_dbl = key_at.clone();
    let activate_selected = move || {
        if let (Some(i), Some(cb)) = (selected_index.get_untracked(), activate)
            && let Some(k) = key_dbl(i)
        {
            cb.call(k);
        }
    };
    root.region(
        THUMB,
        kivact::primitive::part(|state| state)
            .role(Role::Slider)
            .name("Scroll position")
            .value_text(move || {
                let first = (shown.get() / h) as usize + 1;
                format!("row {first} of {}", len.get())
            }),
    )
    .value_text(move || match selected_index.get() {
        Some(i) => format!("{} items, row {} selected", len.get(), i + 1),
        None => format!("{} items", len.get()),
    })
    .node_ref(list_ref)
    .on_wheel(move |w: WheelEvent| {
        scroll_to(offset.get_untracked() + w.delta.y);
        Handled::Yes
    })
    .on_key(move |k: KeyEvent| {
        if k.modifiers != Modifiers::default() {
            return Handled::No;
        }
        let cur = selected_index.get_untracked();
        let n = len.get_untracked();
        match k.key {
            Key::ArrowDown => move_selection(cur.map_or(0, |i| i + 1)),
            Key::ArrowUp => move_selection(cur.map_or(0, |i| i.saturating_sub(1))),
            Key::PageDown => move_selection(cur.map_or(0, |i| i + page())),
            Key::PageUp => move_selection(cur.map_or(0, |i| i.saturating_sub(page()))),
            Key::Home => move_selection(0),
            Key::End => move_selection(n.saturating_sub(1)),
            Key::Enter => match (cur.and_then(&key_act), activate) {
                (Some(k), Some(cb)) => cb.call(k),
                _ => return Handled::No,
            },
            _ => return Handled::No,
        }
        Handled::Yes
    })
    .on_pointer(move |e: PointerEvent| match e.phase {
        // Rows are not pointer listeners: their pointer events reach the list.
        PointerPhase::Down if e.click_count >= 2 && e.region().is_none() => {
            activate_selected();
            Handled::No
        }
        PointerPhase::Down if e.region() == Some(THUMB) => {
            dragging.set(Some((e.pos.y, offset.get_untracked())));
            Handled::Yes
        }
        PointerPhase::Move => match dragging.get() {
            Some((y0, off0)) => {
                let vh = viewport.get_untracked();
                let total = len.get_untracked() as f32 * h;
                let th = (vh * vh / total.max(1.0)).max(24.0);
                let per_px = (total - vh) / (vh - th).max(1.0);
                scroll_to(off0 + (e.pos.y - y0) * per_px);
                Handled::Yes
            }
            None => Handled::No,
        },
        PointerPhase::Up | PointerPhase::Cancel => {
            let was = dragging.get().is_some();
            dragging.set(None);
            if was { Handled::Yes } else { Handled::No }
        }
        _ => Handled::No,
    })
    .draw_over(move |d: &mut Draw, size: Size| {
        if let Some(t) = thumb(size) {
            d.fill_rect(
                Rect::new(size.width - BAR_W, 0.0, BAR_W, size.height),
                Color::token(ColorToken::SurfaceAlt).with_alpha(0.6),
            );
            d.rounded_rect(t, BAR_W / 2.0, ColorToken::TextMuted);
            d.region(THUMB, t);
        }
    })
    .child(content)
}
