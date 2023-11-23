use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::utils::{
    common_struct::{BaseResponse, DataResponse},
    google,
};

#[derive(Deserialize)]
pub struct VerifyGoogleTokenRequest {
    pub id_token: String,
}

#[post("/api/verify-google-token")]
pub async fn verify_google_token(body: web::Json<VerifyGoogleTokenRequest>) -> impl Responder {
    match google::verify_id_token_with_google_api_original(&body.id_token).await {
        Ok(res) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Successful"),
            data: Some(res),
        }),
        Err(e) => {
            println!("{:?}", e);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Internal Server Error"),
            })
        }
    }
}
