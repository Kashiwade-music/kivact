//! Role: One ordered handler list per event kind, delivered layer by layer.
//!
//! Delivery order, for one event, is by layer and then by position in the
//! list. A layer belongs to exactly one place in that order:
//!
//! 1. [`Layer::Observer`]: part state observations. They run first, on the
//!    selected target, even when a later layer consumes the event or fails,
//!    and cannot consume it themselves.
//! 2. [`Layer::Caller`]: the application's `on_*` handlers, the one
//!    registered last first.
//! 3. [`Layer::NodeDefault`]: the node's own `on_*_default` handlers, in
//!    registration order. Like callers they belong to the node: they see every
//!    event that reaches it, a canvas region's included, whatever the
//!    target's state.
//! 4. [`Layer::PartDefault`]: the defaults of attached parts, in attachment
//!    order, on the selected target (the node, or the canvas region the
//!    event carries).
//! 5. [`Layer::RegionDefault`]: the node's fallback for region input, from
//!    parts that opted in.
//!
//! `Handled::Yes` stops the later layers, `No` continues and
//! `PreventDefault` continues and is kept (`Handled::and`). An `Err` stops
//! and goes to the caller of [`deliver`].

use std::rc::Rc;

use crate::Result;
use crate::view::Handled;

/// A place in the delivery order (declared in delivery order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Layer {
    Observer,
    Caller,
    NodeDefault,
    PartDefault,
    RegionDefault,
}

impl Layer {
    /// Every layer, in delivery order.
    pub(crate) const ALL: [Layer; 5] = [
        Layer::Observer,
        Layer::Caller,
        Layer::NodeDefault,
        Layer::PartDefault,
        Layer::RegionDefault,
    ];
    /// The layers that belong to the node itself.
    pub(crate) const NODE: [Layer; 2] = [Layer::Caller, Layer::NodeDefault];
}

use kivact_runtime::Handler;

/// The handlers of one event kind, ordered by layer. Within a layer the
/// order is the delivery order. Cloning shares the list.
pub(crate) struct EventHandlers<E> {
    /// `None`: no handler (nothing allocated).
    entries: Option<Rc<Vec<(Layer, Handler<E>)>>>,
}

impl<E> Default for EventHandlers<E> {
    fn default() -> Self {
        Self { entries: None }
    }
}

impl<E> Clone for EventHandlers<E> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<E> EventHandlers<E> {
    fn list(&self) -> &[(Layer, Handler<E>)] {
        self.entries.as_deref().map_or(&[], Vec::as_slice)
    }
}

impl<E: 'static> EventHandlers<E> {
    fn list_mut(&mut self) -> &mut Vec<(Layer, Handler<E>)> {
        Rc::make_mut(self.entries.get_or_insert_with(Rc::default))
    }

    /// Add `handler` at the end of its layer.
    fn register(&mut self, layer: Layer, handler: Handler<E>) {
        let at = self
            .list()
            .partition_point(|(current, _)| *current <= layer);
        self.list_mut().insert(at, (layer, handler));
    }

    /// Merge a later caller segment without reclassifying handler layers.
    pub(crate) fn merge_caller(&mut self, later: Self) {
        for (layer, handler) in later.list().iter().rev() {
            if *layer == Layer::Caller {
                let at = self
                    .list()
                    .partition_point(|(current, _)| *current < *layer);
                self.list_mut().insert(at, (*layer, handler.clone()));
            }
        }
        for (layer, handler) in later.list().iter() {
            if *layer != Layer::Caller {
                self.register(*layer, handler.clone());
            }
        }
    }
    /// A state observation, after the observations already registered.
    pub(crate) fn observer(&mut self, f: impl Fn(E) + 'static) {
        self.register(
            Layer::Observer,
            Handler::from_fn(move |event| {
                f(event);
                Ok(Handled::No)
            }),
        );
    }

    /// `on_*`: before the callers already registered.
    pub(crate) fn caller(&mut self, f: impl Fn(E) -> Result<Handled> + 'static) {
        let at = self
            .list()
            .partition_point(|(current, _)| *current < Layer::Caller);
        self.list_mut()
            .insert(at, (Layer::Caller, Handler::from_fn(f)));
    }

    /// A kind with one caller handler (`on_file`): a later one replaces it.
    pub(crate) fn replace_caller(&mut self, f: impl Fn(E) -> Result<Handled> + 'static) {
        self.list_mut().retain(|(layer, _)| *layer != Layer::Caller);
        self.caller(f);
    }

    /// `on_*_default`: after the node's earlier defaults.
    pub(crate) fn node_default(&mut self, f: impl Fn(E) -> Result<Handled> + 'static) {
        self.register(Layer::NodeDefault, Handler::from_fn(f));
    }

    /// A list whose only entry is the router a mounted node installs.
    pub(crate) fn router(f: impl Fn(E) -> Result<Handled> + 'static) -> Self {
        Self {
            entries: Some(Rc::new(vec![(Layer::Caller, Handler::from_fn(f))])),
        }
    }

    /// Attach a part's handlers after what is here: its observers stay
    /// observers, its own handlers and the defaults of the parts attached to
    /// it become part defaults, and (with `region_events`) its own handlers
    /// also become region defaults.
    pub(crate) fn absorb_part(&mut self, part: Self, region_events: bool) {
        if region_events {
            for (layer, handler) in part.list().iter() {
                if *layer == Layer::Caller {
                    self.register(Layer::RegionDefault, handler.clone());
                }
            }
        }
        for (layer, handler) in part.list().iter() {
            let layer = match layer {
                Layer::Observer => Layer::Observer,
                Layer::Caller | Layer::NodeDefault | Layer::PartDefault => Layer::PartDefault,
                Layer::RegionDefault => Layer::RegionDefault,
            };
            self.register(layer, handler.clone());
        }
    }

    /// The entries of `layers` only.
    pub(crate) fn only(&self, layers: &[Layer]) -> Self {
        let entries: Vec<_> = self
            .list()
            .iter()
            .filter(|(layer, _)| layers.contains(layer))
            .cloned()
            .collect();
        Self {
            entries: (!entries.is_empty()).then(|| Rc::new(entries)),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.list().is_empty()
    }

    /// A copy for dispatch, if anything is registered.
    pub(crate) fn if_set(&self) -> Option<Self> {
        (!self.is_empty()).then(|| self.clone())
    }
}

impl<E: Clone + 'static> EventHandlers<E> {
    /// Deliver `event` through every layer of this list alone.
    pub(crate) fn deliver(&self, event: E) -> Result<Handled> {
        deliver(&[Source::all(self)], &event)
    }
}

/// Always admitted.
pub(crate) fn open() -> bool {
    true
}

/// One list that takes part in a delivery: the layers of it that do, and
/// whether they run now. `admit` is asked once, when delivery reaches each
/// of those layers, so a handler that changes a target's state is seen by
/// the layers after it.
pub(crate) struct Source<'a, E> {
    list: &'a EventHandlers<E>,
    layers: &'a [Layer],
    admit: &'a dyn Fn() -> bool,
}

impl<'a, E> Source<'a, E> {
    pub(crate) fn new(
        list: &'a EventHandlers<E>,
        layers: &'a [Layer],
        admit: &'a dyn Fn() -> bool,
    ) -> Self {
        Self {
            list,
            layers,
            admit,
        }
    }

    /// Every layer of `list`, always admitted.
    pub(crate) fn all(list: &'a EventHandlers<E>) -> Self {
        Self::new(list, &Layer::ALL, &open)
    }
}

/// Deliver `event` to the `sources` layer by layer, in delivery order; within
/// a layer, source by source in the order given.
pub(crate) fn deliver<E: Clone + 'static>(sources: &[Source<'_, E>], event: &E) -> Result<Handled> {
    let mut result = Handled::No;
    for layer in Layer::ALL {
        for source in sources {
            if !source.layers.contains(&layer) || !(source.admit)() {
                continue;
            }
            for (current, handler) in source.list.list().iter() {
                if *current != layer {
                    continue;
                }
                result = result.and(handler.call(event.clone())?);
                if result == Handled::Yes {
                    return Ok(Handled::Yes);
                }
            }
        }
    }
    Ok(result)
}
