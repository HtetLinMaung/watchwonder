use std::{collections::HashMap, sync::Arc};

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;
use tokio_postgres::Client;

use crate::{
    models::{
        notification,
        seller_information::SellerInformationRequest,
        user::{self, is_phone_existed, UserProfile},
    },
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
        validator::{validate_email, validate_mobile},
    },
};

#[derive(Deserialize)]
pub struct GetUsersQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub role: Option<String>,
    pub account_status: Option<String>,
    pub request_to_agent: Option<bool>,
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

    match user::get_users(
        &query.search,
        query.page,
        query.per_page,
        &query.role,
        &query.account_status,
        query.request_to_agent,
        &client,
    )
    .await
    {
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
    pub account_status: Option<String>,
    pub seller_information: Option<SellerInformationRequest>,
    pub can_modify_order_status: Option<bool>,
    pub can_view_address: Option<bool>,
    pub can_view_phone: Option<bool>,
}

#[post("/api/users")]
pub async fn add_user(
    req: HttpRequest,
    body: web::Json<AddUserRequest>,
    client: web::Data<Arc<Client>>,
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
    if &body.role != "admin" && &body.role != "user" && &body.role != "agent" {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Role must be admin, user or agent!"),
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
    match user::user_exists(&body.username, &client).await {
        Ok(exists) => {
            if exists {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("User already exists!"),
                });
            }

            let mut account_status = "active";
            if let Some(status) = &body.account_status {
                account_status = status;
            }

            let mut can_modify_order_status = false;
            if let Some(yes) = body.can_modify_order_status {
                can_modify_order_status = yes;
            }

            let mut can_view_address: bool = false;
            if let Some(yes) = body.can_view_address {
                can_view_address = yes;
            }

            let mut can_view_phone: bool = false;
            if let Some(yes) = body.can_view_phone {
                can_view_phone = yes;
            }

            match user::add_user(
                &body.name,
                &body.username,
                &body.password,
                &body.email,
                &body.phone,
                &body.profile_image,
                &body.role,
                account_status,
                can_modify_order_status,
                can_view_address,
                can_view_phone,
                &body.seller_information,
                &None,
                false,
                &client,
            )
            .await
            {
                Ok(_) => HttpResponse::Created().json(BaseResponse {
                    code: 201,
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

#[get("/api/users/{user_id}")]
pub async fn get_user_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let mut user_id = path.into_inner();
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

    // let role: &str = parsed_values[1];
    if user_id == 0 {
        user_id = parsed_values[0].parse().unwrap();
    }

    // if role != "admin" {
    //     return HttpResponse::Unauthorized().json(BaseResponse {
    //         code: 401,
    //         message: String::from("Unauthorized!"),
    //     });
    // }

    match user::get_user_by_id(user_id, &client).await {
        Some(u) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("User fetched successfully."),
            data: Some(u),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("User not found!"),
        }),
    }
}

#[derive(Deserialize, Debug)]
pub struct UpdateUserRequest {
    pub name: String,
    pub password: String,
    pub email: String,
    pub phone: String,
    pub role: String,
    pub profile_image: String,
    pub account_status: Option<String>,
    pub seller_information: Option<SellerInformationRequest>,
    pub can_modify_order_status: Option<bool>,
    pub can_view_address: Option<bool>,
    pub can_view_phone: Option<bool>,
    pub request_to_agent: Option<bool>,
}

#[put("/api/users/{user_id}")]
pub async fn update_user(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<UpdateUserRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    println!("update_user body: {:?}", body);
    let mut user_id = path.into_inner();
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
    let request_to_agent = if let Some(rta) = body.request_to_agent {
        rta
    } else {
        false
    };

    if request_to_agent {
        user_id = parsed_values[0].parse().unwrap();
    }

    if (!request_to_agent && role != "admin") || (request_to_agent && &body.role != "user") {
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
    if &body.role != "admin" && &body.role != "user" && &body.role != "agent" {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Role must be admin, user or agent!"),
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
    match user::get_user_by_id(user_id, &client).await {
        Some(u) => {
            if &u.phone != &body.phone && is_phone_existed(&body.phone, &client).await {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Phone number is already in use!"),
                });
            }
            let old_password: &str = &u.password;
            let old_profile_image: &str = &u.profile_image;

            let mut account_status = "active";
            if let Some(status) = &body.account_status {
                account_status = status;
            }

            let mut can_modify_order_status = false;
            if let Some(yes) = body.can_modify_order_status {
                can_modify_order_status = yes;
            }

            let mut can_view_address: bool = false;
            if let Some(yes) = body.can_view_address {
                can_view_address = yes;
            }

            let mut can_view_phone: bool = false;
            if let Some(yes) = body.can_view_phone {
                can_view_phone = yes;
            }

            match user::update_user(
                user_id,
                &body.name,
                old_password,
                &body.password,
                &body.email,
                &body.phone,
                old_profile_image,
                &body.profile_image,
                &body.role,
                account_status,
                can_modify_order_status,
                can_view_address,
                can_view_phone,
                &body.seller_information,
                request_to_agent,
                &client,
            )
            .await
            {
                Ok(()) => {
                    if request_to_agent {
                        let title = format!("Agent Role Upgrade Request");
                        let message = format!(
                            "User {} has requested to upgrade to an 'Agent' role.",
                            &u.username
                        );
                        let mut map = HashMap::new();
                        map.insert(
                            "redirect".to_string(),
                            Value::String("agent-approval".to_string()),
                        );
                        map.insert("id".to_string(), Value::Number(user_id.into()));
                        tokio::spawn(async move {
                            match notification::add_notification_to_admins(
                                &title,
                                &message,
                                &Some(map),
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
                        });
                    }
                    HttpResponse::Ok().json(BaseResponse {
                        code: 200,
                        message: String::from("User updated successfully"),
                    })
                }
                Err(e) => {
                    eprintln!("User updating error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error updating user!"),
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

#[delete("/api/users/{user_id}")]
pub async fn delete_user(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let user_id = path.into_inner();
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

    match user::get_user_by_id(user_id, &client).await {
        Some(u) => match user::delete_user(user_id, &u.profile_image, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 204,
                message: String::from("User deleted successfully"),
            }),
            Err(e) => {
                eprintln!("User deleting error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error deleting user!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("User not found!"),
        }),
    }
}

#[get("/api/profile")]
pub async fn get_user_profile(req: HttpRequest, client: web::Data<Arc<Client>>) -> impl Responder {
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

    match user::get_user_profile(user_id, &client).await {
        Some(p) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Profile fetched successfully."),
            data: Some(p),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Profile not found!"),
        }),
    }
}

#[put("/api/profile")]
pub async fn update_user_profile(
    req: HttpRequest,
    body: web::Json<UserProfile>,
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
    match user::get_user_by_id(user_id, &client).await {
        Some(u) => {
            let old_profile_image: &str = &u.profile_image;
            match user::update_user_profile(user_id, &body, old_profile_image, &client).await {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Profile updated successfully"),
                }),
                Err(e) => {
                    eprintln!("Profile updating error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error updating profile!"),
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

#[delete("/api/delete-account")]
pub async fn delete_account(req: HttpRequest, client: web::Data<Arc<Client>>) -> HttpResponse {
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

    let user_id: i32 = parsed_values[0].parse().unwrap();
    // let role: &str = parsed_values[1];

    match user::get_user_by_id(user_id, &client).await {
        Some(u) => match user::delete_user(user_id, &u.profile_image, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 204,
                message: String::from("User deleted successfully"),
            }),
            Err(e) => {
                eprintln!("User deleting error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error deleting user!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("User not found!"),
        }),
    }
}
