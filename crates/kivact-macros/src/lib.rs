//! Role: The `#[component]` attribute: turn a component function into its props builder.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

mod expand;
mod parse;

/// Turn a component function into its props builder (see the guide "Write reusable components").
///
/// Required arguments become the builder's positional arguments; arguments
/// marked `#[prop(optional)]` or `#[prop(default = ..)]` become named setters
/// on the returned struct. The function body runs once, in the scope the
/// builder mounts.
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    if !attr.is_empty() {
        let error = syn::Error::new(Span::call_site(), "#[component] takes no arguments")
            .to_compile_error();
        return quote! {
            #error
            #item
        }
        .into();
    }
    let expanded = match syn::parse2::<syn::ItemFn>(item) {
        Ok(function) => match parse::component(function) {
            Ok(model) => expand::component(&model),
            Err(error) => error.to_compile_error(),
        },
        Err(error) => error.to_compile_error(),
    };
    expanded.into()
}
