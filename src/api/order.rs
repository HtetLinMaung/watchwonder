use std::{collections::HashMap, sync::Arc};

use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::Client;

use crate::{
    models::{
        invoice, notification,
        order::{
            self, are_items_from_single_shop, are_items_same_currency_and_get_currency_id, NewOrder,
        },
        product, refund_reason, seller_review, user,
    },
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Serialize)]
pub struct AddOrderResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
    pub is_already_reviewed: bool,
}

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

    if !are_items_from_single_shop(&order.order_items, &client).await {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("You can only order items from one shop at a time. Please place separate orders for items from different shops!"),
        });
    }

    let currency_id =
        match are_items_same_currency_and_get_currency_id(&order.order_items, &client).await {
            Some(cur_id) => cur_id,
            None => 0,
        };

    if currency_id == 0 {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("You can only order items with the same currency at a time. Please place separate orders for items with different currencies!"),
        });
    }

    let payment_types: Vec<&str> = vec!["Full Prepaid", "Half Prepaid", "Cash on Delivery"];
    if !payment_types.contains(&order.payment_type.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from(
                "Please select a valid payment type: Full Prepaid, Half Prepaid, or Cash on Delivery.",
            ),
        });
    }

    if &order.payment_type != "Cash on Delivery" && order.payslip_screenshot_path.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Please provide your payment slip."),
        });
    }

    match order::update_stocks(&order.order_items, &client).await {
        Ok(success) => {
            if !success {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Insufficient stock!"),
                });
            }
        }
        Err(err) => {
            println!("{:?}", err);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error updating stocks!"),
            });
        }
    };

    match order::add_order(&order, user_id, currency_id, &client).await {
        Ok(order_id) => {
            let clone_client = client.clone();
            tokio::spawn(async move {
                match invoice::export_invoice(order_id, user_id, &clone_client).await {
                    Ok(_) => {
                        println!("Invoice exported successfully.")
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                };
            });
            let shop_id = order.shop_id.unwrap_or(0);
            let is_already_reviewed =
                seller_review::is_user_already_review(shop_id, user_id, &client).await;

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
                let title = format!("New Order #{order_id}");
                let message = format!("{user_name} has placed a {} order for {} from {}. Please review and process the order.",&order.payment_type.to_lowercase(), items.join(", "), shops.join(", "));
                let mut map = HashMap::new();
                map.insert(
                    "redirect".to_string(),
                    Value::String("order-detail".to_string()),
                );
                map.insert("id".to_string(), Value::Number(order_id.into()));
                let clone_map = map.clone();
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

                if let Some(product_creator_id) =
                    product::get_product_creator_id(order.order_items[0].product_id, &client).await
                {
                    match notification::add_notification(
                        product_creator_id,
                        &title,
                        &message,
                        &Some(clone_map),
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

            return HttpResponse::Ok().json(AddOrderResponse {
                code: 200,
                message: String::from("Successful."),
                data: Some(order_id),
                is_already_reviewed,
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
    pub status: Option<String>,
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
        &query.status,
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

    let user_id: i32 = parsed_values[0].parse().unwrap();
    let role: &str = parsed_values[1];
    let clone_role = role.to_string().clone();

    // if role != "admin" && role != "agent" {
    //     return HttpResponse::Unauthorized().json(BaseResponse {
    //         code: 401,
    //         message: String::from("Unauthorized!"),
    //     });
    // }
    if role == "user"
        && body.status.as_str() != "Cancelled"
        && body.status.as_str() != "Returned"
        && body.status.as_str() != "Completed"
    {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    } else if role == "agent" && !user::can_modify_order_status(user_id, &client).await {
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
                    let mut map = HashMap::new();
                    map.insert(
                        "redirect".to_string(),
                        Value::String("order-detail".to_string()),
                    );
                    map.insert("id".to_string(), Value::Number(order_id.into()));
                    let clone_map = map.clone();
                    match notification::add_notification(
                        client_id,
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

                    if &clone_role != "admin"
                        && (body.status.as_str() == "Cancelled"
                            || body.status.as_str() == "Returned"
                            || body.status.as_str() == "Completed")
                    {
                        let message = format!(
                            "Order #{order_id} has been {}.",
                            &body.status.to_lowercase()
                        );
                        match notification::add_notification_to_admins(
                            &title,
                            &message,
                            &Some(clone_map),
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

#[get("/api/orders/{order_id}/shop-name")]
pub async fn get_order_shop_name(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
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

    if verify_token_and_get_sub(token).is_none() {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Invalid token"),
        });
    }

    let shop_name = order::get_order_shop_name(order_id, &client).await;
    HttpResponse::Ok().json(DataResponse {
        code: 200,
        message: String::from("Successful."),
        data: Some(shop_name),
    })
}

#[get("/api/orders/{order_id}/refund-reason")]
pub async fn get_order_refund_reason(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
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

    if verify_token_and_get_sub(token).is_none() {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Invalid token"),
        });
    }

    match refund_reason::get_refund_reason_by_order_id(order_id, &client).await {
        Some(r) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Successful."),
            data: Some(r),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Refund reason not found!"),
        }),
    }
}

#[get("/api/orders/{order_id}/remind-seller")]
pub async fn remind_seller(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
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

    let user_id: i32 = parsed_values[0].parse().unwrap();

    match order::get_status_by_order_id(order_id, &client).await {
        Some(status) => {
            if status.as_str() == "Completed"
                || status.as_str() == "Refunded"
                || status.as_str() == "Canceled"
            {
                let message = format!("Order has been already {}", status.to_lowercase());
                return HttpResponse::BadRequest().json(BaseResponse { code: 400, message });
            }
            let mut map = HashMap::new();
            map.insert(
                "redirect".to_string(),
                Value::String("order-detail".to_string()),
            );
            map.insert("id".to_string(), Value::Number(order_id.into()));
            let clone_map = map.clone();

            let creator_id = product::get_product_creator_id_from_order_id(order_id, &client).await;

            match user::get_user_name(user_id, &client).await {
                Some(customer_name) => {
                    let title = format!("Urgent Delivery Alert");
                    let message = format!("Order ID #{order_id} for {customer_name} has been marked for urgent delivery - please oversee timely dispatch.");
                    tokio::spawn(async move {
                        match notification::add_notification(
                            creator_id,
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
                        match notification::add_notification_to_admins(
                            &title,
                            &message,
                            &Some(clone_map),
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

                    HttpResponse::Ok().json(BaseResponse {
                        code: 200,
                        message: "Reminder added successfully!".to_string(),
                    })
                }
                None => HttpResponse::NotFound().json(BaseResponse {
                    code: 404,
                    message: String::from("User not found!"),
                }),
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Order not found!"),
        }),
    }
}
