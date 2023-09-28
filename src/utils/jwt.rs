use std::env;

use jsonwebtoken::{
    decode, encode, errors::ErrorKind, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub fn sign_token(claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

// pub fn verify_token(token: &str) -> bool {
//     let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

//     let validation = Validation::default();
//     match decode::<Claims>(
//         &token,
//         &DecodingKey::from_secret(secret.as_ref()),
//         &validation,
//     ) {
//         Ok(_) => true,   // Token is valid
//         Err(_) => false, // Token is invalid
//     }
// }

pub fn verify_token_and_get_sub(token: &str) -> Option<String> {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let validation = Validation::new(Algorithm::HS256); // Assuming you're using HS256
    match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    ) {
        Ok(data) => Some(data.claims.sub),
        Err(err) => {
            match *err.kind() {
                ErrorKind::InvalidToken => println!("Token is invalid"), // Example logging for invalid token
                ErrorKind::ExpiredSignature => println!("Token is expired"), // Example logging for expired token
                _ => println!("Some other error: {:?}", err),                // Other errors
            }
            None
        }
    }
}
