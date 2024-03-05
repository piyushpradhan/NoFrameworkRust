use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
}

impl User {
    pub fn new(id: i32, username: String, password: String) -> Self {
        User {
            id,
            username,
            password,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn set_username(&mut self, username: String) {
        self.username = username;
    }

    pub fn set_password(&mut self, password: String) {
        self.password = password;
    }
}
