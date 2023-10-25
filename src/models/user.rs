use std::fs;

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
            "select u.user_id, u.username, u.password, u.role, u.name, u.profile_image, u.email, u.phone, u.account_status, u.created_at, coalesce(si.company_name, '') as company_name, coalesce(si.professional_title, '') as professional_title, coalesce(si.active_since_year, 0) as active_since_year, coalesce(si.location, '') as location, coalesce(si.offline_trader, false) as offline_trader from users u left join seller_informations si on u.user_id = si.user_id and si.deleted_at is null where u.user_id = $1 and u.deleted_at is null",
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
            created_at: row.get("created_at"),
            seller_information: Some(SellerInformation {
                company_name: row.get("company_name"),
                professional_title: row.get("professional_title"),
                active_since_year: row.get("active_since_year"),
                location: row.get("location"),
                offline_trader: row.get("offline_trader"),
            }),
        }),
        Err(_) => None,
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
    seller_information: &Option<SellerInformationRequest>,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let hashed_password =
        hash(&password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;

    // Insert the new user into the database
    let row =client.query_one(
        "INSERT INTO users (name, username, password, email, phone, profile_image, role, account_status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) returning user_id",
        &[&name, &username, &hashed_password, &email, &phone, &profile_image, &role, &account_status],
    ).await?;

    if account_status == "active" && role == "agent" {
        let user_id: i32 = row.get("user_id");
        if let Some(si) = seller_information {
            client.execute("insert into seller_informations (user_id, company_name, professional_title, active_since_year, location, offline_trader) values ($1, $2, $3, EXTRACT(YEAR FROM CURRENT_DATE), $4, $5)", &[&user_id, &si.company_name, &si.professional_title, &si.location, &si.offline_trader]).await?;
        }
    }

    Ok(())
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
    seller_information: &Option<SellerInformationRequest>,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    if profile_image != old_profile_image {
        match fs::remove_file(old_profile_image) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
    }

    let mut hashed_password = password.to_string();
    if password != old_password {
        hashed_password =
            hash(&password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;
    }

    // Insert the new user into the database
    client.execute(
        "update users set name = $1, password = $2, email = $3, phone = $4, profile_image = $5, role = $6, account_status = $7 where user_id = $8 and deleted_at is null",
        &[&name, &hashed_password, &email, &phone, &profile_image, &role, &account_status, &user_id],
    ).await?;

    if account_status == "active" && role == "agent" {
        if let Some(si) = seller_information {
            println!("si: {:?}", si);
            client
                .execute(
                    "update seller_informations set company_name = $1, professional_title = $2, location = $3, offline_trader = $4 where user_id = $5 and deleted_at is null",
                    &[
                        &si.company_name,
                        &si.professional_title,
                        &si.location,
                        &si.offline_trader,
                        &user_id,
                    ],
                )
                .await?;
        }
    }
    Ok(())
}

pub async fn delete_user(
    user_id: i32,
    old_profile_image: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    match fs::remove_file(old_profile_image) {
        Ok(_) => println!("File deleted successfully!"),
        Err(e) => println!("Error deleting file: {}", e),
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
            "select user_id, username, password, role, name, profile_image, email, phone, account_status, created_at from users where username = $1 and deleted_at is null",
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

    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "user_id, name, username, password, role, email, phone, profile_image, account_status, created_at",
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
) -> Result<(), Box<dyn std::error::Error>> {
    client.execute("update users set name = $1, email = $2, phone = $3, profile_image = $4 where user_id = $5", &[&data.name, &data.email, &data.phone, &data.profile_image, &user_id]).await?;
    if data.profile_image != old_profile_image {
        match fs::remove_file(old_profile_image) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
    }
    Ok(())
}

pub async fn get_admin_user_ids(client: &Client) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
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
