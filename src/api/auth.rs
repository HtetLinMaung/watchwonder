use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::user::{create_user, get_user, user_exists};
use crate::utils::jwt;
use crate::utils::validator::{validate_email, validate_mobile};
use actix_web::{post, web, HttpResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub phone: String,
    pub profile_image: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub code: u16,
    pub message: String,
}

#[post("/api/auth/register")]
pub async fn register(
    client: web::Data<Arc<Client>>,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    if !validate_email(&body.email) {
        return HttpResponse::BadRequest().json(RegisterResponse {
            code: 400,
            message: String::from("Invalid email!"),
        });
    }
    if !validate_mobile(&body.phone) {
        return HttpResponse::BadRequest().json(RegisterResponse {
            code: 400,
            message: String::from("Invalid phone!"),
        });
    }
    match user_exists(&body.username, &client).await {
        Ok(exists) => {
            if exists {
                return HttpResponse::BadRequest().json(RegisterResponse {
                    code: 400,
                    message: String::from("User already exists!"),
                });
            }
            match create_user(
                &body.name,
                &body.username,
                &body.password,
                &body.email,
                &body.phone,
                &body.profile_image,
                &client,
            )
            .await
            {
                Ok(()) => HttpResponse::Ok().json(RegisterResponse {
                    code: 200,
                    message: String::from("Registration successfully"),
                }),
                Err(e) => {
                    eprintln!("User registration error: {}", e);
                    return HttpResponse::InternalServerError().json(RegisterResponse {
                        code: 500,
                        message: String::from("Error registering user!"),
                    });
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(RegisterResponse {
                code: 500,
                message: String::from("Something went wrong!"),
            });
        }
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub code: u16,
    pub message: String,
    pub token: Option<String>,
    pub name: Option<String>,
    pub profile_image: Option<String>,
}

#[post("/api/auth/login")]
pub async fn login(
    client: web::Data<Arc<Client>>,
    credentials: web::Json<LoginRequest>,
) -> HttpResponse {
    // Fetch user from the database based on the username
    let user = get_user(&credentials.username, &client).await;

    match user {
        Some(user) => {
            if verify(&credentials.password, &user.password).unwrap() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as usize;
                let token = jwt::sign_token(&jwt::Claims {
                    sub: format!("{},{}", &user.user_id, &user.role),
                    exp: now + (3600 * 24),
                })
                .unwrap();
                // let token = create_token(&user.username).unwrap();
                HttpResponse::Ok().json(LoginResponse {
                    code: 200,
                    message: String::from("Token generated successfully."),
                    token: Some(token),
                    name: Some(user.name),
                    profile_image: Some(user.profile_image),
                })
            } else {
                HttpResponse::Unauthorized().json(LoginResponse {
                    code: 401,
                    message: String::from("Invalid password!"),
                    token: None,
                    name: None,
                    profile_image: None,
                })
            }
        }
        None => HttpResponse::Unauthorized().json(LoginResponse {
            code: 401,
            message: String::from("Invalid username!"),
            token: None,
            name: None,
            profile_image: None,
        }),
    }
}

#[derive(Deserialize)]
pub struct PasswordInput {
    pub password: String,
}

#[derive(Serialize)]
pub struct HashedPasswordOutput {
    pub hashed_password: String,
}

#[post("/api/hash_password")]
pub async fn hash_password(password_input: web::Json<PasswordInput>) -> HttpResponse {
    match hash(&password_input.password, DEFAULT_COST) {
        Ok(hashed) => HttpResponse::Ok().json(HashedPasswordOutput {
            hashed_password: hashed,
        }),
        Err(_) => HttpResponse::InternalServerError().body("Failed to hash password"),
    }
}
