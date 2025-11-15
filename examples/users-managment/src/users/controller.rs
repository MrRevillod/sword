use std::sync::Arc;
use sword::prelude::*;
use uuid::Uuid;

use crate::{
    shared::{errors::AppError, Hasher},
    users::*,
};

#[controller("/users")]
pub struct UsersController {
    hasher: Arc<Hasher>,
    users: Arc<UserRepository>,
}

#[routes]
impl UsersController {
    #[get("/")]
    async fn get_users(&self) -> HttpResult {
        let data = self.users.find_all().await?;

        Ok(HttpResponse::Ok().data(data))
    }

    #[post("/")]
    async fn create_user(&self, req: Request) -> HttpResult {
        let body = req.body_validator::<CreateUserDto>()?;

        let user = User {
            id: Uuid::new_v4(),
            username: body.username,
            password: self.hasher.hash(&body.password)?,
        };

        self.users.save(&user).await?;

        Ok(HttpResponse::Created().message("User created").data(user))
    }

    #[put("/{id}")]
    async fn update_user(&self, req: Request) -> HttpResult {
        let id = req.param::<Uuid>("id")?;
        let body = req.body_validator::<UpdateUserDto>()?;

        let Some(existing_user) = self.users.find_by_id(&id).await? else {
            return Err(AppError::NotFoundError("User not found"))?;
        };

        let username = body.username.unwrap_or(existing_user.username.clone());

        let password = match &body.password {
            Some(pwd) => self.hasher.hash(pwd)?,
            None => existing_user.password.clone(),
        };

        let updated_user = User {
            id,
            username,
            password,
        };

        self.users.save(&updated_user).await?;

        Ok(HttpResponse::Ok().message("User updated"))
    }

    #[delete("/{id}")]
    async fn delete_user(&self, req: Request) -> HttpResult {
        let id = req.param::<Uuid>("id")?;

        let Some(_) = self.users.find_by_id(&id).await? else {
            return Err(AppError::NotFoundError("User not found"))?;
        };

        self.users.delete(&id).await?;

        Ok(HttpResponse::Ok().message("User deleted"))
    }
}
