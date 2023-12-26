use std::{collections::HashMap, sync::Arc};

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::Value;
use tokio_postgres::Client;

use crate::{
    models::{
        notification, product,
        shop::{self, ShopRequest},
        user,
    },
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct GetShopsQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub platform: Option<String>,
    pub version: Option<String>,
    pub status: Option<String>,
    pub view: Option<String>,
}

#[get("/api/shops")]
pub async fn get_shops(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetShopsQuery>,
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

    let mut role = "user".to_string();
    let mut user_id = 0;
    if !token.is_empty() {
        let sub = match verify_token_and_get_sub(&token) {
            Some(s) => s,
            None => {
                return HttpResponse::Unauthorized().json(BaseResponse {
                    code: 401,
                    message: String::from("Invalid token"),
                })
            }
        };
        // Parse the `sub` value
        let parsed_values: Vec<String> = sub.split(',').map(|s| s.to_string()).collect();
        if parsed_values.len() != 2 {
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Invalid sub format in token"),
            });
        }
        user_id = parsed_values[0].parse().unwrap();
        role = parsed_values[1].clone();
    }

    let platform = match &query.platform {
        Some(p) => p.as_str(),
        None => "",
    };
    let version = match &query.version {
        Some(v) => v.replace(".", "").parse().unwrap(),
        None => 0,
    };
    match shop::get_shops(
        &query.search,
        query.page,
        query.per_page,
        platform,
        &query.status,
        &query.view,
        &role,
        user_id,
        version,
        &client,
    )
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
            println!("Error retrieving shops: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all shops from database"),
            })
        }
    }
}

#[post("/api/shops")]
pub async fn add_shop(
    req: HttpRequest,
    body: web::Json<ShopRequest>,
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

    let user_id: i32 = parsed_values[0].parse().unwrap();
    let role: &str = parsed_values[1];

    if role != "admin" && role != "agent" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Name must not be empty!"),
        });
    }
    if body.cover_image.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Cover image must not be empty!"),
        });
    }

    let status_list: Vec<&str> = vec![
        "Active",
        "Inactive",
        "Closed",
        "Suspended",
        "Pending Approval",
    ];
    if !status_list.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from(
                "Please select a valid status: Active, Inactive, Closed, Suspended, or Pending Approval.",
            ),
        });
    }

    match shop::add_shop(&body, user_id, role, &client).await {
        Ok(shop_id) => {
            if role == "agent" {
                tokio::spawn(async move {
                    let name = user::get_user_name(user_id, &client).await.unwrap();
                    let title = format!("Shop Approval Required");
                    let message = format!(
                        "{name} has added a new shop, {}. Please review and approve.",
                        &body.name
                    );
                    let mut map = HashMap::new();
                    map.insert(
                        "redirect".to_string(),
                        Value::String("shop-approval-detail".to_string()),
                    );
                    map.insert("id".to_string(), Value::Number(shop_id.into()));
                    match notification::add_notification_to_admins(
                        &title,
                        &message,
                        &Some(map),
                        &client,
                    )
                    .await
                    {
                        Ok(()) => {
                            println!("Notification added successfully.");
                        }
                        Err(err) => {
                            println!("Error adding notification: {:?}", err);
                        }
                    };
                });
            }
            HttpResponse::Created().json(BaseResponse {
                code: 201,
                message: String::from("Shop added successfully"),
            })
        }
        Err(e) => {
            eprintln!("Shop adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding shop!"),
            });
        }
    }
}

#[get("/api/shops/{shop_id}")]
pub async fn get_shop_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let shop_id = path.into_inner();

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

    if !token.is_empty() && verify_token_and_get_sub(&token).is_none() {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Invalid token"),
        });
    }

    match shop::get_shop_by_id(shop_id, &client).await {
        Some(s) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Shop fetched successfully."),
            data: Some(s),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Shop not found!"),
        }),
    }
}

#[put("/api/shops/{shop_id}")]
pub async fn update_shop(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<ShopRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let shop_id = path.into_inner();
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

    if role != "admin" && role != "agent" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    if body.name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Name must not be empty!"),
        });
    }
    if body.cover_image.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Cover image must not be empty!"),
        });
    }

    let status_list: Vec<&str> = vec![
        "Active",
        "Inactive",
        "Closed",
        "Suspended",
        "Pending Approval",
    ];
    if !status_list.contains(&body.status.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from(
                "Please select a valid status: Active, Inactive, Closed, Suspended, or Pending Approval.",
            ),
        });
    }

    match shop::get_shop_by_id(shop_id, &client).await {
        Some(s) => {
            if &s.status != "Pending Approval" && role == "agent" {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Your shop has been approved by the admin and can no longer be updated. If you need to make changes, please contact customer support!"),
                });
            }
            match shop::update_shop(shop_id, &s.cover_image, &body, role, s.level, &client).await {
                Ok(()) => {
                    tokio::spawn(async move {
                        let title = format!("Shop Approved");
                        let message = format!(
                            "Your shop, {}, has been approved and is now live!",
                            &body.name
                        );
                        let mut map = HashMap::new();
                        map.insert(
                            "redirect".to_string(),
                            Value::String("shop-detail".to_string()),
                        );
                        map.insert("id".to_string(), Value::Number(shop_id.into()));
                        let clone_map = map.clone();
                        match notification::add_notification_to_admins(
                            &title,
                            &message,
                            &Some(map),
                            &client,
                        )
                        .await
                        {
                            Ok(()) => {
                                println!("Notification added successfully.");
                            }
                            Err(err) => {
                                println!("Error adding notification: {:?}", err);
                            }
                        };
                        match notification::add_notification(
                            s.creator_id,
                            &title,
                            &message,
                            &Some(clone_map),
                            &client,
                        )
                        .await
                        {
                            Ok(()) => {
                                println!("Notification added successfully.");
                            }
                            Err(err) => {
                                println!("Error adding notification: {:?}", err);
                            }
                        };
                    });
                    HttpResponse::Ok().json(BaseResponse {
                        code: 200,
                        message: String::from("Shop updated successfully"),
                    })
                }
                Err(e) => {
                    eprintln!("Shop updating error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error updating shop!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Shop not found!"),
        }),
    }
}

#[delete("/api/shops/{shop_id}")]
pub async fn delete_shop(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let shop_id = path.into_inner();
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

    if role != "admin" && role != "agent" {
        return HttpResponse::Unauthorized().json(BaseResponse {
            code: 401,
            message: String::from("Unauthorized!"),
        });
    }

    match product::is_products_exist("shop_id", shop_id, &client).await {
        Ok(is_exist) => {
            if is_exist {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Please delete the associated products first before deleting the shop. Ensure all products related to this shop are removed to proceed with shop deletion!"),
                });
            };
        }
        Err(err) => {
            println!("{:?}", err);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 400,
                message: String::from("Something went wrong with checking products existence!"),
            });
        }
    }

    match shop::get_shop_by_id(shop_id, &client).await {
        Some(s) => {
            if &s.status != "Pending Approval" && role == "agent" {
                return HttpResponse::BadRequest().json(BaseResponse {
                    code: 400,
                    message: String::from("Your shop has been approved by the admin and can no longer be deleted. If you need to make changes, please contact customer support!"),
                });
            }
            match shop::delete_shop(shop_id, &s.cover_image, &client).await {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 204,
                    message: String::from("Shop deleted successfully"),
                }),
                Err(e) => {
                    eprintln!("Shop deleting error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error deleting shop!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Shop not found!"),
        }),
    }
}
