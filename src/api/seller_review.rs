use std::sync::Arc;

use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::{
    models::seller_review::{self, SellerReviewRequest},
    utils::{
        common_struct::{BaseResponse, DataResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[post("/api/seller-reviews")]
pub async fn add_seller_review(
    req: HttpRequest,
    body: web::Json<SellerReviewRequest>,
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

    // if role != "user" && role != "admin" {
    //     return HttpResponse::Unauthorized().json(BaseResponse {
    //         code: 401,
    //         message: String::from("Unauthorized!"),
    //     });
    // }

    match seller_review::add_review(&body, user_id, &client).await {
        Ok(()) => HttpResponse::Created().json(BaseResponse {
            code: 200,
            message: String::from("Seller review added successfully"),
        }),
        Err(e) => {
            eprintln!("Seller review adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding seller review!"),
            });
        }
    }
}

#[get("/api/shops/{shop_id}/reviews")]
pub async fn get_seller_reviews(
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
    let shop_id = path.into_inner();
    match seller_review::get_seller_reviews(shop_id, &client).await {
        Ok(reviews) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Successful."),
            data: Some(reviews),
        }),
        Err(err) => {
            println!("{:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error fetching seller reviews!"),
            })
        }
    }
}
