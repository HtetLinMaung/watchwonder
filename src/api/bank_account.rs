use std::sync::Arc;

use actix_web::{get, web, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::{
    models::bank_account,
    utils::common_struct::{BaseResponse, DataResponse},
};

#[get("/api/bank-accounts")]
pub async fn get_bank_accounts(client: web::Data<Arc<Client>>) -> impl Responder {
    match bank_account::get_bank_accounts(&client).await {
        Ok(bank_accounts) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Successful."),
            data: Some(bank_accounts),
        }),
        Err(err) => {
            // Log the error message here
            println!("Error retrieving bank accounts: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all bank accounts from database"),
            })
        }
    }
}
