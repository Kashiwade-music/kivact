//! Role: Forbid Machines without state: `machine(())` and `type State = ()`.

use crate::check_source::Source;
use syn::spanned::Spanned;
use syn::visit::Visit;

pub(crate) fn check(source: &Source, errors: &mut Vec<String>) {
    let mut visitor = Units { source, errors };
    for item in &source.items {
        if !matches!(item, syn::Item::Mod(_)) {
            visitor.visit_item(item);
        }
    }
}

struct Units<'a> {
    source: &'a Source,
    errors: &'a mut Vec<String>,
}

impl<'ast> Visit<'ast> for Units<'_> {
    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let named_machine = matches!(call.func.as_ref(), syn::Expr::Path(path)
            if path.path.segments.last().is_some_and(|segment| segment.ident == "machine"));
        let unit_argument = call.args.first().is_some_and(
            |argument| matches!(argument, syn::Expr::Tuple(tuple) if tuple.elems.is_empty()),
        );
        if named_machine && unit_argument {
            self.source.error_at(
                "unit-machine",
                call.span(),
                "`machine(())` builds a Machine without state; write the condition inline",
                self.errors,
            );
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_impl_item_type(&mut self, item: &'ast syn::ImplItemType) {
        if item.ident == "State"
            && matches!(&item.ty, syn::Type::Tuple(tuple) if tuple.elems.is_empty())
        {
            self.source.error_at(
                "unit-machine",
                item.span(),
                "`type State = ()` declares a Machine without state; write the condition inline",
                self.errors,
            );
        }
        syn::visit::visit_impl_item_type(self, item);
    }
}

#[cfg(test)]
mod tests {
    //! Role: Check that Machines without state are reported.
    #[test]
    fn unit_state_machines_are_reported() {
        let source = crate::check_source::fixture(
            "impl Machine for Blur { type State = (); }\nfn f() { let m = machine(()); let n = machine(0u8); }",
        );
        let mut errors = Vec::new();
        super::check(&source, &mut errors);
        assert_eq!(errors.len(), 2, "{errors:?}");
    }
}
