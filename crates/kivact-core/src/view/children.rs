//! Role: The child lists the group components accept: a tuple of views or a `Vec`.

use super::{IntoView, View};

/// Several children for a component that wraps or styles each one: a tuple
/// of views (mixed types, up to 16) or a `Vec` of one kind of view, like
/// `column((..))`.
///
/// A component that takes one `impl IntoView` cannot tell its children
/// apart; the split component takes this instead, because each pane gets
/// its own wrapper and a handle between neighbours.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a list of children",
    label = "not a tuple of views or a `Vec` of views",
    note = "pass a tuple of up to 16 views, `(a, b, c)`, or a `Vec` of views, such as `items.map(..).collect::<Vec<_>>()`"
)]
pub trait IntoChildren: 'static {
    /// Each child as a view, in order.
    fn into_views(self) -> Vec<View>;
}

impl<V: IntoView> IntoChildren for Vec<V> {
    fn into_views(self) -> Vec<View> {
        self.into_iter().map(IntoView::into_view).collect()
    }
}

macro_rules! tuple_children {
    ($($name:ident),+) => {
        #[diagnostic::do_not_recommend]
        impl<$($name: IntoView),+> IntoChildren for ($($name,)+) {
            #[allow(non_snake_case, reason = "tuple element bindings reuse the type parameter names")]
            fn into_views(self) -> Vec<View> {
                let ($($name,)+) = self;
                vec![$($name.into_view()),+]
            }
        }
    };
}

tuple_children!(A);
tuple_children!(A, B);
tuple_children!(A, B, C);
tuple_children!(A, B, C, D);
tuple_children!(A, B, C, D, E);
tuple_children!(A, B, C, D, E, F);
tuple_children!(A, B, C, D, E, F, G);
tuple_children!(A, B, C, D, E, F, G, H);
tuple_children!(A, B, C, D, E, F, G, H, I);
tuple_children!(A, B, C, D, E, F, G, H, I, J);
tuple_children!(A, B, C, D, E, F, G, H, I, J, K);
tuple_children!(A, B, C, D, E, F, G, H, I, J, K, L);
tuple_children!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuple_children!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuple_children!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tuple_children!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
