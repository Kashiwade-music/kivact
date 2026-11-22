//! Role: The `#[component]` attribute: turns a component function into its props builder.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

mod expand;
mod parse;
mod root;

/// Turn a component function into its props builder (see the guide "Write reusable components").
///
/// Required arguments become the builder's positional arguments; `Option<..>`
/// arguments and `#[prop(default = ..)]` arguments become named setters on the
/// returned struct. Integer props (`u8` to `isize`) take their written type,
/// so a literal such as `3` passes as it is. Return `impl Element` to let
/// callers style the component.
/// The function body runs once, in the scope the builder mounts.
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
            Ok(model) => match expand::component(&model) {
                Ok(tokens) => tokens,
                Err(error) => error.to_compile_error(),
            },
            Err(error) => error.to_compile_error(),
        },
        Err(error) => error.to_compile_error(),
    };
    expanded.into()
}

/// Let a type of your own be a prop constant: `#[derive(Clone, PartialEq, Prop)]`.
/// It implements `IntoValue<Self>` for the type, so `badge("Saved", Tone::Info)` compiles.
#[proc_macro_derive(Prop)]
pub fn derive_prop(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let root = match root::root() {
        Ok(root) => root,
        Err(error) => return error.to_compile_error().into(),
    };
    let name = &input.ident;
    // Keep the type's own where-clause and add `Self: 'static` (a prop value
    // lives in the App's arena; 12 章 API-6).
    let mut generics = input.generics.clone();
    generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote!(Self: 'static));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics #root::state::IntoValue<#name #ty_generics> for #name #ty_generics #where_clause {
            fn into_prop(self) -> #root::custom::Prop<#name #ty_generics> {
                #root::custom::Prop::constant(self)
            }
        }
    }
    .into()
}
