//! Role: The list of controls the button group accepts: a tuple of builders or a `Vec`.

use kivact::control::NodeBuilder;
use kivact::style::Style;
use kivact::view::{IntoValue, IntoView, View};

/// Several controls for a component that gives each one a style layer: a
/// tuple of builders (mixed types, up to 16) or a `Vec` of one kind of
/// builder.
///
/// [`button_group`](crate::button_group) takes this, because the children
/// after the first drop the border they share with the one before.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a list of controls",
    label = "not a tuple or a `Vec` of builders",
    note = "pass a tuple of up to 16 builders such as `button(..)`, `button_group_separator()` and `button_group_text(..)`, or a `Vec` of one kind"
)]
pub trait GroupItems: 'static {
    /// Each item with the style layer `style(index)` added after its own.
    fn into_styled_views<S: IntoValue<Style>>(self, style: impl Fn(usize) -> S) -> Vec<View>;
}

impl<V: NodeBuilder + IntoView> GroupItems for Vec<V> {
    fn into_styled_views<S: IntoValue<Style>>(self, style: impl Fn(usize) -> S) -> Vec<View> {
        self.into_iter()
            .enumerate()
            .map(|(index, item)| item.style(style(index)).into_view())
            .collect()
    }
}

macro_rules! tuple_children {
    ($($name:ident),+) => {
        #[diagnostic::do_not_recommend]
        impl<$($name: NodeBuilder + IntoView),+> GroupItems for ($($name,)+) {
            #[allow(
                non_snake_case,
                unused_assignments,
                reason = "tuple element bindings reuse the type parameter names; the last index step is unused"
            )]
            fn into_styled_views<S: IntoValue<Style>>(
                self,
                style: impl Fn(usize) -> S,
            ) -> Vec<View> {
                let ($($name,)+) = self;
                let mut index = 0;
                vec![$({
                    let view = $name.style(style(index)).into_view();
                    index += 1;
                    view
                }),+]
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
