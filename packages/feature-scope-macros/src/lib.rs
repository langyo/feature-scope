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
