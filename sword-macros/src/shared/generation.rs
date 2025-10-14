use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

pub fn extract_arc_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let path = &type_path.path;
    let last_segment = path.segments.last()?;

    if last_segment.ident != "Arc" {
        return None;
    }

    if !is_std_arc_path(path) {
        return None;
    }

    match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            args.args.first().and_then(|arg| match arg {
                syn::GenericArgument::Type(inner) => Some(inner),
                _ => None,
            })
        }
        _ => None,
    }
}

fn is_std_arc_path(path: &syn::Path) -> bool {
    let segments: Vec<_> = path.segments.iter().collect();

    match segments.len() {
        1 => true,

        2 => segments[0].ident == "sync",

        3 => {
            let root = &segments[0].ident;
            let mid = &segments[1].ident;
            (root == "std" || root == "alloc") && mid == "sync"
        }
        _ => false,
    }
}

pub fn generate_field_extractions(fields: &[(Ident, Type)]) -> TokenStream {
    let extractions = fields.iter().map(|(field_name, field_type)| {
        if let Some(inner_type) = extract_arc_inner_type(field_type) {
            let type_str = quote!(#inner_type).to_string();

            quote! {
                let #field_name = <#field_type as ::sword::core::FromStateArc>::from_state_arc(state)
                    .map_err(|_| {
                        ::sword::errors::DependencyInjectionError::DependencyNotFound {
                            type_name: #type_str.to_string(),
                        }
                    })?;
            }
        } else {
            let type_str = quote!(#field_type).to_string();

            quote! {
                let #field_name = <#field_type as ::sword::core::FromState>::from_state(state)
                    .map_err(|_| {
                        ::sword::errors::DependencyInjectionError::DependencyNotFound {
                            type_name: #type_str.to_string(),
                        }
                    })?;
            }
        }
    });

    quote! {
        #(#extractions)*
    }
}

pub fn generate_field_assignments(fields: &[(Ident, Type)]) -> TokenStream {
    let assignments = fields.iter().map(|(name, _)| {
        quote! { #name }
    });

    quote! {
        #(#assignments),*
    }
}
