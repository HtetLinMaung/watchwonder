use std::{fs, path::Path};

use bcrypt::{hash, DEFAULT_COST};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

use super::seller_information::{SellerInformation, SellerInformationRequest};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
    pub name: String,
    pub profile_image: String,
    pub email: String,
    pub phone: String,
    pub account_status: String,
    pub can_modify_order_status: bool,
    pub can_view_address: bool,
    pub can_view_phone: bool,
    pub created_at: NaiveDateTime,
    pub seller_information: Option<SellerInformation>,
}

pub async fn user_exists(username: &str, client: &Client) -> Result<bool, Error> {
    // Execute a query to check if the username exists in the users table
    let row = client
        .query_one(
            "SELECT username FROM users WHERE username = $1 and deleted_at is null",
            &[&username],
        )
        .await;

    // Return whether the user exists
    Ok(row.is_ok())
}

pub async fn get_user_by_id(user_id: i32, client: &Client) -> Option<User> {
    let result = client
        .query_one(
            "select u.user_id, u.username, u.password, u.role, u.name, u.profile_image, u.email, u.phone, u.account_status, u.can_modify_order_status, u.can_view_address, u.can_view_phone, u.created_at, coalesce(si.company_name, '') as company_name, coalesce(si.professional_title, '') as professional_title, coalesce(si.active_since_year, 0) as active_since_year, coalesce(si.location, '') as location, coalesce(si.offline_trader, false) as offline_trader, coalesce(si.facebook_profile_image, '') as facebook_profile_image, coalesce(si.shop_or_page_name, '') as shop_or_page_name, coalesce(si.facebook_page_image, '') as facebook_page_image, coalesce(si.bussiness_phone, '') as bussiness_phone, coalesce(si.address, '') as address, coalesce(si.nrc, '') as nrc, coalesce(si.nrc_front_image, '') as nrc_front_image, coalesce(si.nrc_back_image, '') as nrc_back_image, coalesce(si.bank_code, '') as bank_code, coalesce(si.bank_account, '') as bank_account, coalesce(si.bank_account_image, '') as bank_account_image, coalesce(si.wallet_type, '') as wallet_type, coalesce(si.wallet_account, '') as wallet_account, coalesce(si.fee_id, 0) as fee_id, coalesce(si.monthly_transaction_screenshot, '') as monthly_transaction_screenshot, coalesce(si.passport_image, '') as passport_image, coalesce(si.driving_licence_image, '') as driving_licence_image, coalesce(si.signature_image, '') as signature_image from users u left join seller_informations si on u.user_id = si.user_id and si.deleted_at is null where u.user_id = $1 and u.deleted_at is null",
            &[&user_id],
        )
        .await;

    match result {
        Ok(row) => Some(User {
            user_id: row.get("user_id"),
            username: row.get("username"),
            password: row.get("password"),
            role: row.get("role"),
            name: row.get("name"),
            profile_image: row.get("profile_image"),
            email: row.get("email"),
            phone: row.get("phone"),
            account_status: row.get("account_status"),
            can_modify_order_status: row.get("can_modify_order_status"),
            can_view_address: row.get("can_view_address"),
            can_view_phone: row.get("can_view_phone"),
            created_at: row.get("created_at"),
            seller_information: Some(SellerInformation {
                company_name: row.get("company_name"),
                professional_title: row.get("professional_title"),
                active_since_year: row.get("active_since_year"),
                location: row.get("location"),
                offline_trader: row.get("offline_trader"),
                product_counts: 0,
                sold_product_counts: 0,
                seller_name: row.get("name"),
                seller_profile_image: row.get("profile_image"),
                facebook_profile_image: row.get("facebook_profile_image"),
                shop_or_page_name: row.get("shop_or_page_name"),
                facebook_page_image: row.get("facebook_page_image"),
                bussiness_phone: row.get("bussiness_phone"),
                address: row.get("address"),
                nrc: row.get("nrc"),
                nrc_front_image: row.get("nrc_front_image"),
                nrc_back_image: row.get("nrc_back_image"),
                bank_code: row.get("bank_code"),
                bank_account: row.get("bank_account"),
                bank_account_image: row.get("bank_account_image"),
                wallet_type: row.get("wallet_type"),
                wallet_account: row.get("wallet_account"),
                fee_id: row.get("fee_id"),
                monthly_transaction_screenshot: row.get("monthly_transaction_screenshot"),
                passport_image: row.get("passport_image"),
                driving_licence_image: row.get("driving_licence_image"),
                signature_image: row.get("signature_image"),
            }),
        }),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}

pub async fn add_user(
    name: &str,
    username: &str,
    password: &str,
    email: &str,
    phone: &str,
    profile_image: &str,
    role: &str,
    account_status: &str,
    can_modify_order_status: bool,
    can_view_address: bool,
    can_view_phone: bool,
    seller_information: &Option<SellerInformationRequest>,
    google_id: &Option<String>,
    request_to_agent: bool,
    client: &Client,
) -> Result<i32, Box<dyn std::error::Error>> {
    let hashed_password =
        hash(&password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;

    // Insert the new user into the database
    let row =client.query_one(
        "INSERT INTO users (name, username, password, email, phone, profile_image, role, account_status, can_modify_order_status, can_view_address, can_view_phone, google_id, request_to_agent) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) returning user_id",
        &[&name, &username, &hashed_password, &email, &phone, &profile_image, &role, &account_status, &can_modify_order_status, &can_view_address, &can_view_phone, &google_id, &request_to_agent],
    ).await?;
    let user_id: i32 = row.get("user_id");
    if role == "agent" {
        if let Some(si) = seller_information {
            let facebook_profile_image = if let Some(fpi) = &si.facebook_profile_image {
                fpi
            } else {
                ""
            };

            let shop_or_page_name = if let Some(spn) = &si.shop_or_page_name {
                spn
            } else {
                ""
            };

            let facebook_page_image = if let Some(fpi) = &si.facebook_page_image {
                fpi
            } else {
                ""
            };

            let bussiness_phone = if let Some(bp) = &si.bussiness_phone {
                bp
            } else {
                ""
            };

            let address = if let Some(a) = &si.address { a } else { "" };

            let nrc = if let Some(n) = &si.nrc { n } else { "" };

            let nrc_front_image = if let Some(n) = &si.nrc_front_image {
                n
            } else {
                ""
            };

            let nrc_back_image = if let Some(n) = &si.nrc_back_image {
                n
            } else {
                ""
            };

            let passport_image = if let Some(pi) = &si.passport_image {
                pi
            } else {
                ""
            };

            let driving_licence_image = if let Some(dli) = &si.driving_licence_image {
                dli
            } else {
                ""
            };

            let signature_image = if let Some(si) = &si.signature_image {
                si
            } else {
                ""
            };

            let bank_code = if let Some(b) = &si.bank_code { b } else { "" };

            let bank_account = if let Some(b) = &si.bank_account {
                b
            } else {
                ""
            };

            let bank_account_image = if let Some(b) = &si.bank_account_image {
                b
            } else {
                ""
            };

            let wallet_type = if let Some(w) = &si.wallet_type { w } else { "" };

            let wallet_account = if let Some(w) = &si.wallet_account {
                w
            } else {
                ""
            };

            let fee_id = if let Some(f) = si.fee_id { f } else { 0 };

            let monthly_transaction_screenshot =
                if let Some(mts) = &si.monthly_transaction_screenshot {
                    mts
                } else {
                    ""
                };

            client.execute("insert into seller_informations (user_id, company_name, professional_title, active_since_year, location, offline_trader, facebook_profile_image, shop_or_page_name, facebook_page_image, bussiness_phone, address, nrc, nrc_front_image, nrc_back_image, bank_code, bank_account, bank_account_image, wallet_type, wallet_account, fee_id, monthly_transaction_screenshot, passport_image, driving_licence_image, signature_image) values ($1, $2, $3, EXTRACT(YEAR FROM CURRENT_DATE), $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)", &[&user_id, &si.company_name, &si.professional_title, &si.location, &si.offline_trader, &facebook_profile_image, &shop_or_page_name, &facebook_page_image, &bussiness_phone, &address, &nrc, &nrc_front_image, &nrc_back_image, &bank_code, &bank_account, &bank_account_image, &wallet_type, &wallet_account, &fee_id, &monthly_transaction_screenshot, &passport_image, &driving_licence_image, &signature_image]).await?;
        }
    }

    Ok(user_id)
}

pub async fn update_user(
    user_id: i32,
    name: &str,
    old_password: &str,
    password: &str,
    email: &str,
    phone: &str,
    old_profile_image: &str,
    profile_image: &str,
    role: &str,
    account_status: &str,
    can_modify_order_status: bool,
    can_view_address: bool,
    can_view_phone: bool,
    seller_information: &Option<SellerInformationRequest>,
    request_to_agent: bool,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    if profile_image != old_profile_image {
        match fs::remove_file(old_profile_image) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
        let path = Path::new(&old_profile_image);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        match fs::remove_file(format!("{stem}_original.{extension}")) {
            Ok(_) => println!("Original file deleted successfully!"),
            Err(e) => println!("Error deleting original file: {}", e),
        };
    }

    let mut hashed_password = password.to_string();
    if password != old_password {
        hashed_password =
            hash(&password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;
    }

    // Insert the new user into the database
    client.execute(
        "update users set name = $1, password = $2, email = $3, phone = $4, profile_image = $5, role = $6, account_status = $7, can_modify_order_status = $8, can_view_address = $9, can_view_phone = $10, request_to_agent = $11 where user_id = $12 and deleted_at is null",
        &[&name, &hashed_password, &email, &phone, &profile_image, &role, &account_status, &can_modify_order_status, &can_view_address, &can_view_phone, &request_to_agent, &user_id],
    ).await?;

    if role == "agent" {
        if let Some(si) = seller_information {
            // println!("si: {:?}", si);
            let row = client
                .query_one(
                    "select count(*) as total from seller_informations where user_id = $1 and deleted_at is null",
                    &[&user_id],
                )
                .await?;
            let total: i64 = row.get("total");

            let facebook_profile_image = if let Some(fpi) = &si.facebook_profile_image {
                fpi
            } else {
                ""
            };

            let shop_or_page_name = if let Some(spn) = &si.shop_or_page_name {
                spn
            } else {
                ""
            };

            let facebook_page_image = if let Some(fpi) = &si.facebook_page_image {
                fpi
            } else {
                ""
            };

            let bussiness_phone = if let Some(bp) = &si.bussiness_phone {
                bp
            } else {
                ""
            };

            let address = if let Some(a) = &si.address { a } else { "" };

            let nrc = if let Some(n) = &si.nrc { n } else { "" };

            let nrc_front_image = if let Some(n) = &si.nrc_front_image {
                n
            } else {
                ""
            };

            let nrc_back_image = if let Some(n) = &si.nrc_back_image {
                n
            } else {
                ""
            };

            let passport_image = if let Some(pi) = &si.passport_image {
                pi
            } else {
                ""
            };

            let driving_licence_image = if let Some(dli) = &si.driving_licence_image {
                dli
            } else {
                ""
            };

            let signature_image = if let Some(si) = &si.signature_image {
                si
            } else {
                ""
            };

            let bank_code = if let Some(b) = &si.bank_code { b } else { "" };

            let bank_account = if let Some(b) = &si.bank_account {
                b
            } else {
                ""
            };

            let bank_account_image = if let Some(b) = &si.bank_account_image {
                b
            } else {
                ""
            };

            let wallet_type = if let Some(w) = &si.wallet_type { w } else { "" };

            let wallet_account = if let Some(w) = &si.wallet_account {
                w
            } else {
                ""
            };

            let fee_id = if let Some(f) = si.fee_id { f } else { 0 };

            let monthly_transaction_screenshot =
                if let Some(mts) = &si.monthly_transaction_screenshot {
                    mts
                } else {
                    ""
                };
            if total > 0 {
                client
                .execute(
                    "update seller_informations set company_name = $1, professional_title = $2, location = $3, offline_trader = $4, facebook_profile_image = $5, shop_or_page_name = $6, facebook_page_image = $7,bussiness_phone = $8, address = $9, nrc = $10, nrc_front_image = $11, nrc_back_image = $12, bank_code = $13, bank_account = $14, bank_account_image = $15, wallet_type = $16, wallet_account = $17, fee_id = $18, monthly_transaction_screenshot = $19, passport_image = $20, driving_licence_image = $21, signature_image = $22 where user_id = $23 and deleted_at is null",
                    &[
                        &si.company_name,
                        &si.professional_title,
                        &si.location,
                        &si.offline_trader,
                        &facebook_profile_image,
                        &shop_or_page_name,
                        &facebook_page_image,
                        &bussiness_phone,
                        &address,
                        &nrc,
                        &nrc_front_image,
                        &nrc_back_image,
                        &bank_code,
                        &bank_account,
                        &bank_account_image,
                        &wallet_type,
                        &wallet_account,
                        &fee_id,
                        &monthly_transaction_screenshot,
                        &passport_image,
                        &driving_licence_image,
                        &signature_image,
                        &user_id,
                    ],
                )
                .await?;
            } else {
                client.execute("insert into seller_informations (user_id, company_name, professional_title, active_since_year, location, offline_trader, facebook_profile_image, shop_or_page_name, facebook_page_image, bussiness_phone, address, nrc, nrc_front_image, nrc_back_image, bank_code, bank_account, bank_account_image, wallet_type, wallet_account, fee_id, monthly_transaction_screenshot, passport_image, driving_licence_image, signature_image) values ($1, $2, $3, EXTRACT(YEAR FROM CURRENT_DATE), $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)", &[&user_id, &si.company_name, &si.professional_title, &si.location, &si.offline_trader, &facebook_profile_image, &shop_or_page_name, &facebook_page_image, &bussiness_phone, &address, &nrc, &nrc_front_image, &nrc_back_image, &bank_code, &bank_account, &bank_account_image, &wallet_type, &wallet_account, &fee_id, &monthly_transaction_screenshot, &passport_image, &driving_licence_image, &signature_image]).await?;
            }
        }
    }
    Ok(())
}

pub async fn delete_user(
    user_id: i32,
    old_profile_image: &str,
    client: &Client,
) -> Result<(), Error> {
    match fs::remove_file(old_profile_image) {
        Ok(_) => println!("File deleted successfully!"),
        Err(e) => println!("Error deleting file: {}", e),
    };
    let path = Path::new(&old_profile_image);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    match fs::remove_file(format!("{stem}_original.{extension}")) {
        Ok(_) => println!("Original file deleted successfully!"),
        Err(e) => println!("Error deleting original file: {}", e),
    };
    client.execute(
        "update users set deleted_at = CURRENT_TIMESTAMP where user_id = $1 and deleted_at is null",
        &[&user_id],
    ).await?;
    client.execute("update seller_informations set deleted_at = CURRENT_TIMESTAMP where user_id = $1 and deleted_at is null", &[&user_id]).await?;
    Ok(())
}

pub async fn get_user(username: &str, client: &Client) -> Option<User> {
    // Here we fetch the user from the database using tokio-postgres
    // In a real-world scenario, handle errors gracefully
    let result = client
        .query_one(
            "select user_id, username, password, role, name, profile_image, email, phone, account_status, can_modify_order_status, can_view_address, can_view_phone, created_at from users where username = $1 and deleted_at is null",
            &[&username],
        )
        .await;

    match result {
        Ok(row) => Some(User {
            user_id: row.get("user_id"),
            username: row.get("username"),
            password: row.get("password"),
            role: row.get("role"),
            name: row.get("name"),
            profile_image: row.get("profile_image"),
            email: row.get("email"),
            phone: row.get("phone"),
            account_status: row.get("account_status"),
            created_at: row.get("created_at"),
            can_modify_order_status: row.get("can_modify_order_status"),
            can_view_address: row.get("can_view_address"),
            can_view_phone: row.get("can_view_phone"),
            seller_information: None,
        }),
        Err(_) => None,
    }
}

pub async fn get_user_by_google_id(google_id: &str, client: &Client) -> Option<User> {
    // Here we fetch the user from the database using tokio-postgres
    // In a real-world scenario, handle errors gracefully
    let result = client
        .query_one(
            "select user_id, username, password, role, name, profile_image, email, phone, account_status, can_modify_order_status, can_view_address, can_view_phone, created_at from users where google_id = $1 and deleted_at is null",
            &[&google_id],
        )
        .await;

    match result {
        Ok(row) => Some(User {
            user_id: row.get("user_id"),
            username: row.get("username"),
            password: row.get("password"),
            role: row.get("role"),
            name: row.get("name"),
            profile_image: row.get("profile_image"),
            email: row.get("email"),
            phone: row.get("phone"),
            account_status: row.get("account_status"),
            created_at: row.get("created_at"),
            can_modify_order_status: row.get("can_modify_order_status"),
            can_view_address: row.get("can_view_address"),
            can_view_phone: row.get("can_view_phone"),
            seller_information: None,
        }),
        Err(_) => None,
    }
}

pub async fn get_users(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &Option<String>,
    account_status: &Option<String>,
    request_to_agent: Option<bool>,
    client: &Client,
) -> Result<PaginationResult<User>, Error> {
    let mut base_query = "from users where deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    if let Some(r) = role {
        params.push(Box::new(r));
        base_query = format!("{base_query} and role = ${}", params.len());
    }
    if let Some(status) = account_status {
        params.push(Box::new(status));
        base_query = format!("{base_query} and account_status = ${}", params.len());
    }
    if let Some(rta) = request_to_agent {
        params.push(Box::new(rta));
        base_query = format!("{base_query} and request_to_agent = ${}", params.len());
    }

    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "user_id, name, username, password, role, email, phone, profile_image, account_status, can_modify_order_status, can_view_address, can_view_phone, created_at",
        base_query: &base_query,
        search_columns: vec![
            "name",
            "username",
            "role",
            "email",
            "phone",
            "account_status",
        ],
        search: search.as_deref(),
        order_options: Some("created_at desc"),
        page,
        per_page,
    });

    let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();

    let row = client.query_one(&result.count_query, &params_slice).await?;
    let total: i64 = row.get("total");

    let mut page_counts = 0;
    let mut current_page = 0;
    let mut limit = 0;
    if page.is_some() && per_page.is_some() {
        current_page = page.unwrap();
        limit = per_page.unwrap();
        page_counts = (total as f64 / limit as f64).ceil() as usize;
    }

    let users = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| User {
            user_id: row.get("user_id"),
            username: row.get("username"),
            password: row.get("password"),
            role: row.get("role"),
            name: row.get("name"),
            profile_image: row.get("profile_image"),
            email: row.get("email"),
            phone: row.get("phone"),
            account_status: row.get("account_status"),
            can_modify_order_status: row.get("can_modify_order_status"),
            can_view_address: row.get("can_view_address"),
            can_view_phone: row.get("can_view_phone"),
            created_at: row.get("created_at"),
            seller_information: None,
        })
        .collect();

    Ok(PaginationResult {
        data: users,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn change_password(
    user_id: i32,
    new_password: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let hashed_password =
        hash(new_password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;
    client
        .execute(
            "update users set password = $1 where user_id = $2",
            &[&hashed_password, &user_id],
        )
        .await?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub profile_image: String,
}

pub async fn get_user_profile(user_id: i32, client: &Client) -> Option<UserProfile> {
    let result = client.query_one("select name, email, phone, profile_image from users where deleted_at is null and user_id = $1", &[&user_id]).await;
    match result {
        Ok(row) => Some(UserProfile {
            name: row.get("name"),
            email: row.get("email"),
            phone: row.get("phone"),
            profile_image: row.get("profile_image"),
        }),
        Err(_) => None,
    }
}

pub async fn update_user_profile(
    user_id: i32,
    data: &UserProfile,
    old_profile_image: &str,
    client: &Client,
) -> Result<(), Error> {
    client.execute("update users set name = $1, email = $2, phone = $3, profile_image = $4 where user_id = $5", &[&data.name, &data.email, &data.phone, &data.profile_image, &user_id]).await?;
    if data.profile_image != old_profile_image {
        match fs::remove_file(old_profile_image) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
    }
    Ok(())
}

pub async fn get_admin_user_ids(client: &Client) -> Result<Vec<i32>, Error> {
    Ok(client
        .query(
            "select user_id from users where role = 'admin' and deleted_at is null",
            &[],
        )
        .await?
        .iter()
        .map(|row| row.get("user_id"))
        .collect())
}

pub async fn get_user_name(user_id: i32, client: &Client) -> Option<String> {
    match client
        .query_one(
            "select name from users where user_id = $1 and deleted_at is null",
            &[&user_id],
        )
        .await
    {
        Ok(row) => Some(row.get("name")),
        Err(_) => None,
    }
}

pub async fn get_profile_images(client: &Client) -> Vec<String> {
    match client.query("select profile_image from users", &[]).await {
        Ok(rows) => rows.iter().map(|row| row.get("profile_image")).collect(),
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    }
}

pub async fn can_modify_order_status(user_id: i32, client: &Client) -> bool {
    match client
        .query_one(
            "select can_modify_order_status from users where user_id = $1 and deleted_at is null",
            &[&user_id],
        )
        .await
    {
        Ok(row) => row.get("can_modify_order_status"),
        Err(err) => {
            println!("{:?}", err);
            false
        }
    }
}

pub async fn is_phone_existed(phone: &str, client: &Client) -> bool {
    let total: i64=  match client.query_one(
        "select count(*) as total from users where phone = $1 and deleted_at is null and account_status = 'active'",
        &[&phone],
    ).await {
        Ok(row) => row.get("total"),
        Err(err) => {
            println!("{:?}", err);
            0
        }
    };
    total > 0
}

// pub async fn user_exists_by(vendor: &str, id: &str, client: &Client) -> Result<bool, Error> {
//     let column = if vendor == "google" {
//         "google_id"
//     } else if vendor == "facebook" {
//         "facebook_id"
//     } else {
//         "apple_id"
//     };
//     let query = format!("select username from users where {column} = $1 and deleted_at is null");
//     let row = client.query_one(&query, &[&id]).await;
//     // Return whether the user exists
//     Ok(row.is_ok())
// }

#[derive(Serialize)]
pub struct UserPermission {
    pub can_modify_order_status: bool,
    pub can_view_address: bool,
    pub can_view_phone: bool,
}

pub async fn get_user_permission(user_id: i32, client: &Client) -> UserPermission {
    match client.query_one("select can_modify_order_status, can_view_address, can_view_phone from users where user_id = $1 and deleted_at is null", &[&user_id]).await {
        Ok(row) => UserPermission {
            can_modify_order_status: row.get("can_modify_order_status"),
            can_view_address: row.get("can_view_address"),
            can_view_phone: row.get("can_view_phone")
        }, Err(err) => {
            println!("{:?}", err);
            UserPermission {
                can_modify_order_status: false,
                can_view_address: false,
                can_view_phone: false
            }
        }
    }
}
