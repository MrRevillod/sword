use syn::{Block, FnArg, ItemFn, ReturnType};

pub struct OnMiddlewareFunction {
    pub fn_body: Block,
    pub fn_arguments: Vec<FnArg>,
    pub fn_output: ReturnType,
}

pub fn parse_on_middleware_function(
    input: &ItemFn,
) -> Result<OnMiddlewareFunction, syn::Error> {
    let args: Vec<FnArg> = input.sig.inputs.iter().cloned().collect();

    Ok(OnMiddlewareFunction {
        fn_body: *input.block.clone(),
        fn_arguments: args,
        fn_output: input.sig.output.clone(),
    })
}
