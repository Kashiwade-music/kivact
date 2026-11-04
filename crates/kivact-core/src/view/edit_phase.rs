//! Role: The phases reported by a continuous value edit.

/// The phase of one continuous edit that a component reports to the app.
///
/// A drag, or one key press, writes the value many times or once; an app that
/// records undo steps wants those writes as a single step. A component that
/// takes `on_edit(f)` calls `f` with these phases around its writes, the same
/// way for every kit. They play the role of `onPointerDown` and `onPointerUp`
/// in a React handler that opens and closes an undo transaction, and also cover
/// keys and assistive technology, which have no pointer events.
///
/// The rules are the same for every component:
///
/// - `Begin` comes before the first write of the edit to the binding. Take the
///   snapshot for the undo step here.
/// - `Commit` comes after the last write. Close the undo step here.
/// - `Cancel` comes after the component wrote the value the edit started
///   from back to the binding. Drop the undo step here; the binding already
///   holds the old value.
/// - A `Commit` or a `Cancel` is sent only for an edit that sent `Begin`, and
///   an edit sends `Begin` once and then exactly one of `Commit` and `Cancel`.
/// - If the component is disposed in the middle of an edit (its owner is
///   cleaned up), the edit ends as `Cancel`.
///
/// A drag is one edit from the press to the release. A key press, a wheel
/// notch or an assistive technology action that writes once is its own edit:
/// `Begin`, the write, `Commit`.
///
/// ```no_run
/// # mod kivact {
/// #     //! Role: Supply facade paths for this documentation example.
/// #     pub use kivact_core::{view, state, style};
/// #     pub extern crate kivact_interaction as interaction;
/// # }
/// use kivact::interaction::{EditPhase, ScalarRange, use_splitter};
/// use kivact::custom::{part};
/// use kivact::state::{signal};
/// use kivact::view::{NodeBuilder, Role, node};
/// let width = signal(240.0);
/// let started_at = signal(240.0);
/// let undo = signal(Vec::<(f64, f64)>::new());
/// let on_edit = move |phase| match phase {
///     EditPhase::Begin => started_at.set(width.get_untracked()),
///     EditPhase::Commit => {
///         undo.update(|steps| steps.push((started_at.get_untracked(), width.get_untracked())))
///     }
///     EditPhase::Cancel => {}
/// };
/// let range = ScalarRange::new(120.0, 480.0).unwrap();
/// let handle = use_splitter(width, range, Orientation::Horizontal)
///     .on_edit(on_edit)
///     .handle();
/// let view = node().part(handle).name("Sidebar width");
/// # let _ = view;
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditPhase {
    /// The edit starts; no write has happened yet.
    Begin,
    /// The edit ended with its writes kept.
    Commit,
    /// The edit was abandoned and the value it started from is back in the binding.
    Cancel,
}
