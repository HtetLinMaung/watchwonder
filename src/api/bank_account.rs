use std::sync::Arc;

use actix_web::{delete, get, post, put, web, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tokio_postgres::Client;

use crate::{
    models::bank_account::{self, BankAccountRequest},
    utils::{
        common_struct::{BaseResponse, DataResponse, PaginationResponse},
        jwt::verify_token_and_get_sub,
    },
};

#[derive(Deserialize)]
pub struct GetBanksQuery {
    pub search: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub account_type: Option<String>,
}

#[get("/api/bank-accounts")]
pub async fn get_bank_accounts(
    req: HttpRequest,
    client: web::Data<Arc<Client>>,
    query: web::Query<GetBanksQuery>,
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
    match bank_account::get_bank_accounts(
        &query.search,
        query.page,
        query.per_page,
        &query.account_type,
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
            println!("Error retrieving bank accounts: {:?}", err);
            HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error trying to read all bank accounts from database"),
            })
        }
    }
}

#[post("/api/bank-accounts")]
pub async fn add_bank_account(
    req: HttpRequest,
    body: web::Json<BankAccountRequest>,
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

    if body.account_holder_name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Account holder name must not be empty!"),
        });
    }
    if body.account_number.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Account number must not be empty!"),
        });
    }

    let account_types: Vec<&str> = vec!["mbanking", "wallet"];
    if !account_types.contains(&body.account_type.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Please select a valid account type: mbanking or wallet."),
        });
    }

    match bank_account::add_bank_account(&body, &client).await {
        Ok(()) => HttpResponse::Created().json(BaseResponse {
            code: 201,
            message: String::from("Bank account added successfully"),
        }),
        Err(e) => {
            eprintln!("Bank account adding error: {}", e);
            return HttpResponse::InternalServerError().json(BaseResponse {
                code: 500,
                message: String::from("Error adding bank account!"),
            });
        }
    }
}

#[get("/api/bank-accounts/{account_id}")]
pub async fn get_bank_account_by_id(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let account_id = path.into_inner();
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

    match bank_account::get_bank_account_by_id(account_id, &client).await {
        Some(c) => HttpResponse::Ok().json(DataResponse {
            code: 200,
            message: String::from("Bank account fetched successfully."),
            data: Some(c),
        }),
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Bank account not found!"),
        }),
    }
}

#[put("/api/bank-accounts/{account_id}")]
pub async fn update_bank_account(
    req: HttpRequest,
    path: web::Path<i32>,
    body: web::Json<BankAccountRequest>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let account_id = path.into_inner();
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

    if body.account_holder_name.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Account holder name must not be empty!"),
        });
    }
    if body.account_number.is_empty() {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Account number must not be empty!"),
        });
    }

    let account_types: Vec<&str> = vec!["mbanking", "wallet"];
    if !account_types.contains(&body.account_type.as_str()) {
        return HttpResponse::BadRequest().json(BaseResponse {
            code: 400,
            message: String::from("Please select a valid account type: mbanking or wallet."),
        });
    }

    match bank_account::get_bank_account_by_id(account_id, &client).await {
        Some(ba) => {
            match bank_account::update_bank_account(account_id, &ba.bank_logo, &body, &client).await
            {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 200,
                    message: String::from("Bank account updated successfully"),
                }),
                Err(e) => {
                    eprintln!("Bank account updating error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error updating bank account!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Bank account not found!"),
        }),
    }
}

#[delete("/api/bank-accounts/{account_id}")]
pub async fn delete_bank_account(
    req: HttpRequest,
    path: web::Path<i32>,
    client: web::Data<Arc<Client>>,
) -> HttpResponse {
    let account_id = path.into_inner();
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

    match bank_account::get_bank_account_by_id(account_id, &client).await {
        Some(ba) => {
            match bank_account::delete_bank_account(account_id, &ba.bank_logo, &client).await {
                Ok(()) => HttpResponse::Ok().json(BaseResponse {
                    code: 204,
                    message: String::from("Bank account deleted successfully"),
                }),
                Err(e) => {
                    eprintln!("Bank account deleting error: {}", e);
                    return HttpResponse::InternalServerError().json(BaseResponse {
                        code: 500,
                        message: String::from("Error deleting bank account!"),
                    });
                }
            }
        }
        None => HttpResponse::NotFound().json(BaseResponse {
            code: 404,
            message: String::from("Bank account not found!"),
        }),
    }
}
