//! Role: Read a component function into the model the expansion writes.

use proc_macro2::TokenTree;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Block, Expr, FnArg, GenericArgument, GenericParam, Generics, Ident, ItemFn, Pat,
    PathArguments, ReturnType, Signature, Type, TypeParamBound, Visibility,
};

/// A component function as the expansion sees it.
pub(crate) struct Model {
    pub(crate) vis: Visibility,
    pub(crate) name: Ident,
    pub(crate) struct_name: Ident,
    pub(crate) docs: Vec<Attribute>,
    /// `allow`, `warn` and `deny`: put on every generated item.
    pub(crate) attrs: Vec<Attribute>,
    /// `deprecated`: put on the builder function, the name callers write.
    pub(crate) deprecated: Vec<Attribute>,
    pub(crate) props: Vec<Prop>,
    pub(crate) body: Block,
    /// `-> impl Element` (stylable) or `-> impl IntoView`.
    pub(crate) returns: Returns,
    /// The function's type parameters and where-clause (5.1 rule 9), each
    /// type parameter with `'static` added to its bounds.
    pub(crate) generics: Generics,
    /// The type parameters no prop's type names, held by `__phantom`.
    pub(crate) phantom: Vec<Ident>,
}

/// What the component function returns.
pub(crate) enum Returns {
    Element,
    IntoView,
}

/// One argument of the component function.
pub(crate) struct Prop {
    pub(crate) name: Ident,
    pub(crate) kind: PropKind,
    pub(crate) required: Required,
    /// Declared `mut` in the function, so the body may change it.
    pub(crate) mutable: bool,
    /// The type as written, without `Option` (for the existence check of API-18).
    pub(crate) written: Type,
}

impl Prop {
    /// Whether a caller passes this prop by position.
    pub(crate) fn is_positional(&self) -> bool {
        matches!(self.required, Required::Positional)
    }

    /// Whether a missing setter leaves `None`.
    pub(crate) fn is_optional(&self) -> bool {
        matches!(self.required, Required::Optional)
    }
}

/// How a caller supplies a prop.
pub(crate) enum Required {
    /// A positional argument of the builder function.
    Positional,
    /// `Option<..>`: a setter, absent by default.
    Optional,
    /// `#[prop(default = expr)]`: a setter with a default.
    Default(Expr),
}

/// What the prop holds and how the expansion converts it.
pub(crate) enum PropKind {
    /// `Value<String>`: text, takes `IntoText`, kept as `Prop<String>`.
    Text,
    /// `Value<T>`: takes `IntoValue<T>`, kept as `Prop<T>`.
    Value(Type),
    /// `Signal<T>`: takes exactly a `Signal<T>`.
    Signal(Type),
    /// `Callback<()>`: takes `IntoHandler<(), M>`, kept as `Handler<()>`.
    Callback0,
    /// `Callback<E>` (E is not `()`): takes `Fn(E) -> R` (DEC-16), kept as `Handler<E>`.
    Callback(Type),
    /// `View`: takes `IntoView`.
    View,
    /// `Vec<View>`: takes `IntoChildren`.
    Children,
    /// `ViewFn`: takes `IntoViewFn<V>`, kept as `ViewFn<()>`.
    ViewFn,
    /// A type parameter of the component itself (`control: C`): takes `C`
    /// as it is (rule 3: wrapping it in `Into` stops inference).
    Param(Type),
    /// Any other type `T`: takes `impl Into<T>`.
    Other(Type),
}

/// Read `function` as the model of the builder it becomes.
pub(crate) fn component(function: ItemFn) -> syn::Result<Model> {
    check_signature(&function.sig)?;
    let returns = returns(&function.sig.output)?;
    let generics = generics(&function.sig)?;
    let params: Vec<Ident> = generics
        .type_params()
        .map(|param| param.ident.clone())
        .collect();
    let name = function.sig.ident.clone();
    let struct_name = Ident::new(&upper_camel(&name.to_string()), name.span());
    let (docs, attrs, deprecated) = item_attributes(&function.attrs)?;
    let mut props = Vec::with_capacity(function.sig.inputs.len());
    for input in &function.sig.inputs {
        props.push(prop(input, &params)?);
    }
    let phantom = phantom(&params, &function.sig.inputs);
    Ok(Model {
        vis: function.vis,
        name,
        struct_name,
        docs,
        attrs,
        deprecated,
        props,
        body: *function.block,
        returns,
        generics,
        phantom,
    })
}

/// Split the function's attributes into doc comments, lint attributes and
/// `deprecated`.
fn item_attributes(
    all: &[Attribute],
) -> syn::Result<(Vec<Attribute>, Vec<Attribute>, Vec<Attribute>)> {
    let mut docs = Vec::new();
    let mut attrs = Vec::new();
    let mut deprecated = Vec::new();
    for attr in all {
        let path = attr.path();
        if path.is_ident("doc") {
            docs.push(attr.clone());
        } else if path.is_ident("deprecated") {
            deprecated.push(attr.clone());
        } else if ["allow", "warn", "deny"]
            .iter()
            .any(|name| path.is_ident(name))
        {
            attrs.push(attr.clone());
        } else {
            return Err(syn::Error::new_spanned(
                attr,
                "#[component] keeps doc comments and the allow, warn, deny and deprecated attributes; use #[allow(..)] instead of #[expect(..)] and remove any other attribute",
            ));
        }
    }
    Ok((docs, attrs, deprecated))
}

/// `user_row` as `UserRow`.
pub(crate) fn upper_camel(name: &str) -> String {
    let mut camel = String::new();
    for part in name.split('_').filter(|part| !part.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            camel.extend(first.to_uppercase());
            camel.push_str(chars.as_str());
        }
    }
    camel
}

fn check_signature(sig: &Signature) -> syn::Result<()> {
    if sig.constness.is_some()
        || sig.asyncness.is_some()
        || sig.unsafety.is_some()
        || sig.abi.is_some()
        || sig.variadic.is_some()
    {
        return Err(syn::Error::new_spanned(
            &sig.ident,
            "a #[component] function is a plain `fn`",
        ));
    }
    for input in &sig.inputs {
        if matches!(input, FnArg::Receiver(_)) {
            return Err(syn::Error::new_spanned(
                input,
                "a #[component] is a free function",
            ));
        }
    }
    Ok(())
}

fn returns(output: &ReturnType) -> syn::Result<Returns> {
    let ReturnType::Type(_, ty) = output else {
        return Err(not_a_component_return(output));
    };
    let Type::ImplTrait(trait_type) = ty.as_ref() else {
        return Err(not_a_component_return(output));
    };
    let last = trait_type.bounds.iter().find_map(|bound| match bound {
        TypeParamBound::Trait(bound) => bound.path.segments.last(),
        _ => None,
    });
    match last.map(|segment| segment.ident.to_string()) {
        Some(name) if name == "Element" => Ok(Returns::Element),
        Some(name) if name == "IntoView" => Ok(Returns::IntoView),
        _ => Err(not_a_component_return(output)),
    }
}

fn not_a_component_return(output: &ReturnType) -> syn::Error {
    syn::Error::new_spanned(
        output,
        "a #[component] function returns `impl Element` or `impl IntoView`",
    )
}

/// The type parameters of the function (5.1 rule 9): lifetimes and const
/// parameters are refused, and each type parameter gets `'static`.
fn generics(sig: &Signature) -> syn::Result<Generics> {
    let mut generics = sig.generics.clone();
    for param in &mut generics.params {
        match param {
            GenericParam::Lifetime(lifetime) => {
                return Err(syn::Error::new_spanned(
                    lifetime,
                    "a #[component] takes type parameters only; remove the lifetime parameter",
                ));
            }
            GenericParam::Const(constant) => {
                return Err(syn::Error::new_spanned(
                    constant,
                    "a #[component] takes type parameters only; remove the const parameter",
                ));
            }
            GenericParam::Type(param) => {
                if param.ident.to_string().starts_with("__Kv") {
                    return Err(syn::Error::new_spanned(
                        &param.ident,
                        "type parameter names starting with `__Kv` are reserved for #[component]",
                    ));
                }
                param.bounds.push(syn::parse_quote!('static));
            }
        }
    }
    Ok(generics)
}

fn prop(input: &FnArg, params: &[Ident]) -> syn::Result<Prop> {
    let FnArg::Typed(pat) = input else {
        return Err(syn::Error::new_spanned(
            input,
            "a #[component] is a free function",
        ));
    };
    let Pat::Ident(binding) = pat.pat.as_ref() else {
        return Err(syn::Error::new_spanned(
            &pat.pat,
            "a #[component] prop is a named argument",
        ));
    };
    if binding.by_ref.is_some() || binding.subpat.is_some() {
        return Err(syn::Error::new_spanned(
            &pat.pat,
            "a #[component] prop is a plain named argument (no `ref` or `@` pattern)",
        ));
    }
    let name = binding.ident.clone();
    if name.to_string().starts_with("__") {
        return Err(syn::Error::new_spanned(
            &name,
            "prop names starting with `__` are reserved for #[component]",
        ));
    }
    let (required, written) = required(&pat.attrs, &pat.ty)?;
    if matches!(written, Type::ImplTrait(_)) {
        return Err(syn::Error::new_spanned(
            &written,
            "a #[component] prop has a concrete type: write `Value<T>` for a value, `View` for content or `Callback<E>` for a handler instead of `impl Trait`",
        ));
    }
    let kind = kind(&written, params);
    Ok(Prop {
        name,
        kind,
        required,
        mutable: binding.mutability.is_some(),
        written,
    })
}

/// The required, optional or defaulted form of an argument, and the type
/// without its `Option`.
fn required(attrs: &[Attribute], ty: &Type) -> syn::Result<(Required, Type)> {
    let default = prop_default(attrs)?;
    match (option_inner(ty), default) {
        (Some(_), Some(_)) => Err(syn::Error::new_spanned(
            ty,
            "an `Option<..>` prop is optional already; remove `#[prop(default = ..)]`",
        )),
        (Some(inner), None) => Ok((Required::Optional, inner.clone())),
        (None, Some(expr)) => Ok((Required::Default(expr), ty.clone())),
        (None, None) => Ok((Required::Positional, ty.clone())),
    }
}

/// The `default = expr` of a `#[prop(..)]` attribute; any other attribute
/// on an argument is refused.
fn prop_default(attrs: &[Attribute]) -> syn::Result<Option<Expr>> {
    let mut default: Option<Expr> = None;
    for attr in attrs {
        if !attr.path().is_ident("prop") {
            return Err(syn::Error::new_spanned(
                attr,
                "a #[component] prop takes only #[prop(default = ..)]",
            ));
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("default") {
                default = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error(
                    "#[prop] accepts only `default = <expr>`; an optional prop is written `Option<T>`",
                ))
            }
        })?;
    }
    Ok(default)
}

fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    match arguments.args.first()? {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// The kind of a prop: a type parameter of the component is taken as it
/// is; any other type is recognised by the spelling of its last segment.
fn kind(ty: &Type, params: &[Ident]) -> PropKind {
    if is_param(ty, params) {
        return PropKind::Param(ty.clone());
    }
    let Type::Path(path) = ty else {
        return PropKind::Other(ty.clone());
    };
    let Some(segment) = path.path.segments.last() else {
        return PropKind::Other(ty.clone());
    };
    let inner = type_arguments(segment).first().copied();
    let other = || PropKind::Other(ty.clone());
    match segment.ident.to_string().as_str() {
        "Value" => match inner {
            Some(inner) if is_named(inner, "String") => PropKind::Text,
            Some(inner) => PropKind::Value(inner.clone()),
            None => other(),
        },
        "Signal" => match inner {
            Some(inner) => PropKind::Signal(inner.clone()),
            None => other(),
        },
        "Callback" => match inner {
            Some(inner) if is_unit(inner) => PropKind::Callback0,
            Some(inner) => PropKind::Callback(inner.clone()),
            None => other(),
        },
        "View" => PropKind::View,
        "Vec" if inner.is_some_and(|inner| is_named(inner, "View")) => PropKind::Children,
        "ViewFn" => PropKind::ViewFn,
        _ => other(),
    }
}

/// Whether `ty` is exactly one of the component's type parameters.
fn is_param(ty: &Type, params: &[Ident]) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return false;
    }
    let segment = &path.path.segments[0];
    matches!(segment.arguments, PathArguments::None) && params.contains(&segment.ident)
}

/// The type parameters that no argument's type names (they are held by
/// `__phantom`, 4.8 節).
fn phantom(params: &[Ident], inputs: &Punctuated<FnArg, syn::token::Comma>) -> Vec<Ident> {
    let types: Vec<proc_macro2::TokenStream> = inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Typed(pat) => Some(quote::ToTokens::to_token_stream(&pat.ty)),
            FnArg::Receiver(_) => None,
        })
        .collect();
    params
        .iter()
        .filter(|param| !types.iter().any(|ty| mentions(ty.clone(), param)))
        .cloned()
        .collect()
}

/// Whether the tokens of a type contain the identifier, inside groups too.
fn mentions(tokens: proc_macro2::TokenStream, ident: &Ident) -> bool {
    tokens.into_iter().any(|tree| match tree {
        TokenTree::Ident(found) => found == *ident,
        TokenTree::Group(group) => mentions(group.stream(), ident),
        TokenTree::Punct(_) | TokenTree::Literal(_) => false,
    })
}

fn type_arguments(segment: &syn::PathSegment) -> Vec<&Type> {
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return Vec::new();
    };
    arguments
        .args
        .iter()
        .filter_map(|argument| match argument {
            GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .collect()
}

fn is_named(ty: &Type, name: &str) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == name)
}

fn is_unit(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())
}
