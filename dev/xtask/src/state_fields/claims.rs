//! Role: Verify what state-field markers claim within the module a field is visible in.

use crate::check_source::Source;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::Visit;

/// The rule name reported by these checks.
const RULE: &str = "state-fields";
/// The marker that explains a field.
const MARKER: &str = "// xtask-allow(state-fields):";
/// The phrase that names a projection's one writer (SL-9).
const WRITER: &str = "written only by ";
/// Method calls that write the value they are called on or hand out a
/// mutable handle to it.
const WRITE_METHODS: [&str; 15] = [
    "set",
    "update",
    "replace",
    "take",
    "swap",
    "insert",
    "push",
    "clear",
    "get_or_init",
    "set_if_changed",
    "borrow_mut",
    "get_mut",
    "as_mut",
    "lock",
    "write",
];
/// Method calls that only read the value they are called on.
const READ_METHODS: [&str; 26] = [
    "get",
    "get_untracked",
    "try_get",
    "try_get_untracked",
    "with",
    "with_untracked",
    "borrow",
    "is_some",
    "is_none",
    "is_some_and",
    "is_none_or",
    "as_ref",
    "as_deref",
    "clone",
    "cloned",
    "copied",
    "iter",
    "len",
    "is_empty",
    "contains",
    "contains_key",
    "map",
    "and_then",
    "unwrap_or",
    "unwrap_or_default",
    "to_owned",
];
/// Method calls that pass the value on; their own receiver decides.
const PASS_METHODS: [&str; 2] = ["unwrap", "expect"];
/// Standard macros whose arguments are plain expressions.
const VALUE_MACROS: [&str; 18] = [
    "assert",
    "assert_eq",
    "assert_ne",
    "debug_assert",
    "debug_assert_eq",
    "debug_assert_ne",
    "format",
    "format_args",
    "print",
    "println",
    "eprint",
    "eprintln",
    "write",
    "writeln",
    "panic",
    "unreachable",
    "todo",
    "dbg",
];

/// One `projection:` marker and the writer it names.
struct Claim<'a> {
    source: &'a Source,
    line: usize,
    /// The struct, or `Enum::Variant`, whose field the marker explains.
    owner: Owner,
    field: String,
    /// The module whose subtree can name the field (never the crate root).
    scope: String,
    writer: Writer,
}

/// The item that declares a claimed field.
#[derive(Clone)]
enum Owner {
    Struct(String),
    Variant {
        enumeration: String,
        variant: String,
    },
}

/// The function a projection names: `fn` or `Type::fn`.
#[derive(Clone)]
struct Writer {
    ty: Option<String>,
    name: String,
}

/// What one use of the claimed field does.
enum Effect {
    /// A write: an assignment, a write method, a literal or `..rest`.
    Write,
    /// A use the checker cannot prove to be a read (an alias).
    Unverified(String),
}

/// One use of the claimed field other than a plain read.
struct Use {
    path: PathBuf,
    line: usize,
    /// The innermost function and its impl's self type, if any.
    function: Option<(String, Option<String>)>,
    effect: Effect,
}

/// Check every `projection:` claim and every `independent:` reason (X4, X5).
///
/// `all` is every loaded source; errors are reported only in the sources
/// `report` selects.
pub(crate) fn verify(all: &[Source], report: &dyn Fn(&Source) -> bool, errors: &mut Vec<String>) {
    duplicate_reasons(all, report, errors);
    for source in all.iter().filter(|source| report(source)) {
        for claim in claims(source, errors) {
            verify_claim(all, &claim, errors);
        }
    }
}

/// X5: an `independent:` reason shared with another file says nothing about
/// the item it closes.
fn duplicate_reasons(all: &[Source], report: &dyn Fn(&Source) -> bool, errors: &mut Vec<String>) {
    let mut paths: BTreeMap<String, BTreeSet<PathBuf>> = BTreeMap::new();
    let mut lines = Vec::new();
    for source in all {
        for (index, text) in source.text.lines().enumerate() {
            let Some(reason) = text.trim_start().strip_prefix(MARKER) else {
                continue;
            };
            let reason = reason.split_whitespace().collect::<Vec<_>>().join(" ");
            if !reason.starts_with("independent:") || !source.owns_line(index + 1) {
                continue;
            }
            paths
                .entry(reason.clone())
                .or_default()
                .insert(source.path.clone());
            if report(source) {
                lines.push((source, index + 1, reason));
            }
        }
    }
    for (source, line, reason) in lines {
        let Some(other) = paths[&reason].iter().find(|path| **path != source.path) else {
            continue;
        };
        source.error_at_line(
            RULE,
            line,
            &format!(
                "independent: reason repeats in {}; say why every combination of this item's fields occurs",
                other.display()
            ),
            errors,
        );
    }
}

/// The `projection:` markers on the struct and named-variant fields of `source`.
fn claims<'a>(source: &'a Source, errors: &mut Vec<String>) -> Vec<Claim<'a>> {
    let mut claims = Vec::new();
    for item in &source.items {
        // A variant's fields are as visible as their enum.
        let owners: Vec<(Owner, &syn::Fields, Option<&syn::Visibility>)> = match item {
            syn::Item::Struct(item) => {
                vec![(Owner::Struct(item.ident.to_string()), &item.fields, None)]
            }
            syn::Item::Enum(item) => item
                .variants
                .iter()
                .map(|variant| {
                    let owner = Owner::Variant {
                        enumeration: item.ident.to_string(),
                        variant: variant.ident.to_string(),
                    };
                    (owner, &variant.fields, Some(&item.vis))
                })
                .collect(),
            _ => continue,
        };
        for (owner, fields, enum_visibility) in owners {
            for field in fields {
                let visibility = enum_visibility.unwrap_or(&field.vis);
                if let Some(claim) = claim(source, &owner, field, visibility, errors) {
                    claims.push(claim);
                }
            }
        }
    }
    claims
}

/// The claim of one field's marker, when it is a well-formed `projection:`
/// on a field that is not visible to the whole crate.
fn claim<'a>(
    source: &'a Source,
    owner: &Owner,
    field: &syn::Field,
    visibility: &syn::Visibility,
    errors: &mut Vec<String>,
) -> Option<Claim<'a>> {
    let name = field.ident.as_ref()?.to_string();
    let line = source.allow_line(RULE, field.span())?;
    let reason = source.allow_reason(RULE, line)?;
    if !reason.starts_with("projection:") {
        return None;
    }
    let Some(writer) = named_writer(&reason) else {
        source.error_at_line(
            RULE,
            line,
            "projection marker must name one writer: \"written only by <fn>\"",
            errors,
        );
        return None;
    };
    let Some(scope) = scope(source, visibility) else {
        source.error_at_line(
            RULE,
            line,
            &format!(
                "projection field {}.{name} is visible to the whole crate; make it private or pub(super) below the crate root so its writers can be checked",
                owner.label()
            ),
            errors,
        );
        return None;
    };
    Some(Claim {
        source,
        line,
        owner: owner.clone(),
        field: name,
        scope,
        writer,
    })
}

impl Owner {
    fn label(&self) -> String {
        match self {
            Self::Struct(name) => name.clone(),
            Self::Variant {
                enumeration,
                variant,
            } => format!("{enumeration}::{variant}"),
        }
    }
}

/// The one `written only by <fn>` or `written only by <Type::fn>` in `reason`.
fn named_writer(reason: &str) -> Option<Writer> {
    let mut parts = reason.split(WRITER);
    parts.next()?;
    let rest = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let path: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == ':')
        .collect();
    let path = path.trim_end_matches(':');
    let segments: Vec<&str> = path.split("::").collect();
    let valid = |segment: &&str| {
        segment
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
    };
    match segments.as_slice() {
        [name] if valid(name) => Some(Writer {
            ty: None,
            name: (*name).to_owned(),
        }),
        [ty, name] if valid(ty) && valid(name) => Some(Writer {
            ty: Some((*ty).to_owned()),
            name: (*name).to_owned(),
        }),
        _ => None,
    }
}

/// The module whose subtree can name a field with `visibility`, declared in
/// `source`; `None` when that is the whole crate.
fn scope(source: &Source, visibility: &syn::Visibility) -> Option<String> {
    let parent = |module: &str| {
        module
            .rsplit_once("::")
            .map_or(String::new(), |(parent, _)| parent.to_owned())
    };
    let module = match visibility {
        syn::Visibility::Inherited => source.module.clone(),
        syn::Visibility::Public(_) => String::new(),
        syn::Visibility::Restricted(restricted) => {
            let path: Vec<String> = restricted
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            match path.first().map(String::as_str) {
                Some("crate") => path[1..].join("::"),
                Some("self") => source.module.clone(),
                Some("super") => path
                    .iter()
                    .filter(|segment| *segment == "super")
                    .fold(source.module.clone(), |base, _| parent(&base)),
                _ => String::new(),
            }
        }
    };
    (!module.is_empty()).then_some(module)
}

/// Whether `source` is in the subtree of `scope` within the target of `declared`.
fn in_scope(scope: &str, declared: &Source, source: &Source) -> bool {
    source.identity_root() == declared.identity_root()
        && (source.module == scope || source.module.starts_with(&format!("{scope}::")))
}

/// Rules 2 and 3 of SL-9 for one claim.
fn verify_claim(all: &[Source], claim: &Claim<'_>, errors: &mut Vec<String>) {
    let visible: Vec<&Source> = all
        .iter()
        .filter(|source| in_scope(&claim.scope, claim.source, source))
        .collect();
    let label = format!("{}.{}", claim.owner.label(), claim.field);
    let writer = match &claim.writer.ty {
        Some(ty) => format!("{ty}::{}", claim.writer.name),
        None => claim.writer.name.clone(),
    };
    let found: Vec<Found> = visible
        .iter()
        .filter_map(|source| definition(source, &claim.writer))
        .collect();
    let detail = if found.is_empty() {
        Some(format!(
            "projection writer {writer} of {label} does not exist where the field is visible"
        ))
    } else if found.iter().any(|found| found.returns_reference) {
        Some(format!(
            "projection writer {writer} of {label} returns a reference; a borrow of the field would let its callers change it"
        ))
    } else {
        None
    };
    if let Some(detail) = detail {
        claim
            .source
            .error_at_line(RULE, claim.line, &detail, errors);
        return;
    }
    let uses = visible
        .iter()
        .flat_map(|source| uses(source, &claim.owner, &claim.field));
    let mut reported = BTreeSet::new();
    for found in uses {
        if is_writer(&found, &claim.writer) {
            continue;
        }
        let function = found
            .function
            .as_ref()
            .map_or("no function", |(name, _)| name.as_str());
        let at = format!("{}:{}", found.path.display(), found.line);
        let detail = match &found.effect {
            Effect::Write => format!(
                "{label} is also written in {function} at {at}; a projection has one writer ({writer})"
            ),
            Effect::Unverified(why) => format!(
                "{label} may be changed in {function} at {at} ({why}); the checker cannot prove it is a read, so change `{}` only in {writer} and read it by value or with a read method (give the field a name unique in its module if this is another type's field)",
                claim.field
            ),
        };
        if reported.insert(detail.clone()) {
            claim
                .source
                .error_at_line(RULE, claim.line, &detail, errors);
        }
    }
}

fn is_writer(found: &Use, writer: &Writer) -> bool {
    found.function.as_ref().is_some_and(|(name, ty)| {
        *name == writer.name
            && writer
                .ty
                .as_ref()
                .is_none_or(|expected| ty.as_ref() == Some(expected))
    })
}

/// The definition of the function `writer` names in `source`, if any.
fn definition(source: &Source, writer: &Writer) -> Option<Found> {
    let mut finder = Functions {
        writer,
        impls: Vec::new(),
        found: None,
    };
    for item in &source.items {
        finder.visit_item(item);
    }
    finder.found
}

/// A found writer definition.
struct Found {
    returns_reference: bool,
}

/// Finds the definition of one named function.
struct Functions<'a> {
    writer: &'a Writer,
    impls: Vec<Option<String>>,
    found: Option<Found>,
}

impl Functions<'_> {
    fn note(&mut self, signature: &syn::Signature) {
        let returns_reference = match &signature.output {
            syn::ReturnType::Default => false,
            syn::ReturnType::Type(_, ty) => has_reference(ty),
        };
        self.found = Some(Found { returns_reference });
    }
}

impl<'ast> Visit<'ast> for Functions<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if node.content.is_none() {
            syn::visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.impls.push(type_name(&node.self_ty));
        syn::visit::visit_item_impl(self, node);
        self.impls.pop();
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.writer.ty.is_none() && node.sig.ident == self.writer.name {
            self.note(&node.sig);
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let ty = self.impls.last().cloned().flatten();
        if node.sig.ident == self.writer.name
            && self
                .writer
                .ty
                .as_ref()
                .is_none_or(|expected| ty.as_ref() == Some(expected))
        {
            self.note(&node.sig);
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}

/// Whether `ty` contains a reference type.
fn has_reference(ty: &syn::Type) -> bool {
    struct References(bool);
    impl<'ast> Visit<'ast> for References {
        fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
            self.0 = true;
        }
    }
    let mut references = References(false);
    references.visit_type(ty);
    references.0
}

/// The uses of `field` of `owner` in `source` other than plain reads.
fn uses(source: &Source, owner: &Owner, field: &str) -> Vec<Use> {
    let mut visitor = Uses {
        source,
        owner,
        field,
        functions: Vec::new(),
        impls: Vec::new(),
        found: Vec::new(),
    };
    for item in &source.items {
        visitor.visit_item(item);
    }
    visitor.found
}

/// Collects the uses of one field other than plain reads.
struct Uses<'a> {
    source: &'a Source,
    owner: &'a Owner,
    field: &'a str,
    functions: Vec<(String, Option<String>)>,
    impls: Vec<Option<String>>,
    found: Vec<Use>,
}

impl Uses<'_> {
    fn record(&mut self, line: usize, effect: Effect) {
        if self.source.owns_line(line) {
            self.found.push(Use {
                path: self.source.path.clone(),
                line,
                function: self.functions.last().cloned(),
                effect,
            });
        }
    }

    /// Whether a struct literal's or pattern's path names the owner.
    fn names_owner(&self, path: &syn::Path) -> bool {
        let segments: Vec<String> = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let this = self.impls.last().cloned().flatten();
        match self.owner {
            Owner::Struct(name) => match segments.as_slice() {
                [only] if only == "Self" => this.as_ref() == Some(name),
                [.., last] => last == name,
                [] => false,
            },
            Owner::Variant {
                enumeration,
                variant,
            } => match segments.as_slice() {
                [first, last] if first == "Self" => {
                    last == variant && this.as_ref() == Some(enumeration)
                }
                [.., last] => last == variant,
                [] => false,
            },
        }
    }

    /// Whether the place `expr` (a receiver, an assignee or a borrow) is the
    /// claimed field or a part of it. A variant's field has no `.field`
    /// access; `self.field` in another type's impl is that type's field.
    fn reaches(&self, expr: &syn::Expr) -> bool {
        let Owner::Struct(owner) = self.owner else {
            return false;
        };
        match expr {
            syn::Expr::Paren(inner) => self.reaches(&inner.expr),
            syn::Expr::Group(inner) => self.reaches(&inner.expr),
            syn::Expr::Unary(inner) if matches!(inner.op, syn::UnOp::Deref(_)) => {
                self.reaches(&inner.expr)
            }
            syn::Expr::Index(index) => self.reaches(&index.expr),
            syn::Expr::MethodCall(call)
                if PASS_METHODS.contains(&call.method.to_string().as_str()) =>
            {
                self.reaches(&call.receiver)
            }
            syn::Expr::Field(access) => {
                let named =
                    matches!(&access.member, syn::Member::Named(name) if name == self.field);
                let other_self = is_self(&access.base)
                    && self
                        .impls
                        .last()
                        .cloned()
                        .flatten()
                        .is_some_and(|this| this != *owner);
                (named && !other_self) || self.reaches(&access.base)
            }
            _ => false,
        }
    }

    /// Whether a macro's tokens name `.field` (a use the checker cannot read).
    fn mentions(&self, tokens: proc_macro2::TokenStream) -> bool {
        let mut dot = false;
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Group(group) => {
                    if self.mentions(group.stream()) {
                        return true;
                    }
                    dot = false;
                }
                proc_macro2::TokenTree::Punct(punct) => dot = punct.as_char() == '.',
                proc_macro2::TokenTree::Ident(ident) => {
                    if dot && ident == self.field {
                        return true;
                    }
                    dot = false;
                }
                proc_macro2::TokenTree::Literal(_) => dot = false,
            }
        }
        false
    }
}

impl<'ast> Visit<'ast> for Uses<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if node.content.is_none() {
            syn::visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        self.impls.push(type_name(&node.self_ty));
        syn::visit::visit_item_impl(self, node);
        self.impls.pop();
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.functions.push((node.sig.ident.to_string(), None));
        syn::visit::visit_item_fn(self, node);
        self.functions.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let ty = self.impls.last().cloned().flatten();
        self.functions.push((node.sig.ident.to_string(), ty));
        syn::visit::visit_impl_item_fn(self, node);
        self.functions.pop();
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.functions.push((node.sig.ident.to_string(), None));
        syn::visit::visit_trait_item_fn(self, node);
        self.functions.pop();
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        if self.names_owner(&node.path) {
            let listed = node.fields.iter().find(
                |value| matches!(&value.member, syn::Member::Named(name) if name == self.field),
            );
            match (listed, &node.rest) {
                (Some(value), _) if !is_empty_initial(&value.expr) => {
                    self.record(value.span().start().line, Effect::Write);
                }
                (None, Some(rest)) if !is_empty_initial(rest) => {
                    self.record(rest.span().start().line, Effect::Write);
                }
                _ => {}
            }
        }
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_pat_struct(&mut self, node: &'ast syn::PatStruct) {
        if self.names_owner(&node.path) {
            for pattern in &node.fields {
                if matches!(&pattern.member, syn::Member::Named(name) if name == self.field)
                    && !matches!(*pattern.pat, syn::Pat::Wild(_))
                {
                    let why = "bound by a pattern".to_owned();
                    self.record(pattern.span().start().line, Effect::Unverified(why));
                }
            }
        }
        syn::visit::visit_pat_struct(self, node);
    }

    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        if self.reaches(&node.left) {
            self.record(node.span().start().line, Effect::Write);
        }
        syn::visit::visit_expr_assign(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast syn::ExprBinary) {
        if is_compound_assignment(&node.op) && self.reaches(&node.left) {
            self.record(node.span().start().line, Effect::Write);
        }
        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_expr_reference(&mut self, node: &'ast syn::ExprReference) {
        if self.reaches(&node.expr) {
            let why = if node.mutability.is_some() {
                "borrowed with &mut"
            } else {
                "borrowed with &"
            };
            self.record(node.span().start().line, Effect::Unverified(why.to_owned()));
        }
        syn::visit::visit_expr_reference(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = node.method.to_string();
        let read = READ_METHODS.contains(&method.as_str());
        let pass = PASS_METHODS.contains(&method.as_str());
        if !read && !pass && self.reaches(&node.receiver) {
            let effect = if WRITE_METHODS.contains(&method.as_str()) {
                Effect::Write
            } else {
                Effect::Unverified(format!("calls `{method}`, not a known read"))
            };
            self.record(node.span().start().line, effect);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        // A standard macro's arguments are expressions; any other macro may
        // write a place it is given.
        let known = node
            .path
            .segments
            .last()
            .is_some_and(|segment| VALUE_MACROS.contains(&segment.ident.to_string().as_str()));
        let parser = Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
        match node.parse_body_with(parser) {
            Ok(arguments) if known => {
                for argument in &arguments {
                    self.visit_expr(argument);
                }
            }
            _ => {
                if matches!(self.owner, Owner::Struct(_)) && self.mentions(node.tokens.clone()) {
                    let why = "inside a macro the checker cannot read".to_owned();
                    self.record(node.span().start().line, Effect::Unverified(why));
                }
            }
        }
    }
}

fn is_self(expr: &syn::Expr) -> bool {
    matches!(expr, syn::Expr::Path(path) if path.path.is_ident("self"))
}

fn is_compound_assignment(op: &syn::BinOp) -> bool {
    matches!(
        op,
        syn::BinOp::AddAssign(_)
            | syn::BinOp::SubAssign(_)
            | syn::BinOp::MulAssign(_)
            | syn::BinOp::DivAssign(_)
            | syn::BinOp::RemAssign(_)
            | syn::BinOp::BitXorAssign(_)
            | syn::BinOp::BitAndAssign(_)
            | syn::BinOp::BitOrAssign(_)
            | syn::BinOp::ShlAssign(_)
            | syn::BinOp::ShrAssign(_)
    )
}

/// A constructor's starting value: a literal, `None`, a default, an empty
/// call, or a `new`/`signal`/`var`/`default` call of an empty value.
fn is_empty_initial(expr: &syn::Expr) -> bool {
    match expr {
        syn::Expr::Lit(_) => true,
        syn::Expr::Paren(inner) => is_empty_initial(&inner.expr),
        syn::Expr::Group(inner) => is_empty_initial(&inner.expr),
        syn::Expr::Path(path) => path.path.segments.last().is_some_and(|s| s.ident == "None"),
        syn::Expr::Call(call) => {
            let name = match call.func.as_ref() {
                syn::Expr::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
                _ => None,
            };
            call.args.is_empty()
                || (name.as_deref().is_some_and(is_constructor)
                    && call.args.first().is_some_and(is_empty_initial))
        }
        syn::Expr::MethodCall(call) => {
            is_constructor(&call.method.to_string())
                && (call.args.is_empty() || call.args.first().is_some_and(is_empty_initial))
        }
        syn::Expr::Block(block) => is_seeded_block(&block.block),
        _ => false,
    }
}

/// `{ let mut x = <empty>; x.update(<empty>); x }`: a cache seeded with
/// empty values in a constructor.
fn is_seeded_block(block: &syn::Block) -> bool {
    let Some((tail, statements)) = block.stmts.split_last() else {
        return false;
    };
    let syn::Stmt::Expr(syn::Expr::Path(_), None) = tail else {
        return false;
    };
    statements.iter().all(|statement| match statement {
        syn::Stmt::Local(local) => local
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_none() && is_empty_initial(&init.expr)),
        syn::Stmt::Expr(syn::Expr::MethodCall(call), Some(_)) => {
            matches!(call.receiver.as_ref(), syn::Expr::Path(_))
                && call.args.iter().all(is_empty_initial)
        }
        _ => false,
    })
}

fn is_constructor(name: &str) -> bool {
    matches!(name, "new" | "signal" | "var" | "default")
}

/// The last segment of an impl's self type.
fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        syn::Type::Reference(reference) => type_name(&reference.elem),
        syn::Type::Paren(paren) => type_name(&paren.elem),
        syn::Type::Group(group) => type_name(&group.elem),
        _ => None,
    }
}
