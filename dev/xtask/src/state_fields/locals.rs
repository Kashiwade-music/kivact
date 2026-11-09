//! Role: List function-local state guesses for the state-fields report.

use crate::check_source::Source;
use syn::spanned::Spanned;
use syn::visit::Visit;

/// One function's detected locals as `(line, name)`.
pub(super) type Function = Vec<(usize, String)>;

/// The detected locals of `source`, grouped by function.
///
/// A marker directly before a local records it as used and keeps it out of
/// the group (SL-7).
pub(super) fn scan(source: &Source) -> Vec<Function> {
    let mut visitor = Locals {
        source,
        stack: Vec::new(),
        functions: Vec::new(),
    };
    for item in &source.items {
        visitor.visit_item(item);
    }
    visitor.functions
}

/// The `--report` notes: one line per detected local.
pub(super) fn collect(source: &Source, functions: &[Function]) -> Vec<String> {
    functions
        .iter()
        .flatten()
        .map(|(line, name)| format!("state-locals: {}:{line} {name}", source.path.display()))
        .collect()
}

/// The `Strict` errors: a function with two or more detected locals (SL-7).
pub(super) fn check(source: &Source, functions: &[Function], errors: &mut Vec<String>) {
    for function in functions {
        if function.len() < 2 {
            continue;
        }
        let names = function
            .iter()
            .map(|(_, name)| name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        source.error_at_line(
            "state-fields-strict",
            function[0].0,
            &format!("function combines state locals: {names}"),
            errors,
        );
    }
}

struct Locals<'a> {
    source: &'a Source,
    stack: Vec<Function>,
    functions: Vec<Function>,
}

impl<'ast> Visit<'ast> for Locals<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.stack.push(Vec::new());
        syn::visit::visit_item_fn(self, node);
        self.finish();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.stack.push(Vec::new());
        syn::visit::visit_impl_item_fn(self, node);
        self.finish();
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if local
            .init
            .as_ref()
            .is_some_and(|init| is_state_guess(&init.expr))
            && let Some(name) = pattern_name(&local.pat)
        {
            let line = local.span().start().line;
            if self.source.owns_line(line) {
                match self.source.allow_line("state-fields", local.span()) {
                    Some(marker) => self.source.use_allow(marker),
                    None => {
                        if let Some(function) = self.stack.last_mut() {
                            function.push((line, name));
                        }
                    }
                }
            }
        }
        syn::visit::visit_local(self, local);
    }
}

impl Locals<'_> {
    fn finish(&mut self) {
        if let Some(function) = self.stack.pop()
            && !function.is_empty()
        {
            self.functions.push(function);
        }
    }
}

fn pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(ident) => Some(ident.ident.to_string()),
        syn::Pat::Type(typed) => pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => pattern_name(&paren.pat),
        _ => None,
    }
}

fn is_state_guess(expr: &syn::Expr) -> bool {
    let expr = strip(expr);
    let syn::Expr::Call(call) = expr else {
        return false;
    };
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    let segments: Vec<_> = path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    let Some(last) = segments.last() else {
        return false;
    };
    match last.as_str() {
        "signal" => call.args.first().is_some_and(is_scalar),
        "var" => call.args.first().is_some(),
        "new" => match segments.iter().rev().nth(1).map(String::as_str) {
            Some("Cell") => true,
            Some("Rc") => {
                call.args.first().is_some_and(is_cell_new)
                    || call.args.first().is_some_and(is_refcell_new_none)
            }
            Some("RefCell") => call.args.first().is_some_and(is_none),
            _ => false,
        },
        _ => false,
    }
}

fn strip(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Reference(reference) => strip(&reference.expr),
        syn::Expr::Paren(paren) => strip(&paren.expr),
        syn::Expr::Group(group) => strip(&group.expr),
        _ => expr,
    }
}

fn is_scalar(expr: &syn::Expr) -> bool {
    match strip(expr) {
        syn::Expr::Lit(literal) => matches!(
            literal.lit,
            syn::Lit::Bool(_) | syn::Lit::Int(_) | syn::Lit::Float(_)
        ),
        syn::Expr::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "None"),
        _ => false,
    }
}

fn is_cell_new(expr: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = strip(expr) else {
        return false;
    };
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    let mut segments = path.path.segments.iter().rev();
    segments
        .next()
        .is_some_and(|segment| segment.ident == "new")
        && segments
            .next()
            .is_some_and(|segment| segment.ident == "Cell")
}

fn is_refcell_new_none(expr: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = strip(expr) else {
        return false;
    };
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    let mut segments = path.path.segments.iter().rev();
    segments
        .next()
        .is_some_and(|segment| segment.ident == "new")
        && segments
            .next()
            .is_some_and(|segment| segment.ident == "RefCell")
        && call.args.first().is_some_and(is_none)
}

fn is_none(expr: &syn::Expr) -> bool {
    let syn::Expr::Path(path) = strip(expr) else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "None")
}
