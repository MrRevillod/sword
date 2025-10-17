use syn::{
    Expr, Path,
    parse::{Parse, ParseStream},
};

pub enum MiddlewareArgs {
    SwordSimple(Path),
    /// Any expression (Tower layer or anything else)
    Expression(Expr),
}

impl Parse for MiddlewareArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Some(result) = parse_as_sword_middleware(input)? {
            return Ok(result);
        }

        Ok(MiddlewareArgs::Expression(input.parse()?))
    }
}

fn parse_as_sword_middleware(
    input: ParseStream,
) -> syn::Result<Option<MiddlewareArgs>> {
    let fork = input.fork();

    let _ = match fork.parse::<Path>() {
        Ok(path) => path,
        Err(_) => return Ok(None),
    };

    if fork.is_empty() {
        return Ok(Some(MiddlewareArgs::SwordSimple(input.parse()?)));
    }

    Ok(None)
}
