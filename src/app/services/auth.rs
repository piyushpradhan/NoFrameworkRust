use sqlx::{postgres::PgPool, Error, Row};
extern crate bcrypt;

use bcrypt::{hash, verify, DEFAULT_COST};

use crate::{
    app::models::{claims::Claims, user::User},
    http::utils::extract_token_from_cookies,
};

use super::utils::{generate_refresh_token, generate_token, store_refresh_token, verify_token};

#[derive(Debug)]
pub struct AuthService {
    pool: PgPool,
}

impl AuthService {
    pub fn new(pool: PgPool) -> Self {
        AuthService { pool }
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        cookies: Option<Vec<(&str, &str)>>,
    ) -> Result<User, Error> {
        let query = "SELECT * FROM users WHERE username = $1";

        // Check if a user with provided credentials exists
        let result = sqlx::query(query)
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(result) => {
                let id: i32 = result.try_get("id")?;
                let username: String = result.try_get("username")?;
                let hashed_password: String = result.try_get("password")?;
                let refresh_token: String = result.try_get("refresh_token")?;

                let is_password_correct = verify(password, &hashed_password);

                match is_password_correct {
                    Ok(_) => {
                        let access_token = generate_token(id, username.as_str());

                        return Ok(User::new(
                            id,
                            username.to_string(),
                            access_token.to_string(),
                            refresh_token.to_string(),
                        ));
                    }
                    Err(_) => return Err(Error::RowNotFound),
                }
            }
            None => {
                eprintln!("User not found");
                return Err(Error::RowNotFound);
            }
        }
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<User, Error> {
        let hashed_password = hash(&password, DEFAULT_COST).unwrap();
        let query = "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id, username";
        let result = sqlx::query(query)
            .bind(username)
            .bind(hashed_password)
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(result) => {
                let id: i32 = result.get("id");
                let access_token = generate_token(id, username);
                let refresh_token = generate_refresh_token(&username, id);

                // Store the refresh token in database
                store_refresh_token(&refresh_token, id, &self.pool).await;

                return Ok(User::new(
                    id,
                    username.to_string(),
                    access_token,
                    refresh_token,
                ));
            }
            None => {
                return Err(Error::RowNotFound);
            }
        }
    }
}
