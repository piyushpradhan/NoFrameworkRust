use dotenv::dotenv;
use jsonwebtoken::{
    decode, encode,
    errors::{Error, ErrorKind},
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use sqlx::PgPool;
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
    usize,
};

use crate::{app::models::claims::Claims, http::utils::refresh_access_token};

pub fn generate_token(uid: i32, username: &str) -> String {
    dotenv().ok();
    let secret = match env::var("JWT_SECRET") {
        Ok(jwt_secret) => jwt_secret,
        _ => panic!("JWT_SECRET is not provided"),
    };
    let expiration_secs: usize = match env::var("EXP") {
        Ok(exp) => exp.parse().unwrap(),
        _ => panic!("EXP is not provided"),
    };

    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + expiration_secs;

    let claims = Claims::new(username.to_string(), uid, exp);

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::new(Algorithm::HS256), &claims, &encoding_key).unwrap()
}

pub fn generate_refresh_token(username: &str, uid: i32) -> String {
    dotenv().ok();

    let secret = match env::var("REFRESH_TOKEN_SECRET") {
        Ok(refresh_secret) => refresh_secret,
        _ => panic!("REFRESH_TOKEN_SECRET is invalid"),
    };

    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 604800;

    let claims = Claims::new(username.to_string(), uid, exp);
    let encoding_key = EncodingKey::from_secret(&secret.as_ref());
    encode(&Header::new(Algorithm::HS256), &claims, &encoding_key).unwrap()
}

pub fn verify_token(
    token: &str,
    cookies: Option<Vec<(&str, &str)>>,
) -> Result<TokenData<Claims>, Error> {
    dotenv().ok();
    let secret = match env::var("JWT_SECRET") {
        Ok(jwt_secret) => jwt_secret,
        _ => panic!("JWT_SECRET is not provided"),
    };

    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    match decode::<Claims>(token, &decoding_key, &validation) {
        Ok(token_data) => Ok(token_data),
        Err(error) => {
            return handle_expired_token(error, cookies, decoding_key, validation);
        }
    }
}

fn handle_expired_token(
    error: Error,
    cookies: Option<Vec<(&str, &str)>>,
    decoding_key: DecodingKey<'_>,
    validation: Validation,
) -> Result<TokenData<Claims>, Error> {
    match error.kind() {
        ErrorKind::ExpiredSignature => {
            let access_token = refresh_access_token(cookies);
            match decode::<Claims>(
                access_token.unwrap_or_else(|_| String::new()).as_str(),
                &decoding_key,
                &validation,
            ) {
                Ok(token_data) => Ok(token_data),
                Err(error) => {
                    eprintln!("Invalid refresh token");
                    return Err(error);
                }
            }
        }
        _ => {
            eprintln!("Error while verifying token");
            return Err(error);
        }
    }
}

pub async fn store_refresh_token(refresh_token: &str, id: i32, pool: &PgPool) {
    // Store the refresh token in database
    let store_refresh_token_query = "UPDATE users SET refresh_token = $1 WHERE id = $2;";
    sqlx::query(store_refresh_token_query)
        .bind(&refresh_token)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}
