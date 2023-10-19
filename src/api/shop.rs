use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::shop::{self, ShopRequest},
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct GetShopsQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[get("/api/shops")]
pub async fn get_shops(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetShopsQuery>,
) -> impl Responder {
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1].to_string()
            } else {
                "".to_string()
            }
        }
        None => "".to_string(),
    };

    let mut role = "user".to_string();
    let mut user_id = 0;
    if !token.is_empty() {
        let sub = match verify_token_and_get_sub(&token) {
            Some(s) => s,
            None => {
                return HttpResponse::Unauthorized().json(BaseResponse {
                    code: 401,
                    message: String::from("Invalid token"),
                })
            }
        };
        // Parse the `sub` value
        let parsed_values: Vec<String> = sub.split(',').map(|s| s.to_string()).collect();
        if parsed_values.len() != 2 {
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Invalid sub format in token"),
            });
        }
        user_id = parsed_values[0].parse().unwrap();
        role = parsed_values[1].clone();
    }

    match shop::get_shops(
        &query.search,
        query.page,
        query.per_page,
        &role,
        user_id,
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
            println!("Error retrieving shops: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all shops from database"),
            })
        }
    }
}

#[post("/api/shops")]
pub async fn add_shop(
    req: HttpRequest,
    body: web::Json<ShopRequest>,
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

    let user_id: i32 = parsed_values[0].parse().unwrap();
    let role: &str = parsed_values[1];

    if role != "admin" && role != "agent" {
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
    if body.cover_image.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Cover image must not be empty!"),
        });
    }

    let status_list: Vec<&str> = vec![
        "Active",
        "Inactive",
        "Closed",
        "Suspended",
        "Pending Approval",
    ];
    if !status_list.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from(
                "Please select a valid status: Active, Inactive, Closed, Suspended, or Pending Approval.",
            ),
        });
    }

    match shop::add_shop(&body, user_id, &client).await {
        Ok(()) => HttpResponse::Created().json(BaseResponse {
            code: 201,
            message: String::from("Shop added successfully"),
        }),
        Err(e) => {
            eprintln!("Shop adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding shop!"),
            });
        }
    }
}

#[get("/api/shops/{shop_id}")]
pub async fn get_shop_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let shop_id = path.into_inner();
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

    if role != "admin" && role != "agent" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match shop::get_shop_by_id(shop_id, &client).await {
        Some(s) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Shop fetched successfully."),
            data: Some(s),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Shop not found!"),
        }),
    }
}

#[put("/api/shops/{shop_id}")]
pub async fn update_shop(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<ShopRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let shop_id = path.into_inner();
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

    if role != "admin" && role != "agent" {
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
    if body.cover_image.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Cover image must not be empty!"),
        });
    }

    let status_list: Vec<&str> = vec![
        "Active",
        "Inactive",
        "Closed",
        "Suspended",
        "Pending Approval",
    ];
    if !status_list.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from(
                "Please select a valid status: Active, Inactive, Closed, Suspended, or Pending Approval.",
            ),
        });
    }

    match shop::get_shop_by_id(shop_id, &client).await {
        Some(s) => match shop::update_shop(shop_id, &s.cover_image, &body, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 200,
                message: String::from("Shop updated successfully"),
            }),
            Err(e) => {
                eprintln!("Shop updating error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error updating shop!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Shop not found!"),
        }),
    }
}

#[delete("/api/shops/{shop_id}")]
pub async fn delete_shop(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let shop_id = path.into_inner();
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

    if role != "admin" && role != "agent" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match shop::get_shop_by_id(shop_id, &client).await {
        Some(s) => match shop::delete_shop(shop_id, &s.cover_image, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 204,
                message: String::from("Shop deleted successfully"),
            }),
            Err(e) => {
                eprintln!("Shop deleting error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error deleting shop!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Shop not found!"),
        }),
    }
}
