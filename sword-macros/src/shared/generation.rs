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
                        ::sword::core::DependencyInjectionError::DependencyNotFound {
                            type_name: #type_str.to_string(),
                        }
                    })?;
            }
        } else {
            let type_str = quote!(#field_type).to_string();

            quote! {
                let #field_name = <#field_type as ::sword::core::FromState>::from_state(state)
                    .map_err(|_| {
                        ::sword::core::DependencyInjectionError::DependencyNotFound {
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

/// Generates the implementation of the Build trait for a component.
///
/// This generator is used by all macros (#[middleware], #[injectable], etc.)
/// to create the `build()` method that constructs an instance from the State.
///
/// The generated code extracts dependencies from the State and uses them to
/// construct the component instance.
pub fn gen_build(
    name: &Ident,
    fields: &[(Ident, Type)],
    error_type: &TokenStream,
) -> TokenStream {
    let extracts = generate_field_extractions(fields);
    let assigns = generate_field_assignments(fields);

    quote! {
        impl ::sword::core::Build for #name {
            type Error = #error_type;

            fn build(state: &::sword::core::State) -> Result<Self, Self::Error>
            where
                Self: Sized,
            {
                #extracts

                Ok(Self { #assigns })
            }
        }
    }
}

/// Generates the implementation of the Clone trait for a component.
///
/// This generator creates an explicit Clone implementation that clones each
/// field individually, useful for components with Arc-wrapped dependencies.
pub fn gen_clone(name: &Ident, fields: &[(Ident, Type)]) -> TokenStream {
    let clones = fields.iter().map(|(field_name, _)| {
        quote! { #field_name: self.#field_name.clone() }
    });

    quote! {
        impl Clone for #name {
            fn clone(&self) -> Self {
                Self {
                    #(#clones),*
                }
            }
        }
    }
}

/// Generates the implementation of the `HasDeps` trait for a component.
///
/// This generator creates the `deps()` method that returns the `TypeId` of allS
/// component dependencies, allowing the DI system to resolve the correct
/// construction order using topological sorting.
pub fn gen_deps(name: &Ident, fields: &[(Ident, Type)]) -> TokenStream {
    let dep_types = fields.iter().map(|(_, field_type)| {
        if let Some(inner_type) = extract_arc_inner_type(field_type) {
            quote! { ::std::any::TypeId::of::<#inner_type>() }
        } else {
            quote! { ::std::any::TypeId::of::<#field_type>() }
        }
    });

    quote! {
        impl ::sword::core::HasDeps for #name {
            fn deps() -> Vec<::std::any::TypeId> {
                vec![#(#dep_types),*]
            }
        }
    }
}
