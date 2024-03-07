use dotenv::dotenv;
use jsonwebtoken::{
    decode, encode, errors::Error, Algorithm, DecodingKey, EncodingKey, Header, TokenData,
    Validation,
};
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
    usize,
};

use crate::app::models::claims::Claims;

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

pub fn verify_token(token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
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
            println!("Error while verifying token: {:?}", error);
            return Err(error);
        }
    }
}
