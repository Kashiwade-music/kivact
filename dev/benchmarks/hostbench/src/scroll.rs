//! Role: Scroll performance benchmarks.
//!
//! Scrolling a long list every frame: a 10,000-row `virtual_list` or a
//! plain 2,000-row column. The scroll is programmatic: each frame asks the
//! host to reveal the row two rows further down
//! (`NodeRef::scroll_into_view`, about 48 px per frame), so the host
//! scrolls, reports the new offset and, for the virtual list, the core
//! mounts the rows coming into view.

use kivact::view::View;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use kivact::prelude::*;
use kivact::style::Align;
use kivact::view::RowHeight;

use crate::report::{Json, mark, stats};
use crate::scenes::{SETTLE, every_frame, paragraph_text};

const ROW_HEIGHT: f32 = 24.0;
/// Rows advanced per frame.
const STEP: usize = 2;
/// Measured frames (5 s at 60 Hz).
const MEASURED: u64 = 300;

type Refs = Rc<RefCell<HashMap<usize, (u64, NodeRef)>>>;

fn list_row(i: usize, refs: &Refs, generation: &Rc<Cell<u64>>) -> Node {
    let r = node_ref();
    let g = generation.get() + 1;
    generation.set(g);
    refs.borrow_mut().insert(i, (g, r));
    let refs = refs.clone();
    on_cleanup(move || {
        let mut refs = refs.borrow_mut();
        if refs.get(&i).is_some_and(|(gen_, _)| *gen_ == g) {
            refs.remove(&i);
        }
    });
    let shade = if i.is_multiple_of(2) {
        0xffffff
    } else {
        0xf2f4f7
    };
    row((
        text(format!("{i:05}")).style(style().width(56)),
        text(paragraph_text(i).chars().take(60).collect::<String>()).style(style().grow(1)),
        text(format!("{} KB", (i * 37) % 1000)).style(style().width(64)),
    ))
    .node_ref(r)
    .style(
        style()
            .height(ROW_HEIGHT)
            .shrink(0)
            .padding_x(8)
            .gap(12)
            .align_items(Align::Center)
            .background(rgb(shade)),
    )
}

pub(crate) fn scroll_list(virtualized: bool) -> View {
    let (name, rows) = if virtualized {
        ("scroll_virtual", 10_000usize)
    } else {
        ("scroll_column", 2_000)
    };
    let refs: Refs = Rc::default();
    let generation = Rc::new(Cell::new(0u64));
    let items: Vec<usize> = (0..rows).collect();
    let (r, g) = (refs.clone(), generation.clone());
    let row_view = move |i: usize, _: Value<usize>| list_row(i, &r, &g);
    let list = if virtualized {
        virtual_list(items, |i: &usize| *i, row_view)
            .row_height(RowHeight::Fixed(ROW_HEIGHT))
            .style(style().grow(1).min_height(0))
            .into_view()
    } else {
        scroll(for_index(items, row_view))
            .style(style().grow(1).min_height(0))
            .into_view()
    };
    let begin = Cell::new(0u128);
    let target = Cell::new(0usize);
    let misses = Cell::new(0u32);
    let ticks = RefCell::new(Vec::new());
    every_frame(move |tick, info| {
        if tick < SETTLE {
            return true;
        }
        if tick == SETTLE {
            begin.set(mark("phase_begin"));
            // About the first row below a 720 px window.
            target.set(30);
        }
        if tick < SETTLE + MEASURED {
            let next = (target.get() + STEP).min(rows - 1);
            target.set(next);
            let found = refs.borrow().get(&next).map(|(_, r)| *r);
            match found {
                Some(r) => r.scroll_into_view(),
                None => misses.set(misses.get() + 1),
            }
            if tick > SETTLE {
                ticks.borrow_mut().push(info.delta.as_secs_f64() * 1e3);
            }
            return true;
        }
        let end = mark("phase_end");
        Json::new(name)
            .int("rows", rows as u128)
            .frames(begin.get(), end)
            .int("last_row", target.get() as u128)
            .int("missed_rows", u128::from(misses.get()))
            .stats("tick_ms", stats(ticks.take()))
            .finish();
        false
    });
    column((text(format!("{name}: {rows} rows")), list))
        .style(style().padding(8).gap(4).grow(1).min_height(0))
        .into_view()
}
