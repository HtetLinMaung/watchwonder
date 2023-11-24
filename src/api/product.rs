use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::{
        category, currency,
        product::{self, ProductRequest},
        shop,
    },
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct GetProductsRequestBody {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub platform: Option<String>,
    pub shop_id: Option<i32>,
    pub category_id: Option<i32>,
    pub brands: Option<Vec<i32>>,
    pub models: Option<Vec<String>>,
    pub from_price: Option<f64>,
    pub to_price: Option<f64>,
    pub is_top_model: Option<bool>,
    pub products: Option<Vec<i32>>,
    pub view: Option<String>,
}

#[post("/api/get-products")]
pub async fn get_products(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    body: web::Json<GetProductsRequestBody>,
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

    let platform = match &body.platform {
        Some(p) => p.as_str(),
        None => "",
    };
    match product::get_products(
        &body.search,
        body.page,
        body.per_page,
        platform,
        body.shop_id,
        body.category_id,
        &body.brands,
        &body.models,
        body.from_price,
        body.to_price,
        body.is_top_model,
        &body.products,
        &body.view,
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
            println!("Error retrieving products: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all products from database"),
            })
        }
    }
}

#[derive(Deserialize)]
pub struct GetModelsQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[get("/api/models")]
pub async fn get_models(
    query: web::Query<GetModelsQuery>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
    match product::get_models(&query.search, query.page, query.per_page, &client).await {
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
            println!("Error retrieving models: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all models from database"),
            })
        }
    }
}

#[post("/api/products")]
pub async fn add_product(
    req: HttpRequest,
    body: web::Json<ProductRequest>,
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

    if body.model.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Model must not be empty!"),
        });
    }
    if body.description.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Description must not be empty!"),
        });
    }
    if body.color.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Color must not be empty!"),
        });
    }
    if body.strap_material.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Strap Material must not be empty!"),
        });
    }
    if body.strap_color.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Strap Color must not be empty!"),
        });
    }
    if body.case_material.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Case Material must not be empty!"),
        });
    }
    if body.dial_color.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Dial Color must not be empty!"),
        });
    }
    if body.movement_type.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Movement Type must not be empty!"),
        });
    }
    if body.water_resistance.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Water Resistance must not be empty!"),
        });
    }
    if body.warranty_period.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Warranty Period must not be empty!"),
        });
    }
    // if body.dimensions.is_empty() {
    //     return HttpResponse::BadRequest().json(BaseResponse {
    //         code: 400,
    //         message: String::from("Dimensions must not be empty!"),
    //     });
    // }

    let currency_id = match body.currency_id {
        Some(cur_id) => cur_id,
        None => match currency::get_default_currency_id(&client).await {
            Ok(cur_id) => cur_id,
            Err(err) => {
                println!("{:?}", err);
                0
            }
        },
    };
    if currency_id == 0 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Something went wrong with currency!"),
        });
    }

    if shop::get_shop_by_id(body.shop_id, &client).await.is_none() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("The shop ID provided does not exist. Please provide a valid shop ID to add a product!"),
        });
    }

    if category::get_category_by_id(body.category_id, &client)
        .await
        .is_none()
    {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("The category ID provided does not exist. Please provide a valid category ID to add a product!"),
        });
    }

    let mut creator_id = user_id;
    if role == "admin" {
        if let Some(id) = shop::get_creator_id_from_shop(body.shop_id, &client).await {
            creator_id = id;
        }
    }

    // let creator_id = if role != "admin" {
    //     user_id
    // } else if let Some(creator_id) = body.creator_id {
    //     creator_id
    // } else {
    //     user_id
    // };

    match product::add_product(&body, currency_id, creator_id, &client).await {
        Ok(()) => HttpResponse::Created().json(BaseResponse {
            code: 201,
            message: String::from("Product added successfully"),
        }),
        Err(e) => {
            eprintln!("Product adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding product!"),
            });
        }
    }
}

#[get("/api/products/{product_id}")]
pub async fn get_product_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let product_id = path.into_inner();
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

    match product::get_product_by_id(product_id, &client).await {
        Some(p) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Product fetched successfully."),
            data: Some(p),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Product not found!"),
        }),
    }
}

#[put("/api/products/{product_id}")]
pub async fn update_product(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<ProductRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let product_id = path.into_inner();
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

    // let user_id = parsed_values[0].parse().unwrap();
    let role: &str = parsed_values[1];

    if role != "admin" && role != "agent" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.model.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Model must not be empty!"),
        });
    }
    if body.description.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Description must not be empty!"),
        });
    }
    if body.color.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Color must not be empty!"),
        });
    }
    if body.strap_material.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Strap Material must not be empty!"),
        });
    }
    if body.strap_color.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Strap Color must not be empty!"),
        });
    }
    if body.case_material.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Case Material must not be empty!"),
        });
    }
    if body.dial_color.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Dial Color must not be empty!"),
        });
    }
    if body.movement_type.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Movement Type must not be empty!"),
        });
    }
    if body.water_resistance.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Water Resistance must not be empty!"),
        });
    }
    if body.warranty_period.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Warranty Period must not be empty!"),
        });
    }
    // if body.dimensions.is_empty() {
    //     return HttpResponse::BadRequest().json(BaseResponse {
    //         code: 400,
    //         message: String::from("Dimensions must not be empty!"),
    //     });
    // }

    let currency_id = match body.currency_id {
        Some(cur_id) => cur_id,
        None => match currency::get_default_currency_id(&client).await {
            Ok(cur_id) => cur_id,
            Err(err) => {
                println!("{:?}", err);
                0
            }
        },
    };
    if currency_id == 0 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Something went wrong with currency!"),
        });
    }

    match product::get_product_by_id(product_id, &client).await {
        Some(p) => {
            // let old_creator_id = p.creator_id;
            // let creator_id = if role != "admin" {
            //     old_creator_id
            // } else if let Some(creator_id) = body.creator_id {
            //     creator_id
            // } else {
            //     old_creator_id
            // };
            match product::update_product(
                product_id,
                &p.product_images,
                &body,
                currency_id,
                &client,
            )
            .await
            {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Product updated successfully"),
                }),
                Err(e) => {
                    eprintln!("Product updating error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error updating product!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Product not found!"),
        }),
    }
}

#[delete("/api/products/{product_id}")]
pub async fn delete_product(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let product_id = path.into_inner();
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

    match product::get_product_by_id(product_id, &client).await {
        Some(p) => match product::delete_product(product_id, &p.product_images, &client).await {
            Ok(()) => HttpResponse::Ok().json(BaseResponse {
                code: 204,
                message: String::from("Product deleted successfully"),
            }),
            Err(e) => {
                eprintln!("Product deleting error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error deleting product!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Product not found!"),
        }),
    }
}

#[get("/api/products/{product_id}/recommended")]
pub async fn get_recommended_products_for_product(
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
    let product_id = path.into_inner();
    match product::get_recommended_products_for_product(product_id, &client).await {
        Ok(products) => HttpResponse::Ok().json(DataResponse {
            code: 204,
            message: String::from("Recommended products fetched successfully"),
            data: Some(products),
        }),
        Err(e) => {
            eprintln!("Recommended products fetching error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error fetching recommended products!"),
            });
        }
    }
}

#[get("/api/recommended-products")]
pub async fn get_recommended_products_for_user(
    req: HttpRequest,
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

    match product::get_recommended_products_for_user(user_id, &client).await {
        Ok(products) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Recommended products fetched successfully"),
            data: Some(products),
        }),
        Err(e) => {
            eprintln!("Recommended products fetching error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error fetching recommended products!"),
            });
        }
    }
}
