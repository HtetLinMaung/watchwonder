use std::fs;

use bcrypt::{hash, DEFAULT_COST};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

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
            "select user_id, username, password, role, name, profile_image, email, phone, account_status, created_at from users where user_id = $1 and deleted_at is null",
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
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let hashed_password =
        hash(&password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;

    // Insert the new user into the database
    client.execute(
        "INSERT INTO users (name, username, password, email, phone, profile_image, role, account_status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[&name, &username, &hashed_password, &email, &phone, &profile_image, &role, &account_status],
    ).await?;
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
        "update users set name = $1, password = $2, email = $3, phone = $4, profile_image = $5, role = $6 where user_id = $7 and deleted_at is null",
        &[&name, &hashed_password, &email, &phone, &profile_image, &role, &user_id],
    ).await?;
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
