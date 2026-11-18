//! Role: Index which types of each package their own methods mutate after construction.
//!
//! Types are keyed by `package::Name`: two types of one name in one package
//! share their facts (an approximation the rule accepts).

use crate::check_source::{Source, self_type_name};
use std::collections::BTreeSet;
use syn::visit::Visit;

/// Method calls that write the value they are called on.
const WRITE_METHODS: [&str; 10] = [
    "set",
    "update",
    "replace",
    "take",
    "swap",
    "set_untracked",
    "update_untracked",
    "set_if_changed",
    "borrow_mut",
    "get_mut",
];

/// The mutation facts of the workspace.
#[derive(Default)]
pub(crate) struct Index {
    /// `package::Type` of every type that an impl of its package gives a `&mut self` method.
    mutated: BTreeSet<String>,
    /// `package::Type::field` for every `self.field.<write>(..)` in a method of the type.
    written: BTreeSet<String>,
}

impl Index {
    pub(crate) fn build(sources: &[Source]) -> Self {
        let mut index = Self::default();
        for source in sources {
            for item in &source.items {
                if let syn::Item::Impl(item) = item {
                    index.add_impl(&source.package, item);
                }
            }
        }
        index
    }

    fn add_impl(&mut self, package: &str, item: &syn::ItemImpl) {
        let Some(name) = self_type_name(&item.self_ty) else {
            return;
        };
        let key = format!("{package}::{name}");
        for member in &item.items {
            let syn::ImplItem::Fn(method) = member else {
                continue;
            };
            let Some(receiver) = method.sig.receiver() else {
                continue;
            };
            if matches!(receiver.ty.as_ref(), syn::Type::Reference(reference) if reference.mutability.is_some())
            {
                self.mutated.insert(key.clone());
            }
            if receiver.reference.is_none() && returns_self(&method.sig, &name) {
                continue;
            }
            let mut writes = SelfWrites::default();
            writes.visit_block(&method.block);
            for field in writes.fields {
                self.written.insert(format!("{key}::{field}"));
            }
        }
    }

    pub(crate) fn is_mutated(&self, package: &str, name: &str) -> bool {
        self.mutated.contains(&format!("{package}::{name}"))
    }

    pub(crate) fn is_written(&self, package: &str, name: &str, field: &str) -> bool {
        self.written
            .contains(&format!("{package}::{name}::{field}"))
    }
}

/// A by-value builder: `fn x(mut self, ..) -> Self`.
fn returns_self(signature: &syn::Signature, name: &str) -> bool {
    let syn::ReturnType::Type(_, ty) = &signature.output else {
        return false;
    };
    self_type_name(ty).is_some_and(|returned| returned == "Self" || returned == name)
}

/// The fields of `self` that one method body writes.
#[derive(Default)]
struct SelfWrites {
    fields: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for SelfWrites {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if WRITE_METHODS.contains(&call.method.to_string().as_str())
            && let Some(field) = self_field(&call.receiver)
        {
            self.fields.insert(field);
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

/// `x` when `expr` is `self.x`, `self.x.y`, `&self.x` or `(self.x)`.
fn self_field(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Field(field) => match field.base.as_ref() {
            syn::Expr::Path(path) if path.path.is_ident("self") => match &field.member {
                syn::Member::Named(name) => Some(name.to_string()),
                syn::Member::Unnamed(index) => Some(index.index.to_string()),
            },
            base => self_field(base),
        },
        syn::Expr::Reference(reference) => self_field(&reference.expr),
        syn::Expr::Paren(paren) => self_field(&paren.expr),
        _ => None,
    }
}
