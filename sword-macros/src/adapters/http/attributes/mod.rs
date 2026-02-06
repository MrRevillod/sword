mod generation;
mod parsing;

use proc_macro::TokenStream;

pub use generation::generate_route;
pub use parsing::ParsedRouteAttribute;

pub fn attribute(
    attribute_str: &str,
    attr: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let parsed = match ParsedRouteAttribute::parse(attribute_str, attr, item) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    generate_route(parsed)
}
