use syn::{
    Expr, Path, Token,
    parse::{Parse, ParseStream},
};

pub enum MiddlewareArgs {
    SwordSimple(Path),
    SwordWithConfig {
        middleware: Path,
        config: Expr,
    },
    /// Any expression (Tower layer or anything else)
    Expression(Expr),
}

impl Parse for MiddlewareArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if let Some(result) = parse_as_sword_middleware(input)? {
            return Ok(result);
        }

        Ok(Self::Expression(input.parse()?))
    }
}

fn parse_as_sword_middleware(
    input: ParseStream,
) -> syn::Result<Option<MiddlewareArgs>> {
    let fork = input.fork();

    let Ok(_) = fork.parse::<Path>() else {
        return Ok(None);
    };

    if fork.is_empty() {
        return Ok(Some(MiddlewareArgs::SwordSimple(input.parse()?)));
    }

    if fork.peek(Token![,]) {
        let config_fork = fork;

        // Check , config = expr
        if config_fork.parse::<Token![,]>().is_ok()
            && config_fork.peek(syn::Ident)
            && config_fork.peek2(Token![=])
        {
            if let Ok(ident) = config_fork.parse::<syn::Ident>() {
                if ident == "config"
                    && config_fork.parse::<Token![=]>().is_ok()
                    && config_fork.parse::<Expr>().is_ok()
                {
                    let path: Path = input.parse()?;

                    input.parse::<Token![,]>()?; // ,
                    input.parse::<syn::Ident>()?; // config
                    input.parse::<Token![=]>()?; // =

                    return Ok(Some(MiddlewareArgs::SwordWithConfig {
                        middleware: path,
                        config: input.parse()?,
                    }));
                }
            }
        }
    }

    Ok(None)
}
