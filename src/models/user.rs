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
            "select user_id, username, password, role, name, profile_image, email, phone, created_at from users where user_id = $1 and deleted_at is null",
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
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let hashed_password =
        hash(&password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;

    // Insert the new user into the database
    client.execute(
        "INSERT INTO users (name, username, password, email, phone, profile_image, role) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        &[&name, &username, &hashed_password, &email, &phone, &profile_image, &role],
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
            "select user_id, username, password, role, name, profile_image, email, phone, created_at from users where username = $1 and deleted_at is null",
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
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn get_users(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    client: &Client,
) -> Result<PaginationResult<User>, Error> {
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "user_id, name, username, password, role, email, phone, profile_image, created_at",
        base_query: "from users where deleted_at is null",
        search_columns: vec!["name", "username", "role", "email", "phone"],
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
