// use sword::prelude::*;

// use crate::TaskRepository;

// pub struct MyMiddleware {}

// impl Middleware for MyMiddleware {
//     async fn handle(ctx: Context, next: Next) -> MiddlewareResult {
//         let task_repo = ctx.di::<TaskRepository>()?;
//         let tasks = task_repo.find_all().await;

//         println!("Current tasks:");

//         match tasks {
//             Some(tasks) => println!("{tasks:?}"),
//             None => println!("There's no tasks"),
//         }

//         next!(ctx, next)
//     }
// }
