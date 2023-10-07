use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

use crate::{
    models::category::{self, Category},
    utils::common_struct::BaseResponse,
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
    client: web::Data<Arc<Client>>,
    query: web::Query<GetCategoriesQuery>,
) -> impl Responder {
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
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all categories from database"),
            })
        }
    }
}
