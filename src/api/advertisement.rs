use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::advertisement::{self, AdvertisementRequest},
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct GetAdvertisementsQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[get("/api/advertisements")]
pub async fn get_advertisements(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetAdvertisementsQuery>,
) -> impl Responder {
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1].to_string()
            } else {
                "".to_string()
            }
        }
        None => "".to_string(),
    };

    if !token.is_empty() {
        if verify_token_and_get_sub(&token).is_none() {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            });
        }
    }
    match advertisement::get_advertisements(&query.search, query.page, query.per_page, &client)
        .await
    {
        Ok(item_result) => HttpResponse::Ok().json(PaginationResponse {
            code: 200,
            message: String::from("Successful."),
            data: item_result.data,
            total: item_result.total,
            page: item_result.page,
            per_page: item_result.per_page,
            page_counts: item_result.page_counts,
        }),
        Err(err) => {
            // Log the error message here
            println!("Error retrieving advertisements: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all advertisements from database"),
            })
        }
    }
}

#[post("/api/advertisements")]
pub async fn add_advertisement(
    req: HttpRequest,
    body: web::Json<AdvertisementRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Invalid Authorization header format"),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Authorization header missing"),
            })
        }
    };

    let sub = match verify_token_and_get_sub(token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 2 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Invalid sub format in token"),
        });
    }

    // let user_id = parsed_values[0].parse().unwrap();
    let role: &str = parsed_values[1];

    if role != "admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.media_url.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Media url must not be empty!"),
        });
    }

    let media_types: Vec<&str> = vec!["image", "video"];
    if !media_types.contains(&body.media_type.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Please select a valid media type: image or video."),
        });
    }

    match advertisement::add_advertisement(&body, &client).await {
        Ok(_) => HttpResponse::Created().json(BaseResponse {
            code: 201,
            message: String::from("Advertisement added successfully"),
        }),
        Err(e) => {
            eprintln!("Advertisement adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding advertisement!"),
            });
        }
    }
}

#[get("/api/advertisements/{advertisement_id}")]
pub async fn get_advertisement_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let advertisement_id = path.into_inner();
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Invalid Authorization header format"),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Authorization header missing"),
            })
        }
    };

    let sub = match verify_token_and_get_sub(token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 2 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Invalid sub format in token"),
        });
    }

    let role: &str = parsed_values[1];

    if role != "admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match advertisement::get_advertisement_by_id(advertisement_id, &client).await {
        Some(a) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Advertisement fetched successfully."),
            data: Some(a),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Advertisement not found!"),
        }),
    }
}

#[put("/api/advertisements/{advertisement_id}")]
pub async fn update_advertisement(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<AdvertisementRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let advertisement_id = path.into_inner();
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Invalid Authorization header format"),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Authorization header missing"),
            })
        }
    };

    let sub = match verify_token_and_get_sub(token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 2 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Invalid sub format in token"),
        });
    }

    let role: &str = parsed_values[1];

    if role != "admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.media_url.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Media url must not be empty!"),
        });
    }

    let media_types: Vec<&str> = vec!["image", "video"];
    if !media_types.contains(&body.media_type.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Please select a valid media type: image or video."),
        });
    }

    match advertisement::get_advertisement_by_id(advertisement_id, &client).await {
        Some(a) => {
            match advertisement::update_advertisement(
                advertisement_id,
                &body,
                &a.media_url,
                &client,
            )
            .await
            {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Advertisement updated successfully"),
                }),
                Err(e) => {
                    eprintln!("Advertisement updating error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error updating advertisement!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Advertisement not found!"),
        }),
    }
}

#[delete("/api/advertisements/{advertisement_id}")]
pub async fn delete_advertisement(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let advertisement_id = path.into_inner();
    // Extract the token from the Authorization header
    let token = match req.headers().get("Authorization") {
        Some(value) => {
            let parts: Vec<&str> = value.to_str().unwrap_or("").split_whitespace().collect();
            if parts.len() == 2 && parts[0] == "Bearer" {
                parts[1]
            } else {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Invalid Authorization header format"),
                });
            }
        }
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Authorization header missing"),
            })
        }
    };

    let sub = match verify_token_and_get_sub(token) {
        Some(s) => s,
        None => {
            return HttpResponse::Unauthorized().json(BaseResponse {
                code: 401,
                message: String::from("Invalid token"),
            })
        }
    };

    // Parse the `sub` value
    let parsed_values: Vec<&str> = sub.split(',').collect();
    if parsed_values.len() != 2 {
        return HttpResponse::InternalServerError().json(BaseResponse {
            code: 500,
            message: String::from("Invalid sub format in token"),
        });
    }

    let role: &str = parsed_values[1];

    if role != "admin" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match advertisement::get_advertisement_by_id(advertisement_id, &client).await {
        Some(a) => {
            match advertisement::delete_advertisement(advertisement_id, &a.media_url, &client).await
            {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 204,
                    message: String::from("Advertisement deleted successfully"),
                }),
                Err(e) => {
                    eprintln!("Advertisement deleting error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error deleting advertisement!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Advertisement not found!"),
        }),
    }
}
