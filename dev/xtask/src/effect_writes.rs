//! Role: Keep Signal and Var writes out of framework effect closures.
//!
//! Framework effects derive rendering; state changes belong in handlers or
//! Machine transitions. User application effects are deliberately unrestricted.
//! This syntax check follows typed parameters, self fields, constructors and
//! clone bindings within lexical scopes. It cannot infer opaque helper return
//! types or writes hidden inside helper calls; runtime purity covers memos.
use crate::check_source::{Source, type_names};
use std::collections::{BTreeMap, BTreeSet};
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) fn check(source: &Source, errors: &mut Vec<String>) {
    let path = source.path.to_string_lossy().replace('\\', "/");
    if !(path.starts_with("crates/") || path.contains("/crates/"))
        || !(source.package.starts_with("kivact-") || source.package == "kivact")
    {
        return;
    }
    let fields = source
        .items
        .iter()
        .filter_map(|item| {
            let syn::Item::Struct(item) = item else {
                return None;
            };
            Some((
                item.ident.to_string(),
                item.fields
                    .iter()
                    .filter(|f| signal_type(&f.ty))
                    .filter_map(|f| f.ident.as_ref().map(ToString::to_string))
                    .collect(),
            ))
        })
        .collect();
    let mut visitor = Effects {
        source,
        errors,
        depth: 0,
        signals: BTreeMap::new(),
        fields,
        self_fields: BTreeSet::new(),
    };
    for item in &source.items {
        if !matches!(item, syn::Item::Mod(_)) {
            visitor.visit_item(item);
        }
    }
}
fn signal_type(ty: &syn::Type) -> bool {
    type_names(ty).iter().any(|n| n == "Signal" || n == "Var")
}
struct Effects<'a> {
    source: &'a Source,
    errors: &'a mut Vec<String>,
    depth: usize,
    signals: BTreeMap<String, bool>,
    fields: BTreeMap<String, BTreeSet<String>>,
    self_fields: BTreeSet<String>,
}
impl Effects<'_> {
    fn signal_value(&self, expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::Call(call) => {
                matches!(call.func.as_ref(), syn::Expr::Path(p) if p.path.segments.iter().any(|s| matches!(s.ident.to_string().as_str(), "signal" | "var" | "Signal" | "Var")))
            }
            syn::Expr::MethodCall(call) if call.method == "clone" => {
                self.signal_value(&call.receiver)
            }
            syn::Expr::Path(p) => p
                .path
                .segments
                .last()
                .is_some_and(|s| self.signals.get(&s.ident.to_string()) == Some(&true)),
            syn::Expr::Field(f) => {
                matches!(f.base.as_ref(), syn::Expr::Path(p) if p.path.is_ident("self"))
                    && matches!(&f.member, syn::Member::Named(n) if self.self_fields.contains(&n.to_string()))
            }
            syn::Expr::Reference(r) => self.signal_value(&r.expr),
            syn::Expr::Paren(p) => self.signal_value(&p.expr),
            _ => false,
        }
    }
    fn bind(&mut self, pattern: &syn::Pat, signal: bool) {
        match pattern {
            syn::Pat::Ident(name) => {
                self.signals.insert(name.ident.to_string(), signal);
            }
            syn::Pat::Type(typed) => self.bind(&typed.pat, signal_type(&typed.ty)),
            syn::Pat::Tuple(tuple) => {
                for element in &tuple.elems {
                    self.bind(element, false);
                }
            }
            _ => {}
        }
    }
    fn function(&mut self, signature: &syn::Signature, block: &syn::Block) {
        let previous = std::mem::take(&mut self.signals);
        for arg in &signature.inputs {
            if let syn::FnArg::Typed(arg) = arg {
                self.bind(&arg.pat, signal_type(&arg.ty));
            }
        }
        self.visit_block(block);
        self.signals = previous;
    }
}
impl<'ast> Visit<'ast> for Effects<'_> {
    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        self.function(&item.sig, &item.block);
    }
    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        let previous = std::mem::take(&mut self.self_fields);
        if let syn::Type::Path(path) = item.self_ty.as_ref() {
            if let Some(name) = path.path.segments.last() {
                self.self_fields = self
                    .fields
                    .get(&name.ident.to_string())
                    .cloned()
                    .unwrap_or_default();
            }
        }
        syn::visit::visit_item_impl(self, item);
        self.self_fields = previous;
    }
    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        self.function(&item.sig, &item.block);
    }
    fn visit_block(&mut self, block: &'ast syn::Block) {
        let previous = self.signals.clone();
        syn::visit::visit_block(self, block);
        self.signals = previous;
    }
    fn visit_local(&mut self, local: &'ast syn::Local) {
        let signal = local
            .init
            .as_ref()
            .is_some_and(|i| self.signal_value(&i.expr));
        syn::visit::visit_local(self, local);
        self.bind(&local.pat, signal);
    }
    fn visit_expr_closure(&mut self, closure: &'ast syn::ExprClosure) {
        let previous = self.signals.clone();
        for arg in &closure.inputs {
            self.bind(arg, false);
        }
        self.visit_expr(&closure.body);
        self.signals = previous;
    }
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let effect = matches!(call.func.as_ref(), syn::Expr::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "effect" || s.ident == "effect_on"));
        for arg in &call.args {
            let closure = effect && matches!(arg, syn::Expr::Closure(_));
            if closure {
                self.depth += 1;
            }
            self.visit_expr(arg);
            if closure {
                self.depth -= 1;
            }
        }
        self.visit_expr(&call.func);
    }
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if self.depth > 0
            && matches!(call.method.to_string().as_str(), "set" | "update")
            && self.signal_value(&call.receiver)
        {
            self.source.error(
                "effect-writes",
                call.span(),
                "Signal/Var write inside an effect closure",
                self.errors,
            );
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}
#[cfg(test)]
mod tests {
    //! Role: Check signal closures without matching unrelated writes or text.
    #[test]
    fn checks_effect_closures_only() {
        let source = crate::check_source::fixture(
            "fn setup(signal: Signal<i32>) { signal.set(0); effect(move || { signal.set(1); cell.set(2); }); effect_on(source, move |_| signal.update(|v| *v += 1)); let s = \"effect(|| x.set(0))\"; }",
        );
        let mut errors = Vec::new();
        super::check(&source, &mut errors);
        assert_eq!(errors.len(), 2);
    }
    #[test]
    fn tracks_constructors_and_clones() {
        let source = crate::check_source::fixture(
            "fn setup() { let value = signal(1); let other = value.clone(); effect(move || other.set(2)); }",
        );
        let mut errors = Vec::new();
        super::check(&source, &mut errors);
        assert_eq!(errors.len(), 1);
    }
    #[test]
    fn respects_function_block_and_closure_shadowing() {
        let source = crate::check_source::fixture(
            "fn a(value: Signal<i32>) {} fn b(value: Cell<i32>) { effect(|| value.set(1)); } fn c(value: Signal<i32>) { { let value = Cell::new(1); effect(|| value.set(1)); } effect(|| value.set(2)); effect(|value: Cell<i32>| value.set(3)); }",
        );
        let mut errors = Vec::new();
        super::check(&source, &mut errors);
        assert_eq!(errors.len(), 1);
    }
    #[test]
    fn user_effects_are_unrestricted() {
        let mut source = crate::check_source::fixture(
            "fn setup(signal: Signal<i32>) { effect(|| signal.set(1)); }",
        );
        source.path = "examples/kivact/demo/src/main.rs".into();
        let mut errors = Vec::new();
        super::check(&source, &mut errors);
        assert!(errors.is_empty());
    }
}
