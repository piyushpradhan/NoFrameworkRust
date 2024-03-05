use sqlx::{postgres::PgPool, Error, Row};

use crate::app::models::user::User;

pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        AuthService { pool }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<User, Error> {
        let query = "SELECT * FROM users WHERE username = $1 AND password = $2";
        let result = sqlx::query(query)
            .bind(username)
            .bind(password)
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(result) => {
                let id: i32 = result.try_get("id")?;
                let username: String = result.try_get("username")?;
                let password: String = result.try_get("password")?;
                println!("id: {}, username: {}, password: {}", id, username, password);
                return Ok(User::new(id, username.to_string(), password.to_string()));
            }
            None => {
                println!("Row not found");
                return Err(Error::RowNotFound);
            }
        }
    }
}
