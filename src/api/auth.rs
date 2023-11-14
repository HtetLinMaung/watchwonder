use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::notification;
use crate::models::user::{self, get_user, is_phone_existed, user_exists};
use crate::utils::common_struct::{BaseResponse, DataResponse};
use crate::utils::jwt::{self, verify_token_and_get_sub};
use crate::utils::validator::{validate_email, validate_mobile};
use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
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
    pub role: Option<String>,
}

#[post("/api/auth/register")]
pub async fn register(
    client: web::Data<Arc<Client>>,
    body: web::Json<RegisterRequest>,
) -> HttpResponse {
    if body.name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Name must not be empty!"),
        });
    }

    if !validate_email(&body.email) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Invalid email!"),
        });
    }
    if !validate_mobile(&body.phone) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Invalid phone!"),
        });
    }
    if is_phone_existed(&body.phone, &client).await {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Phone number is already in use!"),
        });
    }

    let mut account_status = "active";
    let mut role = "user";
    if let Some(r) = &body.role {
        role = r.as_str();
        if role != "user" && role != "agent" {
            return HttpResponse::BadRequest().json(BaseResponse {
                code: 400,
                message: String::from("Role must be user or agent!"),
            });
        }
        account_status = match role {
            "agent" => "pending",
            _ => "active",
        };
    }

    match user_exists(&body.username, &client).await {
        Ok(exists) => {
            if exists {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("User already exists!"),
                });
            }

            match user::add_user(
                &body.name,
                &body.username,
                &body.password,
                &body.email,
                &body.phone,
                &body.profile_image,
                role,
                account_status,
                false,
                &None,
                &client,
            )
            .await
            {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Registration successfully"),
                }),
                Err(e) => {
                    eprintln!("User registration error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error registering user!"),
                    });
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
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
pub struct LoginData {
    pub token: String,
    pub name: String,
    pub profile_image: String,
    pub role: String,
    pub can_modify_order_status: bool,
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
            if &user.account_status != "active" {
                return HttpResponse::Unauthorized().json(BaseResponse {
                    code: 401,
                    message: String::from("Your account has not been activated yet. Please wait for an admin to approve your account or contact support for further assistance!")
                });
            }

            if verify(&credentials.password, &user.password).unwrap() {
                // let now = SystemTime::now()
                //     .duration_since(UNIX_EPOCH)
                //     .expect("Time went backwards")
                //     .as_secs() as usize;
                // let token = jwt::sign_token(&jwt::Claims {
                //     sub: format!("{},{}", &user.user_id, &user.role),
                //     exp: now + (3600 * 24),
                // })
                // .unwrap();

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as usize;
                // Setting a far future expiration time
                let far_future_exp = now + (3600 * 24 * 365 * 100); // 100 years into the future
                let token = jwt::sign_token(&jwt::Claims {
                    sub: format!("{},{},{}", &user.id, &user.role_name, &user.shop_id),
                    exp: far_future_exp,
                })
                .unwrap();
                // let token = create_token(&user.username).unwrap();
                HttpResponse::Ok().json(DataResponse {
                    code: 200,
                    message: String::from("Token generated successfully."),
                    data: Some(LoginData {
                        token,
                        name: user.name,
                        profile_image: user.profile_image,
                        role: user.role,
                        can_modify_order_status: user.can_modify_order_status,
                    }),
                })
            } else {
                HttpResponse::Unauthorized().json(BaseResponse {
                    code: 401,
                    message: String::from("Invalid password!"),
                })
            }
        }
        None => HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Invalid username!"),
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

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub new_password: String,
    pub old_password: String,
}

#[post("/api/auth/change-password")]
pub async fn change_password(
    req: HttpRequest,
    body: web::Json<ChangePasswordRequest>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Invalid Authorization header format"),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Authorization header missing"),
            })
        }
    };

    let sub = match verify_token_and_get_sub(token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 2 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Invalid sub format in token"),
        });
    }

    let user_id: &str = parsed_values[0];
    let user_id: i32 = user_id.parse().unwrap();

    if &body.old_password == &body.new_password {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Old password and new password are the same!"),
        });
    }

    match user::get_user_by_id(user_id, &client).await {
        Some(u) => {
            if !verify(&body.old_password, &u.password).unwrap() {
                return HttpResponse::Unauthorized().json(BaseResponse {
                    code: 401,
                    message: String::from("Incorrect password!"),
                });
            }

            match user::change_password(user_id, &body.new_password, &client).await {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Password changed successfully."),
                }),
                Err(e) => {
                    eprintln!("Password changing error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error changing password!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("User not found!"),
        }),
    }
}

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct VerifyTokenData {
    pub room: i32,
}

#[post("/api/auth/verify-token")]
pub async fn verify_token(body: web::Json<VerifyTokenRequest>) -> impl Responder {
    let sub = match verify_token_and_get_sub(&body.token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 2 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Invalid sub format in token"),
        });
    }

    let user_id: i32 = parsed_values[0].parse().unwrap();
    return HttpResponse::Ok().json(DataResponse {
        code: 200,
        message: String::from("Token is valid."),
        data: Some(VerifyTokenData { room: user_id }),
    });
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub username: Option<String>,
    pub email: String,
    pub phone: String,
}

#[post("/api/auth/forgot-password")]
pub async fn forgot_password(
    body: web::Json<ForgotPasswordRequest>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
    let user = get_user(&body.email, &client).await;

    match user {
        Some(user) => {
            if &user.account_status != "active" {
                return HttpResponse::Unauthorized().json(BaseResponse {
                    code: 401,
                    message: String::from("Your account has not been activated yet. Please wait for an admin to approve your account or contact support for further assistance!")
                });
            }

            if &user.role == "admin" || &user.email != &body.email || &user.phone != &user.phone {
                return HttpResponse::Unauthorized().json(BaseResponse {
                    code: 401,
                    message: String::from("Unauthorized!"),
                });
            }
            let message = format!("A password reset request was made for user {} ({}). Please verify legitimacy and monitor for any suspicious activity.",&user.name, &body.email);
            match notification::add_notification_to_admins(
                "Password Reset Alert",
                &message,
                &client,
            )
            .await
            {
                Ok(()) => {
                    println!("Notification added successfully.");
                }
                Err(err) => {
                    println!("Error adding notification: {:?}", err);
                }
            };
            HttpResponse::Ok().json(BaseResponse {
                code: 200,
                message: String::from(
                    "Your request to reset your password has been successfully submitted.",
                ),
            })
        }
        None => HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Invalid username!"),
        }),
    }
}
