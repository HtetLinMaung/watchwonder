use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

use crate::{
    models::{
        category::{self, Category, CategoryRequest},
        product,
    },
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Serialize)]
pub struct GetCategoriesResponse {
    pub code: u16,
    pub message: String,
    pub data: Vec<Category>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub page_counts: usize,
}

#[derive(Deserialize)]
pub struct GetCategoriesQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[get("/api/categories")]
pub async fn get_categories(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetCategoriesQuery>,
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

    if !token.is_empty() {
        if verify_token_and_get_sub(&token).is_none() {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            });
        }
    }
    match category::get_categories(&query.search, query.page, query.per_page, &client).await {
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
            println!("Error retrieving categories: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all categories from database"),
            })
        }
    }
}

#[post("/api/categories")]
pub async fn add_category(
    req: HttpRequest,
    body: web::Json<CategoryRequest>,
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

    let user_id = parsed_values[0].parse().unwrap();
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

    match category::add_category(&body, user_id, &client).await {
        Ok(()) => HttpResponse::Created().json(BaseResponse {
            code: 201,
            message: String::from("Category added successfully"),
        }),
        Err(e) => {
            eprintln!("Category adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding category!"),
            });
        }
    }
}

#[get("/api/categories/{category_id}")]
pub async fn get_category_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let category_id = path.into_inner();
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

    match category::get_category_by_id(category_id, &client).await {
        Some(c) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Category fetched successfully."),
            data: Some(c),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Category not found!"),
        }),
    }
}

#[put("/api/categories/{category_id}")]
pub async fn update_category(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<CategoryRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let category_id = path.into_inner();
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

    match category::get_category_by_id(category_id, &client).await {
        Some(c) => {
            match category::update_category(category_id, &c.cover_image, &body, &client).await {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Category updated successfully"),
                }),
                Err(e) => {
                    eprintln!("Category updating error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error updating category!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Category not found!"),
        }),
    }
}

#[delete("/api/categories/{category_id}")]
pub async fn delete_category(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let category_id = path.into_inner();
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

    match product::is_products_exist("category_id", category_id, &client).await {
        Ok(is_exist) => {
            if is_exist {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Please delete the associated products first before deleting the category. Ensure all products related to this category are removed to proceed with category deletion!"),
                });
            };
        }
        Err(err) => {
            println!("{:?}", err);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 400,
                message: String::from("Something went wrong with checking products existence!"),
            });
        }
    }

    match category::get_category_by_id(category_id, &client).await {
        Some(c) => match category::delete_category(category_id, &c.cover_image, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 204,
                message: String::from("Category deleted successfully"),
            }),
            Err(e) => {
                eprintln!("Category deleting error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error deleting category!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Category not found!"),
        }),
    }
}
