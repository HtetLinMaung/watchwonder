use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Result};
use futures::StreamExt;
use serde::Serialize;

use std::io::Write;
use uuid::Uuid;

#[derive(Serialize)]
pub struct UploadResponse {
    pub code: u16,
    pub message: String,
    pub url: String,
}

#[post("/api/upload")]
pub async fn upload(mut payload: Multipart) -> Result<HttpResponse> {
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();
        let original_name = content_disposition.get_filename().unwrap().to_string();
        let unique_id = Uuid::new_v4();
        let filename = format!("{}_{}", unique_id, original_name);
        let filepath = format!("./images/{}", filename);

        let mut file = web::block(move || std::fs::File::create(filepath.clone()))
            .await?
            .unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            file = web::block(move || file.write_all(&data).map(|_| file))
                .await?
                .unwrap();
        }

        let url = format!("/images/{}", filename);
        return Ok(HttpResponse::Ok().json(UploadResponse {
            code: 200,
            message: "Image uploaded successfully".to_string(),
            url,
        }));
    }

    Ok(HttpResponse::InternalServerError().json(UploadResponse {
        code: 500,
        message: "Image uploaded successfully".to_string(),
        url: "".to_string(),
    }))
}
