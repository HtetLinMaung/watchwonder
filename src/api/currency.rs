use std::sync::Arc;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::{
    models::currency,
    utils::{
        common_struct::{BaseResponse, DataResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[get("/api/currencies")]
pub async fn get_currencies(req: HttpRequest, client: web::Data<Arc<Client>>) -> impl Responder {
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

    match currency::get_currencies(&client).await {
        Ok(currencies) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Successful."),
            data: Some(currencies),
        }),
        Err(err) => {
            println!("{:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error fetching currencies"),
            })
        }
    }
}
