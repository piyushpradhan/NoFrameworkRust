use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub username: String,
    pub uid: i32,
    pub exp: usize,
}

impl Claims {
    pub fn new(username: String, uid: i32, exp: usize) -> Self {
        Claims { username, uid, exp }
    }
}
