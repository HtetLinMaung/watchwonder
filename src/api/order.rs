use std::sync::Arc;

use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDate;
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::order::{self, order_exists, NewOrder},
    utils::{
        common_struct::{BaseResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[post("/api/orders")]
pub async fn add_order(
    req: HttpRequest,
    order: web::Json<NewOrder>,
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

    match order::add_order(&order, user_id, &client).await {
        Ok(_) => HttpResponse::Ok().json(BaseResponse {
            code: 200,
            message: String::from("Successful."),
        }),
        Err(err) => {
            // Log the error message here
            println!("Error adding order: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to add order to database"),
            })
        }
    }
}

#[derive(Deserialize)]
pub struct GetOrdersQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub from_amount: Option<f64>,
    pub to_amount: Option<f64>,
}

#[get("/api/orders")]
pub async fn get_orders(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetOrdersQuery>,
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
    let role: &str = parsed_values[1];

    match order::get_orders(
        &query.search,
        query.page,
        query.per_page,
        &query.from_date,
        &query.to_date,
        &query.from_amount,
        &query.to_amount,
        user_id,
        role,
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
            println!("Error retrieving orders: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all orders from database"),
            })
        }
    }
}

#[derive(Deserialize)]
pub struct GetOrderItemsQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
    pub order_id: Option<i32>,
}

#[get("/api/order-items")]
pub async fn get_order_items(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetOrderItemsQuery>,
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
    let role: &str = parsed_values[1];

    match order::get_order_items(
        &query.search,
        query.page,
        query.per_page,
        &query.from_date,
        &query.to_date,
        query.order_id,
        user_id,
        role,
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
            println!("Error retrieving order items: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all order items from database"),
            })
        }
    }
}

#[derive(Deserialize)]
pub struct UpdateOrderRequest {
    pub status: String,
}

#[put("/api/orders/{order_id}")]
pub async fn update_order(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<UpdateOrderRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let order_id = path.into_inner();
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

    match order::order_exists(order_id, &client).await {
        Ok(exists) => {
            if !exists {
                return HttpResponse::NotFound().json(BaseResponse {
                    code: 404,
                    message: String::from("Order not found!"),
                });
            }
            match order::update_order(order_id, &body.status, &client).await {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Order updated successfully"),
                }),
                Err(e) => {
                    eprintln!("User updating error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error updating order!"),
                    });
                }
            }
        }
        Err(err) => {
            // Log the error message here
            println!("Error updating order: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to update order from database"),
            })
        }
    }
}
