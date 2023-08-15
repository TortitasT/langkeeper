use crate::models::User;
use chrono::Utc;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    exp: i64,
}

pub fn generate_auth_jwt(user: &User) -> Result<String, Box<dyn std::error::Error>> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let claims = Claims {
        sub: user.id,
        exp: Utc::now().timestamp() + 60 * 60 * 24 * 7, // 7 days
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn decode_auth_jwt(token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(decoded.claims)
}
