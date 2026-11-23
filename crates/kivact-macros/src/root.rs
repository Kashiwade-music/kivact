//! Role: Find the path the generated code names Kivact by.

use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;

/// `::kivact_core` in a crate that depends on kivact-core (kivact-ui and
/// kivact-interaction), else `::kivact` (apps and kits); an error naming
/// the missing dependency otherwise. kivact-core is checked first: those
/// two crates also have `kivact` as a dev-dependency, which proc-macro-crate
/// does not tell apart, and no other crate may depend on kivact-core
/// (`visible_to`).
pub(crate) fn root() -> syn::Result<TokenStream> {
    for name in ["kivact-core", "kivact"] {
        match crate_name(name) {
            Ok(FoundCrate::Name(found)) => {
                let ident = syn::Ident::new(&found, Span::call_site());
                return Ok(quote! { ::#ident });
            }
            Ok(FoundCrate::Itself) => {
                let ident = syn::Ident::new(&name.replace('-', "_"), Span::call_site());
                return Ok(quote! { ::#ident });
            }
            Err(_) => {}
        }
    }
    Err(syn::Error::new(
        Span::call_site(),
        "#[component] needs the `kivact` crate as a dependency",
    ))
}
