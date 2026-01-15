mod generation;
mod parsing;

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemImpl;

use generation::generate_controller_routes;
use parsing::parse_routes;

pub fn expand_controller_routes(
    _: TokenStream,
    item: TokenStream,
) -> Result<TokenStream, syn::Error> {
    let mut item = syn::parse::<ItemImpl>(item)?;

    for impl_item in &mut item.items {
        if let syn::ImplItem::Fn(method) = impl_item {
            process_returns_attribute(method)?;
        }
    }

    let parsed = parse_routes(&item)?;
    let generated = generate_controller_routes(&item.self_ty, &parsed)?;

    let expanded = quote! {
        #item
        #generated
    };

    Ok(TokenStream::from(expanded))
}

fn process_returns_attribute(method: &mut syn::ImplItemFn) -> syn::Result<()> {
    let returns_attr = method.attrs.iter().position(|attr| {
        attr.path()
            .get_ident()
            .map(|ident| ident == "returns")
            .unwrap_or(false)
    });

    let Some(attr_index) = returns_attr else {
        if matches!(method.sig.output, syn::ReturnType::Default) {
            method.sig.output = syn::ReturnType::Type(
                syn::Token![->](proc_macro2::Span::call_site()),
                Box::new(syn::parse_quote! { ::sword::prelude::JsonResponse }),
            );
        }
        return Ok(());
    };

    if !matches!(method.sig.output, syn::ReturnType::Default) {
        return Err(syn::Error::new_spanned(
            &method.sig,
            "Cannot use #[returns] attribute when the function already has a return type. Remove either the #[returns] attribute or the '-> Type' return type.",
        ));
    }

    let attr = method.attrs.remove(attr_index);

    if attr.meta.require_path_only().is_ok() {
        return Err(syn::Error::new_spanned(
            attr,
            "#[returns] attribute requires a type argument. Use #[returns(HttpResult)], #[returns(Result<T, E>)], or #[returns(CustomType)]. For JsonResponse (default), simply omit the #[returns] attribute.",
        ));
    }

    let return_type: syn::Type = if let Ok(ty) = attr.parse_args::<syn::Type>() {
        ty
    } else {
        return Err(syn::Error::new_spanned(
            attr,
            "Invalid #[returns] attribute. Expected a valid type.",
        ));
    };

    method.sig.output = syn::ReturnType::Type(
        syn::Token![->](proc_macro2::Span::call_site()),
        Box::new(return_type),
    );

    Ok(())
}
