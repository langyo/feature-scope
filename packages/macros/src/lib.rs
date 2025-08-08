mod parser;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn feature_scope(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let attr = parse_macro_input!(attr as parser::FeatureScope);

    let parser::FeatureScope { ident } = attr;
    quote! {
        #[allow(unexpected_cfgs)]
        #[cfg(#ident)]
        #input
    }
    .into()
}

#[proc_macro_attribute]
pub fn feature_scope_default(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    let attr = parse_macro_input!(_attr as parser::FeatureScopeDefault);

    if let Some(ident) = attr.ident {
        quote! {
            #[allow(unexpected_cfgs)]
            #[cfg(any(__scope_default, #ident))]
            #input
        }
        .into()
    } else {
        quote! {
            #[allow(unexpected_cfgs)]
            #[cfg(__scope_default)]
            #input
        }
        .into()
    }
}
