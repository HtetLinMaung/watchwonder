use std::sync::Arc;

use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDate;
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::{
        notification,
        order::{self, is_items_from_single_shop, NewOrder},
        product, user,
    },
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
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

    if order.order_items.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Order items must not be empty!"),
        });
    }

    if !is_items_from_single_shop(&order.order_items, &client).await {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("You can only order items from one shop at a time. Please place separate orders for items from different shops!"),
        });
    }

    let payment_types: Vec<&str> = vec!["Preorder", "Cash on Delivery"];
    if !payment_types.contains(&order.payment_type.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from(
                "Please select a valid payment type: Preorder, or Cash on Delivery.",
            ),
        });
    }

    if &order.payment_type == "Preorder" && order.payslip_screenshot_path.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Please provide your payment slip."),
        });
    }

    match order::add_order(&order, user_id, &client).await {
        Ok(order_id) => {
            tokio::spawn(async move {
                let user_name = match user::get_user_name(user_id, &client).await {
                    Some(name) => name,
                    None => "".to_string(),
                };
                let product_shop_names = match product::get_product_and_shop_names(
                    &order
                        .order_items
                        .iter()
                        .map(|item| item.product_id)
                        .collect(),
                    &client,
                )
                .await
                {
                    Ok(items) => items,
                    Err(_) => vec![],
                };

                let mut items: Vec<String> = vec![];
                let mut shops: Vec<String> = vec![];
                for product_shop_name in &product_shop_names {
                    items.push(product_shop_name.product_name.clone());
                    shops.push(product_shop_name.shop_name.clone());
                }
                let message = format!("{user_name} has placed a {} order for {} from {}. Please review and process the order.",&order.payment_type.to_lowercase(), items.join(", "), shops.join(", "));
                match notification::add_notification_to_admins("New Order", &message, &client).await
                {
                    Ok(()) => {
                        println!("Notification added successfully.");
                    }
                    Err(err) => {
                        println!("Error adding notification: {:?}", err);
                    }
                };

                if let Some(product_creator_id) =
                    product::get_product_creator_id(order.order_items[0].product_id, &client).await
                {
                    match notification::add_notification(
                        product_creator_id,
                        "New Order",
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
                }
            });
            return HttpResponse::Ok().json(DataResponse {
                code: 200,
                message: String::from("Successful."),
                data: Some(order_id),
            });
        }
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
    pub payment_type: Option<String>,
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
        &query.payment_type,
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

    if role != "admin" && role != "agent" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    let status_list: Vec<&str> = vec![
        "Pending",
        "Processing",
        "Shipped",
        "Delivered",
        "Completed",
        "Cancelled",
        "Refunded",
        "Failed",
        "On Hold",
        "Backordered",
        "Returned",
    ];
    if !status_list.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Please select a valid order status: Pending, Processing, Shipped, Delivered, Completed, Cancelled, Refunded, Failed, On Hold, Backordered, or Returned."),
        });
    }

    match order::get_user_id_by_order_id(order_id, &client).await {
        Some(client_id) => match order::update_order(order_id, &body.status, &client).await {
            Ok(()) => {
                tokio::spawn(async move {
                    let title = format!("Order {}", &body.status);
                    let status: &str = &body.status;
                    let message = match status {
                        "Processing" => {
                            format!("Your order #{order_id} is {}.", &body.status.to_lowercase())
                        }
                        _ => format!(
                            "Your order #{order_id} has been {}.",
                            &body.status.to_lowercase()
                        ),
                    };

                    match notification::add_notification(client_id, &title, &message, &client).await
                    {
                        Ok(()) => {
                            println!("Notification added successfully.");
                        }
                        Err(err) => {
                            println!("Error adding notification: {:?}", err);
                        }
                    };
                });
                return HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Order updated successfully"),
                });
            }
            Err(e) => {
                eprintln!("User updating error: {}", e);
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Error updating order!"),
                });
            }
        },
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Order not found!"),
        }),
    }
}
