use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn feature_scope(attr: TokenStream, input: TokenStream) -> TokenStream {
    quote! {}.into()
}
