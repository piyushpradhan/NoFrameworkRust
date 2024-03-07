use sqlx::{postgres::PgPool, Error, Row};
extern crate bcrypt;

use bcrypt::{hash, verify, BcryptError, DEFAULT_COST};

use crate::app::models::{claims::Claims, user::User};

use super::utils::{generate_token, verify_token};

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
        // Extract the token from cookies
        let token = cookies.and_then(|cookies| {
            cookies
                .iter()
                .find(|(name, _)| *name == "token")
                .map(|(_, value)| value.to_string())
        });

        let decoded_token = match token {
            Some(token) => {
                println!("Extracted token: {}", token);
                verify_token(&token)
            }
            None => return Err(Error::RowNotFound),
        };

        let claims: Claims = match decoded_token {
            Ok(token_data) => {
                println!("Token data: {:?}", token_data);
                token_data.claims.into()
            }
            Err(err) => {
                print!("Something went wrong while getting token_data: {}", err);
                return Err(Error::RowNotFound);
            }
        };
        let query = "SELECT * FROM users WHERE id = $1 AND username = $2";

        // Check if a user with provided credentials exists
        let result = sqlx::query(query)
            .bind(claims.uid)
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(result) => {
                let id: i32 = result.try_get("id")?;
                let username: String = result.try_get("username")?;
                let hashed_password: String = result.try_get("password")?;

                let is_password_correct = verify(password, &hashed_password);

                match is_password_correct {
                    Ok(_) => {
                        let custom_token = generate_token(id, username.as_str());
                        return Ok(User::new(
                            id,
                            username.to_string(),
                            custom_token.to_string(),
                        ));
                    }
                    Err(_) => return Err(Error::RowNotFound),
                }
            }
            None => {
                println!("User not found");
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
                let custom_token = generate_token(id, username);
                return Ok(User::new(id, username.to_string(), custom_token));
            }
            None => {
                return Err(Error::RowNotFound);
            }
        }
    }
}
