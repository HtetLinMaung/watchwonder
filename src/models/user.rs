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

pub async fn create_user(
    name: &str,
    username: &str,
    password: &str,
    email: &str,
    phone: &str,
    profile_image: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let hashed_password =
        hash(&password, DEFAULT_COST).map_err(|e| format!("Failed to hash password: {}", e))?;

    // Insert the new user into the database
    client.execute(
        "INSERT INTO users (name, username, password, email, phone, profile_image) VALUES ($1, $2, $3, $4, $5, $6)",
        &[&name, &username, &hashed_password, &email, &phone, &profile_image],
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
