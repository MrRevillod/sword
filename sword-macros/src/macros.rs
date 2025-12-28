macro_rules! http_method {
    ($($method:ident),+ $(,)?) => {
        $(
            #[proc_macro_attribute]
            pub fn $method(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
                let _ = attr;
                item
            }
        )+
    };
}
