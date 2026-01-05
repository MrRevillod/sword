mod adapters;
use std::sync::LazyLock;

pub use adapters::CommonHttpAdapterInput;

use regex_lite::Regex;
use syn::{Fields, Ident, ItemStruct, Type};

pub static PATH_KIND_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\/(?:[^\/{}:]+|\{[^*{}][^{}]*\}|\{\*[^{}]+\})*(?:\/(?:[^\/{}:]+|\{[^*{}][^{}]*\}|\{\*[^{}]+\}))*$").unwrap()
});

pub struct StructFields;

impl StructFields {
    pub fn parse(input: &ItemStruct) -> syn::Result<Vec<(Ident, Type)>> {
        let mut fields_vec = Vec::new();

        if let Fields::Unnamed(_) = input.fields {
            return Err(syn::Error::new(
                input.ident.span(),
                "Tuple structs are not supported. Please use named fields.",
            ));
        }

        if let Fields::Named(named_fields) = &input.fields {
            for field in &named_fields.named {
                if let Some(ident) = &field.ident {
                    fields_vec.push((ident.clone(), field.ty.clone()));
                }
            }
        }

        Ok(fields_vec)
    }
}
