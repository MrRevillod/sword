use syn::{Expr, Path};

pub enum InterceptorArgs {
    SwordSimple(Path),
    SwordWithConfig {
        middleware: Path,
        config: Expr,
    },
    /// Any expression (Tower layer or anything else)
    Expression(Expr),
}
