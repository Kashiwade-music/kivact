//! Role: Write the props builder of a component function.

use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::{Model, Prop, PropKind, Required, Returns};

mod kinds;

use kinds::{generic_decl, input_type, keep_type, kivact_type, mount, store};

/// The struct, its constructor and setters, and its `Element` (or
/// `IntoView`) impls, all named by the path the crate resolves to.
pub(crate) fn component(model: &Model) -> syn::Result<TokenStream> {
    let r = crate::root::root()?;
    let (impl_generics, ty_generics, where_clause) = model.generics.split_for_impl();
    let struct_name = &model.struct_name;
    let vis = &model.vis;
    let docs = &model.docs;
    let attrs = &model.attrs;
    let fields = struct_fields(model, &r);
    let constructor = constructor(model, &r);
    let setters = model
        .props
        .iter()
        .filter(|prop| !prop.is_positional())
        .map(|prop| setter(prop, &r));
    let into_view = into_view(model, &r);
    let element = element_impl(model, &r);
    // A component that returns `impl IntoView` has no `Element` impl, so no
    // attributes either (they would land on the next item twice).
    let element = if element.is_empty() {
        element
    } else {
        quote! { #(#attrs)* #element }
    };
    Ok(quote! {
        #(#docs)*
        #(#attrs)*
        #[must_use]
        #vis struct #struct_name #impl_generics #where_clause {
            #(#fields,)*
        }

        #constructor

        #(#attrs)*
        impl #impl_generics #struct_name #ty_generics #where_clause {
            #(#setters)*
        }

        #element

        #(#attrs)*
        #into_view
    })
}

/// The fields of the struct: each prop, then `__root` for an `impl Element`,
/// then `__phantom` for a type parameter that no prop names.
fn struct_fields(model: &Model, r: &TokenStream) -> Vec<TokenStream> {
    let mut fields: Vec<TokenStream> = model.props.iter().map(|prop| field(prop, r)).collect();
    if matches!(model.returns, Returns::Element) {
        fields.push(quote! { __root: #r::view::Node });
    }
    if !model.phantom.is_empty() {
        let phantom = &model.phantom;
        fields.push(quote! {
            __phantom: ::std::marker::PhantomData<fn() -> (#(#phantom,)*)>
        });
    }
    fields
}

fn field(prop: &Prop, r: &TokenStream) -> TokenStream {
    let name = &prop.name;
    let ty = keep_type(prop, r);
    if prop.is_optional() {
        quote! { #name: ::std::option::Option<#ty> }
    } else {
        quote! { #name: #ty }
    }
}

/// `<A, B>` for the generics that are not empty, else nothing.
fn angle(params: Vec<TokenStream>) -> TokenStream {
    if params.is_empty() {
        TokenStream::new()
    } else {
        quote! { <#(#params),*> }
    }
}

fn constructor(model: &Model, r: &TokenStream) -> TokenStream {
    let (vis, name, struct_name, docs) = (&model.vis, &model.name, &model.struct_name, &model.docs);
    let (attrs, deprecated) = (&model.attrs, &model.deprecated);
    let (_, ty_generics, where_clause) = model.generics.split_for_impl();
    let positional: Vec<&Prop> = model
        .props
        .iter()
        .filter(|prop| prop.is_positional())
        .collect();
    let mut generics: Vec<TokenStream> = model
        .generics
        .params
        .iter()
        .map(|param| quote! { #param })
        .collect();
    generics.extend(positional.iter().filter_map(|prop| generic_decl(prop, r)));
    let params = positional.iter().map(|prop| {
        let name = &prop.name;
        let ty = input_type(prop, r);
        quote! { #name: #ty }
    });
    let check = api18_check(model, r);
    let fields = struct_initializers(model, r);
    let generics = angle(generics);
    quote! {
        #(#attrs)* #(#deprecated)*
        #(#docs)*
        #[track_caller]
        #vis fn #name #generics (#(#params),*) -> #struct_name #ty_generics #where_clause {
            #check
            #struct_name { #(#fields,)* }
        }
    }
}

/// The fields of the constructor's struct literal.
fn struct_initializers(model: &Model, r: &TokenStream) -> Vec<TokenStream> {
    let mut fields: Vec<TokenStream> = model
        .props
        .iter()
        .map(|prop| initializer(prop, r))
        .collect();
    if matches!(model.returns, Returns::Element) {
        fields.push(quote! { __root: #r::view::node() });
    }
    if !model.phantom.is_empty() {
        fields.push(quote! { __phantom: ::std::marker::PhantomData });
    }
    fields
}

fn initializer(prop: &Prop, r: &TokenStream) -> TokenStream {
    let name = &prop.name;
    match &prop.required {
        Required::Positional => {
            let value = store(prop, r, quote! { #name });
            quote! { #name: #value }
        }
        Required::Optional => quote! { #name: ::std::option::Option::None },
        Required::Default(expr) => {
            let value = default_value(prop, r, expr);
            quote! { #name: #value }
        }
    }
}

/// The value of a `#[prop(default = expr)]`. A callback's default goes
/// through `Handler::from_fn` directly, so the closure's argument type is
/// inferred from the handler's bound.
fn default_value(prop: &Prop, r: &TokenStream, expr: &syn::Expr) -> TokenStream {
    match &prop.kind {
        PropKind::Callback0 => quote! {{
            let __default = #expr;
            #r::custom::Handler::<()>::from_fn(move |()| __default())
        }},
        PropKind::Callback(ty) => quote! { #r::custom::Handler::<#ty>::from_fn(#expr) },
        _ => {
            let value = store(prop, r, quote! { __default });
            quote! {{ let __default = #expr; #value }}
        }
    }
}

/// The API-18 check: each recognised prop's written type is the Kivact type
/// it stands for, so a foreign `Value` or a missing import is an error here.
fn api18_check(model: &Model, r: &TokenStream) -> TokenStream {
    let pairs: Vec<(&syn::Type, TokenStream)> = model
        .props
        .iter()
        .filter_map(|prop| kivact_type(prop, r).map(|kivact| (&prop.written, kivact)))
        .collect();
    if pairs.is_empty() {
        return TokenStream::new();
    }
    let written = pairs.iter().map(|(written, _)| written);
    let kivact = pairs.iter().map(|(_, kivact)| kivact);
    quote! {
        let _: fn(::std::marker::PhantomData<(#(#written,)*)>) -> ::std::marker::PhantomData<(#(#kivact,)*)> = |written| written;
    }
}

fn setter(prop: &Prop, r: &TokenStream) -> TokenStream {
    let name = &prop.name;
    let generics = angle(generic_decl(prop, r).into_iter().collect());
    let input = input_type(prop, r);
    let value = store(prop, r, quote! { #name });
    let assign = if prop.is_optional() {
        quote! { self.#name = ::std::option::Option::Some(#value); }
    } else {
        quote! { self.#name = #value; }
    };
    quote! {
        #[track_caller]
        pub fn #name #generics (mut self, #name: #input) -> Self {
            #assign
            self
        }
    }
}

fn into_view(model: &Model, r: &TokenStream) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = model.generics.split_for_impl();
    let struct_name = &model.struct_name;
    let names = destructure(model);
    let mounts = model.props.iter().filter_map(|prop| mount(prop, r));
    let rebinds = model.props.iter().filter(|prop| prop.mutable).map(|prop| {
        let name = &prop.name;
        quote! { #[allow(unused_mut)] let mut #name = #name; }
    });
    let statements = &model.body.stmts;
    let finish = match model.returns {
        Returns::Element => quote! {
            #r::view::Element::with_node(__body, move |__n| __n.absorb(__root))
        },
        Returns::IntoView => quote! { __body },
    };
    quote! {
        impl #impl_generics #r::view::IntoView for #struct_name #ty_generics #where_clause {
            fn into_view(self) -> #r::view::View {
                let #struct_name { #(#names,)* } = self;
                #r::custom::scope(move || {
                    #(#mounts)*
                    #(#rebinds)*
                    let __body = (move || {
                        #(#statements)*
                    })();
                    #finish
                })
            }
        }
    }
}

/// The pattern of `into_view`'s destructuring: the props, then `__root` and
/// `__phantom` as the struct has them.
fn destructure(model: &Model) -> Vec<TokenStream> {
    let mut names: Vec<TokenStream> = model
        .props
        .iter()
        .map(|prop| {
            let name = &prop.name;
            quote! { #name }
        })
        .collect();
    if matches!(model.returns, Returns::Element) {
        names.push(quote! { __root });
    }
    if !model.phantom.is_empty() {
        names.push(quote! { __phantom: _ });
    }
    names
}

/// `impl Element`: the node the caller's modifiers are collected in.
fn element_impl(model: &Model, r: &TokenStream) -> TokenStream {
    if !matches!(model.returns, Returns::Element) {
        return TokenStream::new();
    }
    let (impl_generics, ty_generics, where_clause) = model.generics.split_for_impl();
    let struct_name = &model.struct_name;
    quote! {
        impl #impl_generics #r::view::Element for #struct_name #ty_generics #where_clause {
            fn node_mut(&mut self) -> &mut #r::view::Node {
                &mut self.__root
            }
        }
    }
}
