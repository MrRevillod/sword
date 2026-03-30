use crate::{common::CMetaStack, interceptor::InterceptorArgs};
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, ItemFn, LitStr, Type};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RequestMode {
    None,
    Buffered,
    Streaming,
}

pub struct ParsedRouteAttribute {
    pub method: String,
    pub path: String,
    pub function: ItemFn,
    pub interceptors: Vec<InterceptorArgs>,
    pub request_mode: RequestMode,
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
        let request_mode = Self::detect_request_mode(&input_fn)?;

        let (interceptors, retained_attrs) = Self::extract_interceptors(&input_fn)?;
        input_fn.attrs = retained_attrs;

        Self::validate_controller_level_interceptor_compatibility(request_mode)?;

        let (controller_name, controller_path) =
            Self::get_controller_metadata(method)?;

        Ok(Self {
            method: method.to_string(),
            path,
            function: input_fn,
            interceptors,
            request_mode,
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
                1. The struct has #[controller(kind = Controller::Web, path = \"/path\")] attribute\n\
                2. The struct is defined BEFORE the impl block\n\
                3. The impl block is for the same struct\n",
                method
            );

            return Err(syn::Error::new(Span::call_site(), error));
        };

        let controller_path = CMetaStack::get("controller_path").unwrap_or_default();

        Ok((controller_name, controller_path))
    }

    fn detect_request_mode(input_fn: &ItemFn) -> syn::Result<RequestMode> {
        let mut mode = RequestMode::None;

        for arg in &input_fn.sig.inputs {
            let syn::FnArg::Typed(pat_type) = arg else {
                continue;
            };

            let arg_mode = Self::request_mode_from_type(&pat_type.ty);

            if arg_mode == RequestMode::None {
                continue;
            }

            if mode != RequestMode::None && mode != arg_mode {
                return Err(syn::Error::new(
                    pat_type.ty.span(),
                    "A route handler cannot use both `Request` and `StreamRequest` in the same signature",
                ));
            }

            mode = arg_mode;
        }

        Ok(mode)
    }

    fn request_mode_from_type(ty: &Type) -> RequestMode {
        let Type::Path(type_path) = ty else {
            return RequestMode::None;
        };

        let Some(last_segment) = type_path.path.segments.last() else {
            return RequestMode::None;
        };

        if last_segment.ident == "Request" {
            return RequestMode::Buffered;
        }

        if last_segment.ident == "StreamRequest" {
            return RequestMode::Streaming;
        }

        RequestMode::None
    }

    fn validate_controller_level_interceptor_compatibility(
        request_mode: RequestMode,
    ) -> syn::Result<()> {
        if request_mode != RequestMode::Streaming {
            return Ok(());
        }

        let has_controller_sword_interceptors =
            CMetaStack::get("controller_has_sword_interceptors")
                .as_deref()
                .is_some_and(|value| value == "true");

        if has_controller_sword_interceptors {
            return Err(syn::Error::new(
                Span::call_site(),
                "`StreamRequest` routes cannot be used with controller-level Sword `#[interceptor(...)]` attributes. Move to expression-based layers or use `Request`.",
            ));
        }

        Ok(())
    }
}
