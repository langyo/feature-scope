use syn::parse::{Parse, ParseStream};

#[derive(Debug, Clone)]
pub struct FeatureScopeDefault {}

impl Parse for FeatureScopeDefault {
    fn parse(_: ParseStream) -> syn::Result<Self> {
        Ok(FeatureScopeDefault {})
    }
}
