use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::{
    models::counter,
    utils::common_struct::{BaseResponse, DataResponse},
};

#[get("/api/counters/generate-invoice-id")]
pub async fn generate_invoice_id(client: web::Data<Arc<Client>>) -> impl Responder {
    match counter::generate_invoice_id(&client).await {
        Ok(invoice_id) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: "Invoice ID generated successfully.".to_string(),
            data: Some(invoice_id),
        }),
        Err(err) => {
            println!("{:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: "Invoice ID generation failed!".to_string(),
            })
        }
    }
}
