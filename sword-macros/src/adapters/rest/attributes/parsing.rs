use crate::{adapters::InterceptorArgs, shared::CMetaStack};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Attribute, ItemFn, LitStr};

/// Parsed data from a route attribute
pub struct ParsedRouteAttribute {
    /// The HTTP method (GET, POST, etc.)
    pub method: String,

    /// The route path (e.g., "/", "/:id")
    pub path: String,

    /// The original function with interceptor attributes removed
    pub function: ItemFn,

    /// Parsed interceptor arguments
    pub interceptors: Vec<InterceptorArgs>,

    /// Controller metadata from CMetaStack
    pub controller_name: String,
    pub controller_path: String,
}

impl ParsedRouteAttribute {
    pub fn parse(
        method: &str,
        attr: TokenStream,
        item: TokenStream,
    ) -> syn::Result<Self> {
        if attr.is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                "Route path is required, e.g., #[get(\"/path\")]. If you want the root path, use \"/\".",
            ));
        }

        let path = match syn::parse::<LitStr>(attr) {
            Ok(lit) => lit.value(),
            Err(e) => return Err(e),
        };

        let mut input_fn = syn::parse::<ItemFn>(item)?;

        let (interceptors, retained_attrs) = Self::extract_interceptors(&input_fn)?;
        input_fn.attrs = retained_attrs;

        let (controller_name, controller_path) =
            Self::get_controller_metadata(method)?;

        Ok(Self {
            method: method.to_string(),
            path,
            function: input_fn,
            interceptors,
            controller_name,
            controller_path,
        })
    }

    fn extract_interceptors(
        input_fn: &ItemFn,
    ) -> syn::Result<(Vec<InterceptorArgs>, Vec<Attribute>)> {
        let mut interceptors = Vec::new();
        let mut retained_attrs = Vec::new();

        for attr in input_fn.attrs.iter() {
            if attr.path().is_ident("interceptor") {
                interceptors.push(attr.parse_args::<InterceptorArgs>()?);
            } else {
                retained_attrs.push(attr.clone());
            }
        }

        Ok((interceptors, retained_attrs))
    }

    fn get_controller_metadata(method: &str) -> syn::Result<(String, String)> {
        let Some(controller_name) = CMetaStack::get("controller_name") else {
            let error = format!(
                "\n[ERROR] The #[{}] attribute must be used inside a #[controller] impl block.\n\
                \n\
                Make sure:\n\
                1. The struct has #[controller(\"/path\")] attribute\n\
                2. The struct is defined BEFORE the impl block\n\
                3. The impl block is for the same struct\n",
                method
            );

            return Err(syn::Error::new(Span::call_site(), error));
        };

        let controller_path = CMetaStack::get("controller_path").unwrap_or_default();

        Ok((controller_name, controller_path))
    }
}
