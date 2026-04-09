use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}