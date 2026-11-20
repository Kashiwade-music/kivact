//! Role: The per-kind tables of the expansion: the field, the argument, the
//! conversion, the generic and the scope conversion of each prop kind.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::parse::{Prop, PropKind, upper_camel};

/// The marker generic of a `Callback<()>` prop (`IntoHandler`'s `M`).
fn marker(prop: &Prop) -> Ident {
    format_ident!(
        "__KvM{}",
        upper_camel(&prop.name.to_string()),
        span = prop.name.span()
    )
}

/// The result generic of a `Callback<E>` prop (`Fn(E) -> R`).
fn result(prop: &Prop) -> Ident {
    format_ident!(
        "__KvR{}",
        upper_camel(&prop.name.to_string()),
        span = prop.name.span()
    )
}

/// The view generic of a `ViewFn` prop (`IntoViewFn<V>`).
fn view(prop: &Prop) -> Ident {
    format_ident!(
        "__KvV{}",
        upper_camel(&prop.name.to_string()),
        span = prop.name.span()
    )
}

/// The generic a prop adds to its constructor or setter, with its bound.
pub(super) fn generic_decl(prop: &Prop, r: &TokenStream) -> Option<TokenStream> {
    match &prop.kind {
        PropKind::Callback0 => {
            let marker = marker(prop);
            Some(quote! { #marker })
        }
        PropKind::Callback(_) => {
            let result = result(prop);
            Some(quote! { #result: #r::view::HandlerResult })
        }
        PropKind::ViewFn => {
            let view = view(prop);
            Some(quote! { #view: #r::view::IntoView })
        }
        _ => None,
    }
}

/// The type a constructor or setter takes for the prop.
pub(super) fn input_type(prop: &Prop, r: &TokenStream) -> TokenStream {
    match &prop.kind {
        PropKind::Text => quote! { impl #r::state::IntoText },
        PropKind::Value(ty) => quote! { impl #r::state::IntoValue<#ty> },
        PropKind::Signal(ty) => quote! { #r::state::Signal<#ty> },
        PropKind::Callback0 => {
            let marker = marker(prop);
            quote! { impl #r::state::IntoHandler<(), #marker> }
        }
        PropKind::Callback(ty) => {
            let result = result(prop);
            quote! { impl ::std::ops::Fn(#ty) -> #result + 'static }
        }
        PropKind::View => quote! { impl #r::view::IntoView },
        PropKind::Children => quote! { impl #r::view::IntoChildren },
        PropKind::ViewFn => {
            let view = view(prop);
            quote! { impl #r::view::IntoViewFn<#view> }
        }
        PropKind::Param(ty) => quote! { #ty },
        PropKind::Other(ty) => quote! { impl ::std::convert::Into<#ty> },
    }
}

/// The type the struct keeps the prop as.
pub(super) fn keep_type(prop: &Prop, r: &TokenStream) -> TokenStream {
    match &prop.kind {
        PropKind::Text => quote! { #r::custom::Prop<::std::string::String> },
        PropKind::Value(ty) => quote! { #r::custom::Prop<#ty> },
        PropKind::Signal(ty) => quote! { #r::state::Signal<#ty> },
        PropKind::Callback0 => quote! { #r::custom::Handler<()> },
        PropKind::Callback(ty) => quote! { #r::custom::Handler<#ty> },
        PropKind::View => quote! { #r::view::View },
        PropKind::Children => quote! { ::std::vec::Vec<#r::view::View> },
        PropKind::ViewFn => quote! { #r::view::ViewFn<()> },
        PropKind::Param(ty) | PropKind::Other(ty) => quote! { #ty },
    }
}

/// The conversion a caller's argument goes through when it is stored.
pub(super) fn store(prop: &Prop, r: &TokenStream, input: TokenStream) -> TokenStream {
    match &prop.kind {
        PropKind::Text => quote! { #r::custom::Prop::from_text(#input) },
        PropKind::Value(_) => quote! { #r::custom::Prop::new(#input) },
        PropKind::Signal(_) | PropKind::Param(_) => input,
        PropKind::Callback0 => quote! { #r::state::IntoHandler::into_handler(#input) },
        PropKind::Callback(_) => quote! { #r::custom::Handler::from_fn(#input) },
        PropKind::View => quote! { #r::view::IntoView::into_view(#input) },
        PropKind::Children => quote! { #r::view::IntoChildren::into_views(#input) },
        PropKind::ViewFn => quote! { #r::view::IntoViewFn::into_view_fn(#input) },
        PropKind::Other(_) => quote! { ::std::convert::Into::into(#input) },
    }
}

/// The type the body sees a prop as, and the conversion to it (the name is
/// the variable holding the prop). `None` when the body sees the prop as it is.
fn scope_conversion(prop: &Prop, r: &TokenStream) -> Option<(TokenStream, TokenStream)> {
    let name = &prop.name;
    match &prop.kind {
        PropKind::Text => Some((
            quote! { #r::state::Value<::std::string::String> },
            quote! { #r::state::IntoValue::into_value(#name) },
        )),
        PropKind::Value(ty) => Some((
            quote! { #r::state::Value<#ty> },
            quote! { #r::state::IntoValue::into_value(#name) },
        )),
        PropKind::Callback0 => Some((
            quote! { #r::state::Callback<()> },
            quote! { #r::state::Callback::from_handler(#name) },
        )),
        PropKind::Callback(ty) => Some((
            quote! { #r::state::Callback<#ty> },
            quote! { #r::state::Callback::from_handler(#name) },
        )),
        _ => None,
    }
}

/// The statement that turns a prop into the form the body reads, inside the
/// scope: once, before the body's statements.
pub(super) fn mount(prop: &Prop, r: &TokenStream) -> Option<TokenStream> {
    let name = &prop.name;
    let (target, conversion) = scope_conversion(prop, r)?;
    if prop.is_optional() {
        Some(quote! {
            let #name: ::std::option::Option<#target> = #name.map(|#name| #conversion);
        })
    } else {
        Some(quote! { let #name: #target = #conversion; })
    }
}

/// The kivact type the body reads a recognised prop as, which API-18 names.
pub(super) fn kivact_type(prop: &Prop, r: &TokenStream) -> Option<TokenStream> {
    match &prop.kind {
        PropKind::Text => Some(quote! { #r::state::Value<::std::string::String> }),
        PropKind::Value(ty) => Some(quote! { #r::state::Value<#ty> }),
        PropKind::Signal(ty) => Some(quote! { #r::state::Signal<#ty> }),
        PropKind::Callback0 => Some(quote! { #r::state::Callback<()> }),
        PropKind::Callback(ty) => Some(quote! { #r::state::Callback<#ty> }),
        PropKind::View => Some(quote! { #r::view::View }),
        PropKind::Children => Some(quote! { ::std::vec::Vec<#r::view::View> }),
        PropKind::ViewFn => Some(quote! { #r::view::ViewFn }),
        PropKind::Param(_) | PropKind::Other(_) => None,
    }
}
