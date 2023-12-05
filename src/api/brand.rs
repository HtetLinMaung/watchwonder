use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::{
        brand::{self},
        product,
    },
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct BrandRequest {
    pub name: String,
    pub description: String,
    pub logo_url: String,
}

#[derive(Deserialize)]
pub struct GetBrandsQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub shop_id: Option<i32>,
    pub platform: Option<String>,
    pub version: Option<String>,
}

#[get("/api/brands")]
pub async fn get_brands(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetBrandsQuery>,
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

    // let mut role = "user".to_string();
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
        // role = parsed_values[1].clone();
    }
    let platform = match &query.platform {
        Some(p) => p.as_str(),
        None => "",
    };
    let version = match &query.version {
        Some(v) => v.replace(".", "").parse().unwrap(),
        None => 0,
    };
    match brand::get_brands(
        &query.search,
        query.page,
        query.per_page,
        query.shop_id,
        platform,
        user_id,
        version,
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
            println!("Error retrieving brands: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all brands from database"),
            })
        }
    }
}

#[post("/api/brands")]
pub async fn add_brand(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    body: web::Json<BrandRequest>,
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
    if role != "admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Brand name cannot be empty!"),
        });
    }
    match brand::add_brand(
        &body.name,
        &body.description,
        &body.logo_url,
        user_id,
        &client,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(BaseResponse {
            code: 200,
            message: String::from("Brand added successfully."),
        }),
        Err(e) => {
            println!("Error adding brands: {:?}", e);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to add brands to database"),
            })
        }
    }
}

#[get("/api/brands/{brand_id}")]
pub async fn get_brand_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let brand_id = path.into_inner();
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

    match brand::get_brand_by_id(brand_id, &client).await {
        Some(b) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Brand fetched successfully."),
            data: Some(b),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Brand not found!"),
        }),
    }
}

#[put("/api/brands/{brand_id}")]
pub async fn update_brand(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    path: web::Path<i32>,
    body: web::Json<BrandRequest>,
) -> HttpResponse {
    let brand_id = path.into_inner();
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
            message: String::from("Brand name cannot be empty!"),
        });
    }
    match brand::get_brand_by_id(brand_id, &client).await {
        Some(b) => {
            match brand::update_brand(
                brand_id,
                &body.name,
                &body.description,
                &body.logo_url,
                &b.logo_url,
                &client,
            )
            .await
            {
                Ok(_) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Brand Updated successfully."),
                }),
                Err(e) => {
                    println!("Error updating brand: {:?}", e);
                    HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error trying to updating Brand to database"),
                    })
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Brand not found!"),
        }),
    }
}

#[delete("/api/brands/{brand_id}")]
pub async fn delete_brand(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let brand_id = path.into_inner();
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

    match product::is_products_exist("brand_id", brand_id, &client).await {
        Ok(is_exist) => {
            if is_exist {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Please delete the associated products first before deleting the brand. Ensure all products related to this brand are removed to proceed with brand deletion!"),
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

    match brand::get_brand_by_id(brand_id, &client).await {
        Some(b) => match brand::delete_brand(brand_id, &b.logo_url, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 204,
                message: String::from("Brand deleted successfully"),
            }),
            Err(e) => {
                eprintln!("Brand deleting error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error deleting Brand!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Brand not found!"),
        }),
    }
}
