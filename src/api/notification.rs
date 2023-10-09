use std::sync::Arc;

use actix_web::{get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::notification,
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct GetNotificationsBody {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub status_list: Option<Vec<String>>,
}

#[post("/api/get-notifications")]
pub async fn get_notifications(
    req: HttpRequest,
    body: web::Json<GetNotificationsBody>,
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
    // let role: &str = parsed_values[1];
    let user_id: i32 = user_id.parse().unwrap();

    match notification::get_notifications(
        &body.search,
        body.page,
        body.per_page,
        user_id,
        &body.status_list,
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
            println!("Error retrieving notifications: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all notifications from database"),
            })
        }
    }
}

#[get("/api/unread-notifications")]
pub async fn get_unread_counts(req: HttpRequest, client: web::Data<Arc<Client>>) -> impl Responder {
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

    match notification::get_unread_counts(user_id, &client).await {
        Ok(total) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Unread notification counts fetched successfully"),
            data: Some(total),
        }),
        Err(e) => {
            eprintln!("Unread notification counts fetching error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error fetching unread notification counts!"),
            });
        }
    }
}

#[derive(Deserialize)]
pub struct NotificationRequest {
    pub status: String,
}

#[put("/api/notifications/{notification_id}")]
pub async fn update_notification_status(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<NotificationRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let notification_id = path.into_inner();
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

    let status_list: Vec<&str> = vec!["Unread", "Read", "Acted", "Dismissed", "Archived"];
    if !status_list.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from(
                "Please select a valid status: Unread, Read, Acted, Dismissed, or Archived.",
            ),
        });
    }

    match notification::update_notification_status(notification_id, &body.status, &client).await {
        Ok(()) => HttpResponse::Ok().json(BaseResponse {
            code: 200,
            message: String::from("Notification status updated successfully"),
        }),
        Err(e) => {
            eprintln!("Notification status updating error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error updating notification status!"),
            });
        }
    }
}
