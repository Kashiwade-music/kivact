//! Role: Find functions that keep two or more state locals.

use crate::check_source::Source;
use syn::spanned::Spanned;
use syn::visit::Visit;

/// One detected local: its line and name.
pub(super) struct Local {
    pub(super) line: usize,
    pub(super) name: String,
}

/// One function and the state locals it declares.
pub(super) struct Function {
    pub(super) name: String,
    /// The line of the `// xtask-allow(state):` marker before the function.
    pub(super) marker: Option<usize>,
    pub(super) locals: Vec<Local>,
}

/// Report functions with two or more state locals and no marker; record the markers used.
pub(super) fn check(source: &Source, errors: &mut Vec<String>) {
    for function in scan(source) {
        if function.locals.len() < 2 {
            continue;
        }
        match function.marker {
            Some(line) => source.use_allow(line),
            None => {
                let names = function
                    .locals
                    .iter()
                    .map(|local| local.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                source.error_at_line(
                    "state",
                    function.locals[0].line,
                    &format!(
                        "function {} combines state locals {names}; close them with an enum, or put one `// xtask-allow(state): <how the combinations are closed>` line before the function",
                        function.name
                    ),
                    errors,
                );
            }
        }
    }
}

/// The detected locals of `source`, grouped by function.
fn scan(source: &Source) -> Vec<Function> {
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

struct Locals<'a> {
    source: &'a Source,
    stack: Vec<Function>,
    functions: Vec<Function>,
}

impl<'ast> Visit<'ast> for Locals<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.stack.push(Function {
            name: node.sig.ident.to_string(),
            marker: self.source.allow_line("state", node.span()),
            locals: Vec::new(),
        });
        syn::visit::visit_item_fn(self, node);
        self.finish();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.stack.push(Function {
            name: node.sig.ident.to_string(),
            marker: self.source.allow_line("state", node.span()),
            locals: Vec::new(),
        });
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
            if self.source.owns_line(line)
                && let Some(function) = self.stack.last_mut()
            {
                function.locals.push(Local { line, name });
            }
        }
        syn::visit::visit_local(self, local);
    }
}

impl Locals<'_> {
    fn finish(&mut self) {
        if let Some(function) = self.stack.pop()
            && !function.locals.is_empty()
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
        syn::Expr::Path(_) => is_none(expr),
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

/// Whether `expr` is `Option::None` (bare or qualified), not an enum variant named `None`.
fn is_none(expr: &syn::Expr) -> bool {
    let syn::Expr::Path(path) = strip(expr) else {
        return false;
    };
    let names: Vec<String> = path
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    matches!(
        names.as_slice(),
        ["None"] | ["Option", "None"] | ["std" | "core", "option", "Option", "None"]
    )
}
