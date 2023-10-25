use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::{
    models::seller_information,
    utils::common_struct::{BaseResponse, DataResponse},
};

#[get("/api/users/{user_id}/seller-information")]
pub async fn get_seller_information(
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> impl Responder {
    let user_id: i32 = path.into_inner();
    match seller_information::get_seller_information(user_id, &client).await {
        Some(seller_information) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: "Successful.".to_string(),
            data: Some(seller_information),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: "Seller information not found!".to_string(),
        }),
    }
}
