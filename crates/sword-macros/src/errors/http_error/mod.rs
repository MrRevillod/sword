mod parse;

use parse::{HttpErrorConfig, MessageValue};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, Type};

pub fn derive_http_error(input: DeriveInput) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;

    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "HttpError can only be derived for enums",
        ));
    };

    let defaults = HttpErrorConfig::from_attrs(&input.attrs)?;
    let mut from_arms = Vec::new();
    let mut variant_fns = Vec::new();

    for variant in &data.variants {
        let variant_config =
            HttpErrorConfig::parse_enum_variant_config(&variant.ident, &variant.attrs)?;

        let merged = variant_config.merged(&defaults);

        validate_variant_config(enum_name, &variant.ident, &variant.fields, &merged)?;

        variant_fns.push(generate_variant_fn(&variant.ident, &variant.fields));
        from_arms.push(generate_from_arm(
            enum_name,
            &variant.ident,
            &variant.fields,
            &merged,
        ));
    }

    Ok(quote! {
        impl From<#enum_name> for ::sword::web::JsonResponse {
            fn from(err: #enum_name) -> Self {
                let __sword_internal_error = err.to_string();

                match err {
                    #(#from_arms)*
                }
            }
        }

        impl ::sword::internal::web::IntoResponse for #enum_name {
            fn into_response(self) -> ::sword::internal::web::AxumResponse {
                ::sword::web::JsonResponse::from(self).into_response()
            }
        }

        #[allow(non_snake_case)]
        impl #enum_name {
            #(#variant_fns)*
        }
    })
}

fn validate_variant_config(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &Fields,
    config: &HttpErrorConfig,
) -> syn::Result<()> {
    if config.transparent {
        let is_single_unnamed =
            matches!(fields, Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1);

        if !is_single_unnamed {
            return Err(Error::new_spanned(
                variant_name,
                "transparent variants must have exactly one unnamed field",
            ));
        }

        return Ok(());
    }

    if config.code.is_none() {
        return Err(Error::new_spanned(
            variant_name,
            "missing `code` after merging #[http_error(...)] and #[http(...)]",
        ));
    }

    match fields {
        Fields::Named(named) => {
            if let Some(field) = message_field_name(config) {
                ensure_named_field_exists(enum_name, variant_name, named, field, "message")?;
            }

            if let Some(field) = &config.error_field {
                ensure_named_field_exists(enum_name, variant_name, named, field, "error")?;
            }

            if let Some(field) = &config.errors_field {
                ensure_named_field_exists(enum_name, variant_name, named, field, "errors")?;
            }
        }
        Fields::Unnamed(unnamed) => {
            if unnamed.unnamed.len() != 1 {
                return Err(Error::new_spanned(
                    variant_name,
                    "non-transparent tuple variants are only supported with exactly one field",
                ));
            }

            if message_field_name(config).is_some()
                || config.error_field.is_some()
                || config.errors_field.is_some()
            {
                return Err(Error::new_spanned(
                    variant_name,
                    "tuple variants do not support `message = field`, `error`, or `errors`; use named fields or construct the client message before creating the error",
                ));
            }
        }
        Fields::Unit => {
            if message_field_name(config).is_some()
                || config.error_field.is_some()
                || config.errors_field.is_some()
            {
                return Err(Error::new_spanned(
                    variant_name,
                    "unit variants do not support field-based `message`, `error`, or `errors`",
                ));
            }
        }
    }

    Ok(())
}

fn ensure_named_field_exists(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &syn::FieldsNamed,
    field_name: &str,
    attr_name: &str,
) -> syn::Result<()> {
    let exists = fields
        .named
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .any(|ident| ident == field_name);

    if exists {
        return Ok(());
    }

    Err(Error::new_spanned(
        variant_name,
        format!(
            "`{attr_name} = {field_name}` references a missing field on {enum_name}::{variant_name}`"
        ),
    ))
}

fn message_field_name(config: &HttpErrorConfig) -> Option<&str> {
    match &config.message {
        Some(MessageValue::Field(field)) => Some(field.as_str()),
        _ => None,
    }
}

fn generate_variant_fn(variant_name: &Ident, fields: &Fields) -> TokenStream {
    let Fields::Named(named) = fields else {
        return quote! {};
    };

    let fn_name = variant_name;

    let params = named.named.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        if is_string_type(ty) {
            quote! { #name: impl Into<String> }
        } else {
            quote! { #name: #ty }
        }
    });

    let field_assignments = named.named.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        if is_string_type(ty) {
            quote! { #name: #name.into() }
        } else {
            quote! { #name }
        }
    });

    quote! {
        #[allow(non_snake_case)]
        pub fn #fn_name(#(#params),*) -> Self {
            Self::#variant_name {
                #(#field_assignments),*
            }
        }
    }
}

fn is_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }

    false
}

fn generate_from_arm(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &Fields,
    config: &HttpErrorConfig,
) -> TokenStream {
    if config.transparent {
        return quote! {
            #enum_name::#variant_name(inner) => ::sword::web::JsonResponse::from(inner),
        };
    }

    let pattern = generate_pattern(enum_name, variant_name, fields);
    let tracing_stmt = generate_tracing_stmt(variant_name, config, fields);
    let builder = generate_json_builder(config);

    quote! {
        #pattern => {
            #tracing_stmt
            #builder
        },
    }
}

fn generate_pattern(enum_name: &Ident, variant_name: &Ident, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let field_names: Vec<_> = named
                .named
                .iter()
                .map(|field| field.ident.as_ref())
                .collect();
            quote! { #enum_name::#variant_name { #(#field_names),* } }
        }
        Fields::Unnamed(_) => {
            quote! { #enum_name::#variant_name(_inner) }
        }
        Fields::Unit => {
            quote! { #enum_name::#variant_name }
        }
    }
}

fn generate_json_builder(config: &HttpErrorConfig) -> TokenStream {
    let status_code = config.code.as_ref().unwrap().as_u16();

    let message_expr = match &config.message {
        Some(MessageValue::Static(message)) => quote! { #message },
        Some(MessageValue::Field(field_name)) => {
            let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
            quote! { format!("{}", #field_ident) }
        }
        None => {
            let default_message = config.default_message();
            quote! { #default_message }
        }
    };

    let mut builder = quote! {
        ::sword::web::JsonResponse::status(#status_code).message(#message_expr)
    };

    if let Some(field_name) = &config.error_field {
        let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
        builder = quote! { #builder.error(&#field_ident) };
    }

    if let Some(field_name) = &config.errors_field {
        let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
        builder = quote! { #builder.errors(&#field_ident) };
    }

    builder
}

fn generate_tracing_stmt(
    variant_name: &Ident,
    config: &HttpErrorConfig,
    fields: &Fields,
) -> TokenStream {
    let Some(level) = &config.tracing_level else {
        return quote! {};
    };

    let tracing_macro = match level.as_str() {
        "trace" => quote! { ::sword::internal::tracing::trace },
        "debug" => quote! { ::sword::internal::tracing::debug },
        "info" => quote! { ::sword::internal::tracing::info },
        "warn" => quote! { ::sword::internal::tracing::warn },
        "error" => quote! { ::sword::internal::tracing::error },
        _ => return quote! {},
    };

    let variant_str = variant_name.to_string();
    let status_code = config.code.as_ref().unwrap().as_u16();

    match fields {
        Fields::Named(named) => {
            let field_logs = named.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                quote! { #field_name = ?#field_name, }
            });

            quote! {
                #tracing_macro!(
                    error = %__sword_internal_error,
                    error_type = #variant_str,
                    status_code = #status_code,
                    #(#field_logs)*
                    "HTTP error response"
                );
            }
        }
        Fields::Unnamed(_) => {
            quote! {
                #tracing_macro!(
                    error = %__sword_internal_error,
                    inner = ?_inner,
                    error_type = #variant_str,
                    status_code = #status_code,
                    "HTTP error response"
                );
            }
        }
        Fields::Unit => {
            quote! {
                #tracing_macro!(
                    error = %__sword_internal_error,
                    error_type = #variant_str,
                    status_code = #status_code,
                    "HTTP error response"
                );
            }
        }
    }
}
