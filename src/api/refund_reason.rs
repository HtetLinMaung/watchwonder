use std::sync::Arc;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::{
    models::{
        notification, product,
        refund_reason::{self, RefundReasonRequest},
        user,
    },
    utils::{common_struct::BaseResponse, jwt::verify_token_and_get_sub},
};

#[post("/api/refund-reasons")]
pub async fn add_refund_reason(
    req: HttpRequest,
    body: web::Json<RefundReasonRequest>,
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

    let user_id: i32 = parsed_values[0].parse().unwrap();
    let order_id = body.order_id;
    match refund_reason::add_refund_reason(&body, user_id, &client).await {
        Ok(()) => {
            tokio::spawn(async move {
                let user_name = match user::get_user_name(user_id, &client).await {
                    Some(name) => name,
                    None => "".to_string(),
                };
                let title = format!("Return Request Submitted");
                let message = format!(
                    "Return requested by {user_name} for Order ID #{order_id} - please review and process.",
                );
                match notification::add_notification_to_admins(&title, &message, &client).await {
                    Ok(()) => {
                        println!("Notification added successfully.");
                    }
                    Err(err) => {
                        println!("Error adding notification: {:?}", err);
                    }
                };

                let creator_id =
                    product::get_product_creator_id_from_order_id(order_id, &client).await;

                if creator_id != 0 {
                    match notification::add_notification(creator_id, &title, &message, &client)
                        .await
                    {
                        Ok(()) => {
                            println!("Notification added to seller successfully.");
                        }
                        Err(err) => {
                            println!("Error adding notification to seller: {:?}", err);
                        }
                    };
                }
            });
            HttpResponse::Created().json(BaseResponse {
                code: 201,
                message: String::from("Refund reason added successfully"),
            })
        }
        Err(e) => {
            eprintln!("Refund reason adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding refund reason!"),
            });
        }
    }
}
