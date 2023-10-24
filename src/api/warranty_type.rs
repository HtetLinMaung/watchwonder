use std::sync::Arc;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::{
    models::warranty_type,
    utils::{
        common_struct::{BaseResponse, DataResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[get("/api/warranty-types")]
pub async fn get_warranty_types(
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

    // let user_id: i32 = parsed_values[0].parse().unwrap();
    let role: &str = parsed_values[1];

    if role != "admin" && role != "agent" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match warranty_type::get_warranty_types(&client).await {
        Ok(warranty_types) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Successful."),
            data: Some(warranty_types),
        }),
        Err(err) => {
            // Log the error message here
            println!("Error retrieving warranty types: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all warranty types from database"),
            })
        }
    }
}
