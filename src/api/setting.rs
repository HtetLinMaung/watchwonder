use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;

use crate::utils::{
    common_struct::DataResponse,
    setting::{get_android_version, get_ios_version, get_version_update_message},
};

#[derive(Serialize)]
pub struct Setting {
    pub platform_required_signin: String,
    pub android_version: String,
    pub ios_version: String,
    pub version_update_message: String,
}

#[get("/api/settings")]
pub async fn get_settings() -> impl Responder {
    let platform_required_signin =
        std::env::var("PLATFORM_REQUIRED_SIGNIN").unwrap_or("ios".to_string());
    HttpResponse::Ok().json(DataResponse {
        code: 200,
        message: String::from("Successful."),
        data: Some(Setting {
            platform_required_signin,
            android_version: get_android_version(),
            ios_version: get_ios_version(),
            version_update_message: get_version_update_message(),
        }),
    })
}
