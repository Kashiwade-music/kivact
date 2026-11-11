//! Role: Read a component function into the model the expansion writes.

use syn::{
    Attribute, Block, Expr, FnArg, GenericArgument, Ident, ItemFn, Pat, PathArguments, ReturnType,
    Type, TypeParamBound, Visibility,
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
}

/// One argument of the component function.
pub(crate) struct Prop {
    pub(crate) name: Ident,
    pub(crate) kind: PropKind,
    pub(crate) required: Required,
    /// Declared `mut` in the function, so the body may change it.
    pub(crate) mutable: bool,
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
    /// `#[prop(optional)]`: a setter, absent by default.
    Optional,
    /// `#[prop(default = expr)]`: a setter with a default.
    Default(Expr),
}

/// What the prop holds and how the expansion converts it.
pub(crate) enum PropKind {
    /// `Value<String>`: text, kept as `Reactive<String>`.
    Text,
    /// `Value<T>`: a value, kept as `Reactive<T>`.
    Value(Type),
    /// `Binding<T>`: a two-way binding, kept as it is.
    Binding(Type),
    /// `Callback<()>`: a handler without a payload, converted in the scope.
    Callback0,
    /// `Callback<E>`: a handler with a payload, converted in the scope.
    Callback(Type),
    /// `View`: one content slot, kept as it is.
    View,
    /// `Vec<View>`: several children, kept as `Vec<View>`.
    Children,
    /// `ViewFn`: content created again on every opening, kept as `ViewFn<()>`.
    ViewFn,
    /// Any other type, kept as `T` through `Into`.
    Other(Type),
}

/// Read `function` as the model of the builder it becomes.
pub(crate) fn component(function: ItemFn) -> syn::Result<Model> {
    check_signature(&function)?;
    let name = function.sig.ident.clone();
    let struct_name = Ident::new(&upper_camel(&name.to_string()), name.span());
    let mut docs = Vec::new();
    let mut attrs = Vec::new();
    let mut deprecated = Vec::new();
    for attr in &function.attrs {
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
    let mut props = Vec::with_capacity(function.sig.inputs.len());
    for input in &function.sig.inputs {
        props.push(prop(input)?);
    }
    Ok(Model {
        vis: function.vis,
        name,
        struct_name,
        docs,
        attrs,
        deprecated,
        props,
        body: *function.block,
    })
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

fn check_signature(function: &ItemFn) -> syn::Result<()> {
    let sig = &function.sig;
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
    if !sig.generics.params.is_empty() || sig.generics.where_clause.is_some() {
        return Err(syn::Error::new_spanned(
            &sig.generics,
            "#[component] functions cannot be generic; take `Value<T>` props instead",
        ));
    }
    if !returns_into_view(&sig.output) {
        return Err(syn::Error::new_spanned(
            &sig.output,
            "a #[component] function returns `impl IntoView`",
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

fn returns_into_view(output: &ReturnType) -> bool {
    let ReturnType::Type(_, ty) = output else {
        return false;
    };
    let Type::ImplTrait(trait_type) = ty.as_ref() else {
        return false;
    };
    trait_type.bounds.iter().any(|bound| {
        let TypeParamBound::Trait(bound) = bound else {
            return false;
        };
        bound
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "IntoView")
    })
}

fn prop(input: &FnArg) -> syn::Result<Prop> {
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
    let (required, ty) = required(&pat.attrs, &pat.ty)?;
    if matches!(ty, Type::ImplTrait(_)) {
        return Err(syn::Error::new_spanned(
            &ty,
            "a #[component] prop has a concrete type: write `Value<T>` for a value, `View` for content or `Callback<E>` for a handler instead of `impl Trait`",
        ));
    }
    let kind = kind(&ty)?;
    Ok(Prop {
        name,
        kind,
        required,
        mutable: binding.mutability.is_some(),
    })
}

fn required(attrs: &[Attribute], ty: &Type) -> syn::Result<(Required, Type)> {
    let mut optional = false;
    let mut default: Option<Expr> = None;
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("prop")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("optional") {
                optional = true;
                Ok(())
            } else if meta.path.is_ident("default") {
                default = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("#[prop] accepts `optional` or `default = <expr>`"))
            }
        })?;
    }
    match (optional, default) {
        (true, Some(_)) => Err(syn::Error::new_spanned(
            ty,
            "use either `optional` or `default`",
        )),
        (true, None) => match option_inner(ty) {
            Some(inner) => Ok((Required::Optional, inner.clone())),
            None => Err(syn::Error::new_spanned(
                ty,
                "an optional prop has type `Option<..>`",
            )),
        },
        (false, Some(expr)) => Ok((Required::Default(expr), ty.clone())),
        (false, None) => Ok((Required::Positional, ty.clone())),
    }
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

fn kind(ty: &Type) -> syn::Result<PropKind> {
    let Type::Path(path) = ty else {
        return Ok(PropKind::Other(ty.clone()));
    };
    let Some(segment) = path.path.segments.last() else {
        return Ok(PropKind::Other(ty.clone()));
    };
    let inner = type_arguments(segment).first().copied();
    let other = || PropKind::Other(ty.clone());
    let kind = match segment.ident.to_string().as_str() {
        "Value" => match inner {
            Some(inner) if is_named(inner, "String") => PropKind::Text,
            Some(inner) => PropKind::Value(inner.clone()),
            None => other(),
        },
        "Binding" => match inner {
            Some(inner) => PropKind::Binding(inner.clone()),
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
    };
    Ok(kind)
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
