use syn::{
    parse::{Parse, ParseStream},
    Ident,
};

#[derive(Debug, Clone)]
pub struct FeatureScopeDefault {
    pub ident: Option<Ident>,
}

impl Parse for FeatureScopeDefault {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            Ok(FeatureScopeDefault { ident: None })
        } else {
            let ident: Ident = input.parse()?;
            let ident = Ident::new(&format!("__scope_{ident}"), ident.span());
            Ok(FeatureScopeDefault { ident: Some(ident) })
        }
    }
}
