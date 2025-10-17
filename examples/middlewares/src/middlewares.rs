// use sword::prelude::*;

// pub struct ExtensionsTestMiddleware;

// impl Middleware for ExtensionsTestMiddleware {
//     async fn handle(mut ctx: Context, next: Next) -> MiddlewareResult {
//         ctx.extensions
//             .insert::<String>("test_extension".to_string());

//         next!(ctx, next)
//     }
// }

// pub struct RoleMiddleware;

// impl MiddlewareWithConfig<Vec<&str>> for RoleMiddleware {
//     async fn handle(roles: Vec<&str>, ctx: Context, next: Next) -> MiddlewareResult {
//         println!("Allowed roles: {roles:?}");
//         next!(ctx, next)
//     }
// }

// pub struct ErrorMiddleware;

// impl Middleware for ErrorMiddleware {
//     async fn handle(_ctx: Context, _next: Next) -> MiddlewareResult {
//         Err(HttpResponse::InternalServerError())
//     }
// }

// pub struct LoggerMiddleware;

// impl Middleware for LoggerMiddleware {
//     async fn handle(ctx: Context, next: Next) -> MiddlewareResult {
//         println!("Request: {} {}", ctx.method(), ctx.uri());

//         next!(ctx, next)
//     }
// }
