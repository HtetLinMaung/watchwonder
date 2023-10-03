use std::sync::Arc;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::{
    models::order::{self, NewOrder},
    utils::{common_struct::BaseResponse, jwt::verify_token_and_get_sub},
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
