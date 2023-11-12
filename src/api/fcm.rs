use std::sync::Arc;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::fcm::{self, Fcm},
    utils::{common_struct::BaseResponse, fcm::send_notification, jwt::verify_token_and_get_sub},
};

#[post("/api/fcm/token")]
pub async fn add_fcm(
    req: HttpRequest,
    body: web::Json<Fcm>,
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

    let device_types: Vec<&str> = vec!["android", "ios", "web"];
    if !device_types.contains(&body.device_type.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Please select a valid device type: android, ios, or web."),
        });
    }

    match fcm::add_fcm_token(user_id, &body, &client).await {
        Ok(()) => HttpResponse::Created().json(BaseResponse {
            code: 201,
            message: String::from("FCM token added successfully"),
        }),
        Err(e) => {
            eprintln!("FCM token adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding FCM token!"),
            });
        }
    }
}

#[derive(Deserialize)]
pub struct NotifyAllRequest {
    pub title: String,
    pub message: String,
}

#[post("/api/fcm/notify-all")]
pub async fn notify_all(req: HttpRequest, body: web::Json<NotifyAllRequest>) -> impl Responder {
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

    match send_notification(&body.title, &body.message, "/topics/all", None).await {
        Ok(_) => {
            println!("notification message sent successfully.");
            HttpResponse::Ok().json(BaseResponse {
                code: 200,
                message: "Notification message sent successfully.".to_string(),
            })
        }
        Err(err) => {
            println!("{:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: "Fail to send notification!".to_string(),
            })
        }
    }
}
