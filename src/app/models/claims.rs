use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

impl Claims {
    fn new(sub: String, exp: usize) -> Self {
        Claims { sub, exp }
    }
}

pub fn generate_token(secret: &str, subject: &str, expiration_secs: usize) -> String {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + expiration_secs;

    let claims = Claims::new(subject.to_string(), exp);

    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    encode(&Header::new(Algorithm::HS256), &claims, &encoding_key).unwrap()
}

pub fn verify_token(secret: &str, token: &str) -> Result<(), jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    decode::<Claims>(token, &decoding_key, &validation).map(|_| ())
}
