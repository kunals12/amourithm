use actix_web::{
    cookie::time::{Duration, OffsetDateTime},
    HttpRequest, HttpResponse,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

use crate::common::ResponseToSend;

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    sub: uuid::Uuid,
    exp: usize,
    iat: usize,
    iss: String,
}

pub fn generate_token(id: uuid::Uuid) -> String {
    // Retrieve the secret key from environment variables
    let secret_key =
        env::var("COOKIES_SECRET_KEY").expect("COOKIES_SECRET_KEY must be set in the .env file");
    // Calculate expiration time (current time + 2 hours)
    let expiration = OffsetDateTime::now_utc() + Duration::hours(24);
    let issuer = String::from("Amourithm");
    let claims = Claims {
        sub: id,
        exp: expiration.unix_timestamp() as usize,
        iat: OffsetDateTime::now_utc().unix_timestamp() as usize,
        iss: issuer,
    };

    // Encode the Claims into a JWT token
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret_key.as_bytes()),
    )
    .expect("Error generating token")
}

// Middleware-like function to validate token and extract user data
pub async fn validate_token(req: HttpRequest) -> Result<uuid::Uuid, HttpResponse> {
    if let Some(cookie) = req.cookie("auth_token") {
        let token = cookie.value();
        let secret_key = env::var("COOKIES_SECRET_KEY")
            .expect("COOKIES_SECRET_KEY must be set in the .env file");
        let verified_token = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret_key.as_bytes()), // Decode using the same secret key
            &Validation::default(), // Use default validation (e.g., check expiry)
        );

        match verified_token {
            Ok(data) => {
                let user_id = data.claims.sub;
                Ok(user_id) // Return user ID and username
            }
            Err(e) => {
                // Handle invalid token
                Err(HttpResponse::Unauthorized().json(ResponseToSend::<()> {
                    success: false,
                    message: format!("Invalid token: {}", e), // Serialize the error
                    data: None,
                }))
            }
        }
    } else {
        // Handle missing token
        Err(HttpResponse::Unauthorized().json(ResponseToSend::<()> {
            success: false,
            message: "Missing token".to_string(),
            data: None,
        }))
    }
}
