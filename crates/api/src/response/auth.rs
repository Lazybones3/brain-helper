use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub permissions: Vec<String>,
    pub token: TokenInfo,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub expiry: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
}
