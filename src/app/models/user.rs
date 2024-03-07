use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub access_token: String,
}

impl User {
    pub fn new(id: i32, username: String, access_token: String) -> Self {
        User {
            id,
            username,
            access_token,
        }
    }
}
