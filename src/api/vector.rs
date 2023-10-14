use std::fs;

use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;

use crate::utils::{
    common_struct::{BaseResponse, DataResponse},
    vector_finder,
};

#[derive(Deserialize)]
pub struct SearchRequest {
    pub image_path: String,
}

#[derive(Debug, Deserialize)]
struct VectorData {
    // vector_id: i32,
    label: String,
    // distance: f64,
}

#[post("/api/vectors/search")]
pub async fn search_vectors(body: web::Json<SearchRequest>) -> impl Responder {
    match vector_finder::search_vectors(&body.image_path).await {
        Ok(response) => {
            let clone_image_path = body.image_path.clone().replace("/images", "./images");
            tokio::spawn(async move {
                match fs::remove_file(clone_image_path) {
                    Ok(_) => println!("File deleted successfully!"),
                    Err(e) => println!("Error deleting file: {}", e),
                };
            });
            // println!("Vector added successfully.");
            println!("{:?}", response);
            // Extracting code, data, and message from the response
            let code = response.get("code").and_then(Value::as_i64).unwrap_or(0) as i32;
            let data: Vec<VectorData> =
                serde_json::from_value(response.get("data").cloned().unwrap_or(Value::Null))
                    .unwrap_or_default();
            // let message = response
            //     .get("message")
            //     .and_then(Value::as_str)
            //     .unwrap_or("")
            //     .to_string();
            if code != 200 {
                return HttpResponse::InternalServerError().json(BaseResponse {
                    code: 500,
                    message: String::from("Something went wrong!"),
                });
            }
            let products: Vec<i32> = data.iter().map(|d| d.label.parse().unwrap()).collect();
            return HttpResponse::Ok().json(DataResponse {
                code: 200,
                message: String::from("Vector search successfully."),
                data: Some(products),
            });
        }
        Err(err) => {
            println!("Error searching vectors: {:?}", err);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Something went wrong!"),
            });
        }
    }
}
