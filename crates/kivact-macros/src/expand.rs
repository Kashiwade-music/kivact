//! Role: Write the props builder of a component function.

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use crate::parse::{Model, Prop, PropKind, Required, upper_camel};

/// The struct, its constructor and setters, and its `IntoView` impl.
pub(crate) fn component(model: &Model) -> TokenStream {
    let struct_name = &model.struct_name;
    let vis = &model.vis;
    let docs = &model.docs;
    let fields = model.props.iter().map(field);
    let constructor = constructor(model);
    let setters = model
        .props
        .iter()
        .filter(|prop| !prop.is_positional())
        .map(setter);
    let into_view = into_view(model);
    quote! {
        #(#docs)*
        #[must_use]
        #vis struct #struct_name {
            #(#fields),*
        }

        #constructor

        impl #struct_name {
            #(#setters)*
        }

        #into_view
    }
}

fn field(prop: &Prop) -> TokenStream {
    let name = &prop.name;
    let ty = keep_type(prop);
    if prop.is_optional() {
        quote! { #name: ::std::option::Option<#ty> }
    } else {
        quote! { #name: #ty }
    }
}

fn constructor(model: &Model) -> TokenStream {
    let (vis, name, struct_name, docs) = (&model.vis, &model.name, &model.struct_name, &model.docs);
    let positional: Vec<&Prop> = model
        .props
        .iter()
        .filter(|prop| prop.is_positional())
        .collect();
    let generics = generics(positional.iter().filter_map(|prop| generic(prop)).collect());
    let params = positional.iter().map(|prop| {
        let name = &prop.name;
        let ty = input_type(prop);
        quote! { #name: #ty }
    });
    let fields = model.props.iter().map(initializer);
    quote! {
        #(#docs)*
        #[track_caller]
        #vis fn #name #generics (#(#params),*) -> #struct_name {
            #struct_name { #(#fields),* }
        }
    }
}

fn setter(prop: &Prop) -> TokenStream {
    let name = &prop.name;
    let ty = keep_type(prop);
    let value = store(prop, quote! { #name });
    let input = input_type(prop);
    let generics = generics(generic(prop).into_iter().collect());
    let assign = if prop.is_optional() {
        quote! { self.#name = ::std::option::Option::Some(#name); }
    } else {
        quote! { self.#name = #name; }
    };
    quote! {
        #[track_caller]
        pub fn #name #generics (mut self, #name: #input) -> Self {
            let #name: #ty = #value;
            #assign
            self
        }
    }
}

fn into_view(model: &Model) -> TokenStream {
    let struct_name = &model.struct_name;
    let names = model.props.iter().map(|prop| &prop.name);
    let mounts = model.props.iter().filter_map(mount);
    let statements = &model.body.stmts;
    quote! {
        impl ::kivact::view::IntoView for #struct_name {
            fn into_view(self) -> ::kivact::view::View {
                let #struct_name { #(#names),* } = self;
                ::kivact::view::component(move || {
                    #(#mounts)*
                    #(#statements)*
                })
            }
        }
    }
}

fn initializer(prop: &Prop) -> TokenStream {
    let name = &prop.name;
    match &prop.required {
        Required::Positional => {
            let value = store(prop, quote! { #name });
            quote! { #name: #value }
        }
        Required::Optional => quote! { #name: ::std::option::Option::None },
        Required::Default(expr) => {
            let value = store(prop, quote! { #expr });
            quote! { #name: #value }
        }
    }
}

fn mount(prop: &Prop) -> Option<TokenStream> {
    let name = &prop.name;
    match &prop.kind {
        PropKind::Text | PropKind::Value(_) => {
            Some(quote! { let #name = ::kivact::state::IntoValue::into_value(#name); })
        }
        PropKind::Callback0 | PropKind::Callback(_) if prop.is_optional() => {
            Some(quote! { let #name = #name.map(|f| f()); })
        }
        PropKind::Callback0 | PropKind::Callback(_) => Some(quote! { let #name = #name(); }),
        _ => None,
    }
}

fn generics(params: Vec<TokenStream>) -> TokenStream {
    if params.is_empty() {
        TokenStream::new()
    } else {
        quote! { <#(#params),*> }
    }
}

fn generic(prop: &Prop) -> Option<TokenStream> {
    match &prop.kind {
        PropKind::Callback(_) => {
            let result = r_ident(prop);
            Some(quote! { #result: ::kivact::view::HandlerResult })
        }
        PropKind::ViewFn => {
            let view = v_ident(prop);
            Some(quote! { #view: ::kivact::view::IntoView })
        }
        _ => None,
    }
}

fn r_ident(prop: &Prop) -> Ident {
    format_ident!(
        "R{}",
        upper_camel(&prop.name.to_string()),
        span = prop.name.span()
    )
}

fn v_ident(prop: &Prop) -> Ident {
    format_ident!(
        "V{}",
        upper_camel(&prop.name.to_string()),
        span = prop.name.span()
    )
}

fn input_type(prop: &Prop) -> TokenStream {
    match &prop.kind {
        PropKind::Text => quote! { impl ::kivact::state::IntoText },
        PropKind::Value(ty) => quote! { impl ::kivact::state::IntoValue<#ty> },
        PropKind::Binding(ty) => quote! { impl ::kivact::state::IntoBinding<#ty> },
        PropKind::Callback0 => {
            quote! { impl ::std::convert::Into<::kivact::state::Callback<()>> + 'static }
        }
        PropKind::Callback(ty) => {
            let result = r_ident(prop);
            quote! { impl ::std::ops::Fn(#ty) -> #result + 'static }
        }
        PropKind::View => quote! { impl ::kivact::view::IntoView },
        PropKind::Children => quote! { impl ::kivact::view::IntoChildren },
        PropKind::ViewFn => {
            let view = v_ident(prop);
            quote! { impl ::kivact::view::IntoViewFn<#view> }
        }
        PropKind::Other(ty) => quote! { impl ::std::convert::Into<#ty> },
    }
}

fn keep_type(prop: &Prop) -> TokenStream {
    match &prop.kind {
        PropKind::Text => quote! { ::kivact::custom::Reactive<String> },
        PropKind::Value(ty) => quote! { ::kivact::custom::Reactive<#ty> },
        PropKind::Binding(ty) => quote! { ::kivact::state::Binding<#ty> },
        PropKind::Callback0 => quote! {
            ::std::boxed::Box<dyn ::std::ops::FnOnce() -> ::kivact::state::Callback<()>>
        },
        PropKind::Callback(ty) => quote! {
            ::std::boxed::Box<dyn ::std::ops::FnOnce() -> ::kivact::state::Callback<#ty>>
        },
        PropKind::View => quote! { ::kivact::view::View },
        PropKind::Children => quote! { ::std::vec::Vec<::kivact::view::View> },
        PropKind::ViewFn => quote! { ::kivact::view::ViewFn<()> },
        PropKind::Other(ty) => quote! { #ty },
    }
}

/// The conversion a caller's argument goes through when it is stored.
fn store(prop: &Prop, input: TokenStream) -> TokenStream {
    match &prop.kind {
        PropKind::Text => quote! { ::kivact::custom::Reactive::from_text(#input) },
        PropKind::Value(_) => quote! { ::kivact::custom::Reactive::new(#input) },
        PropKind::Binding(_) => quote! { ::kivact::state::IntoBinding::into_binding(#input) },
        PropKind::Callback0 => quote! {
            ::std::boxed::Box::new(move || ::std::convert::Into::into(#input))
        },
        PropKind::Callback(_) => quote! {
            ::std::boxed::Box::new(move || ::kivact::state::callback(#input))
        },
        PropKind::View => quote! { ::kivact::view::IntoView::into_view(#input) },
        PropKind::Children => quote! { ::kivact::view::IntoChildren::into_views(#input) },
        PropKind::ViewFn => quote! { ::kivact::view::IntoViewFn::into_view_fn(#input) },
        PropKind::Other(_) => quote! { ::std::convert::Into::into(#input) },
    }
}
