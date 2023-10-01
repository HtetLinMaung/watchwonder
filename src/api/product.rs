use std::sync::Arc;

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::product::{self, Product},
    utils::{common_struct::PaginationResponse, jwt::verify_token_and_get_sub},
};

#[derive(Deserialize)]
pub struct GetProductsRequestBody {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub brands: Option<Vec<i32>>,
    pub models: Option<Vec<String>>,
    pub from_price: Option<f64>,
    pub to_price: Option<f64>,
}

#[post("/api/get-products")]
pub async fn get_products(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    body: web::Json<GetProductsRequestBody>,
) -> impl Responder {
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(PaginationResponse::<Product> {
                    code: 400,
                    message: String::from("Invalid Authorization header format"),
                    data: vec![],
                    total: 0,
                    page: 0,
                    per_page: 0,
                    page_counts: 0,
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(PaginationResponse::<Product> {
                code: 401,
                message: String::from("Authorization header missing"),
                data: vec![],
                total: 0,
                page: 0,
                per_page: 0,
                page_counts: 0,
            })
        }
    };

    let sub = match verify_token_and_get_sub(token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(PaginationResponse::<Product> {
                code: 401,
                message: String::from("Invalid token"),
                data: vec![],
                total: 0,
                page: 0,
                per_page: 0,
                page_counts: 0,
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 2 {
        return HttpResponse::InternalServerError().json(PaginationResponse::<Product> {
            code: 500,
            message: String::from("Invalid sub format in token"),
            data: vec![],
            total: 0,
            page: 0,
            per_page: 0,
            page_counts: 0,
        });
    }

    // let user_id: &str = parsed_values[0];
    let role: &str = parsed_values[1];

    match product::get_products(
        &body.search,
        body.page,
        body.per_page,
        &body.brands,
        &body.models,
        body.from_price,
        body.to_price,
        role,
        &client,
    )
    .await
    {
        Ok(item_result) => HttpResponse::Ok().json(PaginationResponse {
            code: 200,
            message: String::from("Successful."),
            data: item_result.products,
            total: item_result.total,
            page: item_result.page,
            per_page: item_result.per_page,
            page_counts: item_result.page_counts,
        }),
        Err(err) => {
            // Log the error message here
            println!("Error retrieving products: {:?}", err);
            HttpResponse::InternalServerError().json(PaginationResponse::<Product> {
                code: 500,
                message: String::from("Error trying to read all products from database"),
                data: vec![],
                total: 0,
                page: 0,
                per_page: 0,
                page_counts: 0,
            })
        }
    }
}
