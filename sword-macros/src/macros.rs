#[proc_macro_attribute]
pub fn get(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    crate::adapters::rest::attributes::attribute("GET", attr, item)
}

#[proc_macro_attribute]
pub fn post(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    crate::adapters::rest::attributes::attribute("POST", attr, item)
}

#[proc_macro_attribute]
pub fn put(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    crate::adapters::rest::attributes::attribute("PUT", attr, item)
}

#[proc_macro_attribute]
pub fn delete(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    crate::adapters::rest::attributes::attribute("DELETE", attr, item)
}

#[proc_macro_attribute]
pub fn patch(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    crate::adapters::rest::attributes::attribute("PATCH", attr, item)
}
