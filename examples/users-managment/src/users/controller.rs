use std::sync::Arc;
use sword::prelude::*;
use uuid::Uuid;

use crate::{shared::Hasher, users::*};

#[controller("/users")]
pub struct UsersController {
    hasher: Arc<Hasher>,
    users: Arc<UserRepository>,
}

#[routes]
impl UsersController {
    #[get("/")]
    async fn get_users(&self) -> HttpResponse {
        let data = self.users.find_all().await;

        HttpResponse::Ok().data(data)
    }

    #[post("/")]
    async fn create_user(&self, req: Request) -> HttpResult {
        let body = req.body_validator::<CreateUserDto>()?;

        let hash = self.hasher.hash(&body.password).map_err(|_| {
            HttpResponse::InternalServerError().message("Failed to hash password")
        })?;

        let user = User {
            id: Uuid::new_v4(),
            username: body.username,
            password: hash,
        };

        self.users.create(&user).await;

        Ok(HttpResponse::Created().message("User created").data(user))
    }

    #[delete("/{id}")]
    async fn delete_user(&self, req: Request) -> HttpResult {
        let id = req.param::<Uuid>("id")?;

        let Some(_) = self.users.find_by_id(&id).await else {
            return Err(HttpResponse::NotFound().message("User not found"))?;
        };

        self.users.delete(&id).await;

        Ok(HttpResponse::Ok().message("User deleted"))
    }
}
