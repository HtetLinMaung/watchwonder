use std::sync::Arc;

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

use crate::{
    models::category::{self, Category},
    utils::jwt::verify_token_and_get_sub,
};

#[derive(Serialize)]
pub struct GetCategoriesResponse {
    pub code: u16,
    pub message: String,
    pub data: Vec<Category>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub page_counts: usize,
}

#[derive(Deserialize)]
pub struct GetCategoriesQuery {
    pub search: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[get("/api/categories")]
pub async fn get_categories(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetCategoriesQuery>,
) -> impl Responder {
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(GetCategoriesResponse {
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
            return HttpResponse::Unauthorized().json(GetCategoriesResponse {
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
            return HttpResponse::Unauthorized().json(GetCategoriesResponse {
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
        return HttpResponse::InternalServerError().json(GetCategoriesResponse {
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
    // let role_name: &str = parsed_values[1];

    match category::get_categories(&query.search, &query.page, &query.per_page, &client).await {
        Ok(item_result) => HttpResponse::Ok().json(GetCategoriesResponse {
            code: 200,
            message: String::from("Successful."),
            data: item_result.categories,
            total: item_result.total,
            page: item_result.page,
            per_page: item_result.per_page,
            page_counts: item_result.page_counts,
        }),
        Err(err) => {
            // Log the error message here
            println!("Error retrieving categories: {:?}", err);
            HttpResponse::InternalServerError().json(GetCategoriesResponse {
                code: 500,
                message: String::from("Error trying to read all categories from database"),
                data: vec![],
                total: 0,
                page: 0,
                per_page: 0,
                page_counts: 0,
            })
        }
    }
}
