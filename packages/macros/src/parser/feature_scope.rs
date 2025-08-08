use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

#[derive(Debug, Clone)]
pub struct FeatureScope {
    pub ident: Ident,
}

impl Parse for FeatureScope {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let ident = Ident::new(&format!("__scope_{}", ident), ident.span());
        Ok(FeatureScope { ident })
    }
}
