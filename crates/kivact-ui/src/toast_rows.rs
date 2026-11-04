//! Role: Retain only toast row geometry while a dismissed card exits.
use kivact_core::control::{NodeBuilder, column};
use kivact_core::motion::{Transition, animated};
use kivact_core::style::{Overflow, style};
use kivact_core::view::{View, for_each, node_ref, show};
use kivact_core::{Value, effect, memo, on_mount, signal, var};
use kivact_interaction::ToastId;

#[derive(Clone, Copy, PartialEq)]
struct Slot {
    id: ToastId,
    generation: u64,
}

pub(super) fn animate(
    shown: Value<Vec<ToastId>>,
    transition: Transition,
    gap: f64,
    build: impl Fn(ToastId) -> View + 'static,
) -> View {
    let slots = signal(Vec::<Slot>::new());
    let live_slots = signal(Vec::<Slot>::new());
    let next_generation = var(0_u64);
    effect(move || {
        let current = live_slots.get_untracked();
        let mut added = Vec::new();
        let next = shown.with(|ids| {
            ids.iter()
                .map(|&id| {
                    current
                        .iter()
                        .find(|slot| slot.id == id)
                        .copied()
                        .unwrap_or_else(|| {
                            let generation = next_generation.get();
                            next_generation.set(generation + 1);
                            let slot = Slot { id, generation };
                            added.push(slot);
                            slot
                        })
                })
                .collect()
        });
        live_slots.set(next);
        if !added.is_empty() {
            slots.update(|slots| slots.extend(added));
        }
    });
    let build = std::rc::Rc::new(build);
    for_each(
        slots,
        |slot| slot.generation,
        move |generation, slot| {
            let slot = slot.get();
            // The current toaster value hides the real card in the same update
            // that removes it. The presentation generation keeps a reappearing
            // ID independent of its old exiting shell.
            let live = memo(move || {
                shown.with(|ids| ids.contains(&slot.id))
                    && live_slots.with(|slots| slots.contains(&slot))
            });
            let mounted = signal(false);
            on_mount(move || mounted.set(true));
            let progress = animated(
                move || {
                    if mounted.get() && live.get() {
                        1.0_f64
                    } else {
                        0.0
                    }
                },
                transition.clone(),
            );
            let measured = node_ref();
            let rect = measured.rect();
            let height = signal(0.0_f64);
            effect(move || {
                if live.get()
                    && let Some(rect) = rect.get()
                {
                    height.set(rect.height);
                }
            });
            effect(move || {
                if !live.get() && progress.get() <= 0.0 {
                    slots.update(|slots| slots.retain(|slot| slot.generation != generation));
                }
            });
            let build = build.clone();
            // The card and its item owner unmount immediately. The host retains
            // only its inert exit paint while this outer box animates to zero.
            let card = show(live, move || build(slot.id));
            column(column(card).node_ref(measured).style(style().shrink(0))).style(move || {
                let share = progress.get().clamp(0.0, 1.0);
                let base = style().shrink(0).padding_top(gap * share);
                if share < 1.0 || !live.get() {
                    base.height((height.get() + gap) * share)
                        .overflow(Overflow::Hidden)
                } else {
                    base
                }
            })
        },
    )
}
