use super::entity::User;
use crate::shared::database::Database;

use std::sync::Arc;
use sword::core::injectable;
use uuid::Uuid;

#[injectable]
pub struct UserRepository {
    db: Arc<Database>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: &Uuid) -> Option<User> {
        sqlx::query_as::<_, User>(
            "SELECT id, username, password FROM users WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.db.get_pool())
        .await
        .expect("Failed to fetch user")
    }

    pub async fn find_all(&self) -> Vec<User> {
        sqlx::query_as::<_, User>("SELECT id, username, password FROM users")
            .fetch_all(self.db.get_pool())
            .await
            .expect("Failed to fetch users")
    }

    pub async fn create(&self, user: &User) {
        sqlx::query(
            "INSERT INTO users (id, username, password) VALUES ($1, $2, $3)",
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.password)
        .execute(self.db.get_pool())
        .await
        .expect("Failed to insert user");
    }

    pub async fn delete(&self, id: &Uuid) {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(self.db.get_pool())
            .await
            .expect("Failed to delete user");
    }
}
