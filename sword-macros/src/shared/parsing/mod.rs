mod adapters;

pub use adapters::CommonHttpAdapterInput;
use syn::{Fields, Ident, ItemStruct, Type};

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

use syn::{FnArg, ImplItemFn, Pat, PatIdent, PatType};

/// Extracts function arguments (excluding `self`) from a handler function.
/// Returns a vector of (identifier, type) tuples.
///
/// This is shared logic used by both REST and SocketIO adapters.
pub fn extract_function_args(func: &ImplItemFn) -> Vec<(Ident, Type)> {
    func.sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let ident = extract_pat_ident(pat)?;
                Some((ident, (**ty).clone()))
            }
        })
        .collect()
}

fn extract_pat_ident(pat: &Pat) -> Option<Ident> {
    match pat {
        Pat::Ident(PatIdent { ident, .. }) => Some(ident.clone()),
        Pat::TupleStruct(tuple_struct) => tuple_struct.path.get_ident().cloned(),
        _ => None,
    }
}
