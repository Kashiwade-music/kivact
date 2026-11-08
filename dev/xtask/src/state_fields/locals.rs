//! Role: List function-local state guesses for the state-fields report.

use crate::check_source::Source;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(super) fn collect(source: &Source) -> Vec<String> {
    let mut visitor = Locals {
        source,
        lines: Vec::new(),
    };
    for item in &source.items {
        visitor.visit_item(item);
    }
    visitor.lines
}

struct Locals<'a> {
    source: &'a Source,
    lines: Vec<String>,
}

impl<'ast> Visit<'ast> for Locals<'_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if local
            .init
            .as_ref()
            .is_some_and(|init| is_state_guess(&init.expr))
            && let Some(name) = pattern_name(&local.pat)
        {
            let line = local.span().start().line;
            self.lines.push(format!(
                "state-locals: {}:{line} {name}",
                self.source.path.display()
            ));
        }
        syn::visit::visit_local(self, local);
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
        "new" => match segments.iter().rev().nth(1).map(String::as_str) {
            Some("Cell") => true,
            Some("Rc") => call.args.first().is_some_and(is_cell_new),
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
        syn::Expr::Lit(literal) => {
            matches!(literal.lit, syn::Lit::Bool(_) | syn::Lit::Int(_))
        }
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

fn is_none(expr: &syn::Expr) -> bool {
    let syn::Expr::Path(path) = strip(expr) else {
        return false;
    };
    path.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "None")
}
