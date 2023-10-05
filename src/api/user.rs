use std::sync::Arc;

use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::user::{self, user_exists},
    utils::{
        common_struct::{BaseResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
        validator::{validate_email, validate_mobile},
    },
};

#[derive(Deserialize)]
pub struct GetUsersQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[get("/api/users")]
pub async fn get_users(
    req: HttpRequest,
    query: web::Query<GetUsersQuery>,
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

    let role: &str = parsed_values[1];

    if role != "admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match user::get_users(&query.search, query.page, query.per_page, &client).await {
        Ok(item_result) => HttpResponse::Ok().json(PaginationResponse {
            code: 200,
            message: String::from("Successful."),
            data: item_result.data,
            total: item_result.total,
            page: item_result.page,
            per_page: item_result.per_page,
            page_counts: item_result.page_counts,
        }),
        Err(err) => {
            // Log the error message here
            println!("Error retrieving users: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all users from database"),
            })
        }
    }
}

#[derive(Deserialize)]
pub struct AddUserRequest {
    pub name: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub phone: String,
    pub role: String,
    pub profile_image: String,
}

#[post("/api/users")]
pub async fn add_user(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    body: web::Json<AddUserRequest>,
) -> HttpResponse {
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

    let role: &str = parsed_values[1];

    if role != "admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Name must not be empty!"),
        });
    }
    if &body.role != "admin" && &body.role != "user" {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Role must be admin or user!"),
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
                &body.role,
                &client,
            )
            .await
            {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("User added successfully"),
                }),
                Err(e) => {
                    eprintln!("User adding error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error adding user!"),
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
