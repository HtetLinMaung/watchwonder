use std::sync::Arc;

use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::chat::{self, MessageRequest},
    utils::{
        common_struct::{BaseResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct GetChatSessionsQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[get("/api/chat-sessions")]
pub async fn get_chat_sessions(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetChatSessionsQuery>,
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
    let role: &str = parsed_values[1];

    match chat::get_chat_sessions(
        &query.search,
        query.page,
        query.per_page,
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
            println!("Error retrieving chat sessions: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all chat sessions from database"),
            })
        }
    }
}

#[derive(Deserialize)]
pub struct GetChatMessagesQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[get("/api/chat-sessions/{chat_id}/chat-messages")]
pub async fn get_chat_messages(
    req: HttpRequest,
    path: web::Path<i32>,
    query: web::Query<GetChatMessagesQuery>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
    let chat_id = path.into_inner();
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

    match chat::get_chat_messages(&query.search, query.page, query.per_page, chat_id, &client).await
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
            println!("Error retrieving chat messages: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all chat messages from database"),
            })
        }
    }
}

#[post("/api/send-message")]
pub async fn send_message(
    req: HttpRequest,
    body: web::Json<MessageRequest>,
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
    // let role: &str = parsed_values[1];

    match chat::add_message(&body, user_id, &client).await {
        Ok(()) => HttpResponse::Created().json(BaseResponse {
            code: 201,
            message: String::from("Message sent successfully"),
        }),
        Err(e) => {
            eprintln!("Message sending error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error sending message!"),
            });
        }
    }
}
