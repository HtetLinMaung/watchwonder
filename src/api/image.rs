use crate::utils::{common_struct::BaseResponse, image::get_image_format_from_path};
use actix_multipart::Multipart;
use actix_web::{post, web, HttpResponse, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write};
use uuid::Uuid;

#[derive(Serialize)]
pub struct UploadResponse {
    pub code: u16,
    pub message: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct ResolutionInfo {
    resolution: Option<String>,
}

#[post("/api/image/upload")]
pub async fn upload(
    web::Query(info): web::Query<ResolutionInfo>,
    mut payload: Multipart,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
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

        // Resize the image if resolution parameter is provided
        if let Some(resolution) = &info.resolution {
            let parts: Vec<&str> = resolution.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(width), Ok(height)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                {
                    let img_path = format!("./images/{}", filename);
                    match image::open(img_path) {
                        Ok(img) => {
                            let resized = img.resize_exact(
                                width,
                                height,
                                image::imageops::FilterType::Triangle,
                            );
                            // Determine the format based on the original image's format
                            let format = get_image_format_from_path(
                                format!("./images/{}", filename).as_str(),
                            )
                            .unwrap_or(image::ImageFormat::Png);
                            if let Err(e) =
                                resized.save_with_format(format!("./images/{}", filename), format)
                            {
                                eprintln!("Resized image saving error: {}", e);
                                match fs::remove_file(format!("./images/{}", filename)) {
                                    Ok(_) => println!("File deleted successfully!"),
                                    Err(e) => println!("Error deleting file: {}", e),
                                };
                                return Ok(HttpResponse::InternalServerError().json(
                                    BaseResponse {
                                        code: 500,
                                        message: String::from("Error resizing image!"),
                                    },
                                ));
                            }
                        }
                        Err(e) => {
                            eprintln!("Image opening error: {}", e);
                            match fs::remove_file(format!("./images/{}", filename)) {
                                Ok(_) => println!("File deleted successfully!"),
                                Err(e) => println!("Error deleting file: {}", e),
                            };
                            return Ok(HttpResponse::InternalServerError().json(BaseResponse {
                                code: 500,
                                message: String::from("Error resizing image!"),
                            }));
                        }
                    }
                }
            }
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
        message: "Image upload failed".to_string(),
        url: "".to_string(),
    }))
}

#[derive(Deserialize)]
pub struct ResizeRequest {
    pub image_path: String,
    pub resolution: String,
}

#[post("/api/image/resize")]
pub async fn resize_image(
    body: web::Json<ResizeRequest>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = body.resolution.split('x').collect();
    if parts.len() == 2 {
        if let (Ok(width), Ok(height)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            let img_path = &body.image_path;
            match image::open(img_path) {
                Ok(img) => {
                    let resized =
                        img.resize_exact(width, height, image::imageops::FilterType::Triangle);

                    // Determine the format based on the original image's format
                    let format =
                        get_image_format_from_path(img_path).unwrap_or(image::ImageFormat::Png);

                    if let Err(e) = resized.save_with_format(&body.image_path, format) {
                        eprintln!("Resized image saving error: {}", e);
                        return Ok(HttpResponse::InternalServerError().json(BaseResponse {
                            code: 500,
                            message: String::from("Error resizing image!"),
                        }));
                    }
                }
                Err(e) => {
                    eprintln!("Image opening error: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error resizing image!"),
                    }));
                }
            }
        }
    }
    return Ok(HttpResponse::Ok().json(BaseResponse {
        code: 200,
        message: "Image resized successfully".to_string(),
    }));
}
