use crate::{logger::log, models::User};
use chrono::Utc;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    exp: i64,
}

fn get_jwt_secret() -> String {
    match std::env::var("JWT_SECRET") {
        Ok(secret) => secret,
        Err(_) => {
            log("JWT_SECRET not set, remember to create a `.env` file and run `diesel migration run`", crate::logger::LogLevel::Error);
            std::process::exit(1);
        }
    }
}

pub fn generate_auth_jwt(user: &User) -> Result<String, Box<dyn std::error::Error>> {
    let secret = get_jwt_secret();

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
    let secret = get_jwt_secret();

    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(decoded.claims)
}
