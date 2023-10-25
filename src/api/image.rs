use crate::{
    models::{bank_account, brand, category, product, shop, user},
    utils::{common_struct::BaseResponse, image::get_image_format_from_path},
};
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder, Result};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, read_dir, remove_file},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
    vec,
};
use tokio_postgres::Client;
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
        let path = Path::new(&original_name);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let unique_id = Uuid::new_v4();

        let original_filename = format!("{}_{}_original.{}", unique_id, stem, extension);
        let original_filepath = format!("./images/{}", original_filename);

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

        match fs::copy(format!("./images/{}", filename), &original_filepath) {
            Ok(_) => {
                println!("File copied successfully!");
            }
            Err(e) => {
                println!("Failed to copy file: {}", e);
            }
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
                            // Check if the original image dimensions are smaller than the target dimensions
                            if img.width() < width || img.height() < height {
                                let url = format!("/images/{}", filename);
                                return Ok(HttpResponse::Ok().json(UploadResponse {
                                    code: 200,
                                    message: "Original image resolution is lower than the given resolution. No resizing performed.".to_string(),
                                    url
                                }));
                            }

                            let resized =
                                img.resize(width, height, image::imageops::FilterType::Lanczos3);
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
                    let resized = img.resize(width, height, image::imageops::FilterType::Lanczos3);

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

#[get("/api/remove-dangling-images")]
pub async fn remove_dangling_images(client: web::Data<Arc<Client>>) -> impl Responder {
    let mut images: Vec<PathBuf> = vec![PathBuf::from("./images/policy.html")];
    let user_profile_images = user::get_profile_images(&client).await;
    for profile_image in user_profile_images {
        let profile_file_path = profile_image.replace("/images", "./images");
        images.push(PathBuf::from(&profile_file_path));
        let path = Path::new(&profile_file_path);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        images.push(PathBuf::from(format!("{stem}_original.{extension}")));
    }
    let bank_logos = bank_account::get_bank_logos(&client).await;
    for bank_logo in bank_logos {
        let bank_logo_path = bank_logo.replace("/images", "./images");
        images.push(PathBuf::from(&bank_logo_path));
        let path = Path::new(&bank_logo_path);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        images.push(PathBuf::from(format!("{stem}_original.{extension}")));
    }

    let shop_cover_images = shop::get_cover_images(&client).await;
    for cover_image in shop_cover_images {
        let cover_image_path = cover_image.replace("/images", "./images");
        images.push(PathBuf::from(&cover_image_path));
        let path = Path::new(&cover_image_path);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        images.push(PathBuf::from(format!("{stem}_original.{extension}")));
    }

    let category_cover_images = category::get_cover_images(&client).await;
    for cover_image in category_cover_images {
        let cover_image_path = cover_image.replace("/images", "./images");
        images.push(PathBuf::from(&cover_image_path));
        let path = Path::new(&cover_image_path);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        images.push(PathBuf::from(format!("{stem}_original.{extension}")));
    }

    let brand_logo_urls = brand::get_logo_urls(&client).await;
    for logo_url in brand_logo_urls {
        let logo_url_path = logo_url.replace("/images", "./images");
        images.push(PathBuf::from(&logo_url_path));
        let path = Path::new(&logo_url_path);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        images.push(PathBuf::from(format!("{stem}_original.{extension}")));
    }

    let product_images = product::get_product_images(&client).await;
    for product_image in product_images {
        let product_image_path = product_image.replace("/images", "./images");
        images.push(PathBuf::from(&product_image_path));
        let path = Path::new(&product_image_path);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        images.push(PathBuf::from(format!("{stem}_original.{extension}")));
    }

    let path = Path::new("./images");

    match read_dir(&path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(e) => {
                        // println!("{:?}", e.path());
                        let p = e.path();
                        if !images.contains(&p) {
                            match remove_file(&p) {
                                Ok(_) => println!("Successfully deleted {:?}", p),
                                Err(err) => eprintln!("Failed to delete {:?} due to {}", p, err),
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading directory entry: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading directory: {}", e);
        }
    }

    HttpResponse::Ok().json(BaseResponse {
        code: 200,
        message: String::from("Image removed successfully"),
    })
}
