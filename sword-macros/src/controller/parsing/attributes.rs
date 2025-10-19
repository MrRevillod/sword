use regex_lite::Regex;
use std::sync::LazyLock;
use syn::{
    Ident, LitStr, Token,
    parse::{Parse, ParseStream},
};

static VERSION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"v\d+").expect("Failed to compile version regex"));

// #[controller("/", version = "v1")]
pub struct ControllerArgs {
    pub base_path: String,
    pub version: Option<String>,
}

impl Parse for ControllerArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let base_path = input.parse::<LitStr>()?.value();
        let mut version = None;

        if input.parse::<Token![,]>().is_ok() && input.peek(Ident) {
            let ident = input.parse::<Ident>()?;

            if ident == "version" {
                input.parse::<Token![=]>()?;
                let ver = input.parse::<LitStr>()?;
                let ver_str = ver.value();

                if !VERSION_REGEX.is_match(&ver_str) {
                    return Err(syn::Error::new(
                        ver.span(),
                        "Invalid version format. Expected: v<number> (e.g., v1, v2)",
                    ));
                }

                version = Some(ver_str);
            }
        }

        Ok(Self { base_path, version })
    }
}
