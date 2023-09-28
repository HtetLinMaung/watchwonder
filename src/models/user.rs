use bcrypt::{hash, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: i32,
    pub username: String,
    pub password: String,
    pub role_name: String,
    pub name: String,
    pub profile_image: String,
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

pub struct NewUser {
    pub name: String,
    pub username: String,
    pub password: String,
    pub email: String,
    pub phone: String,
    pub profile_image: String,
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
            "select u.user_id, u.username, u.password, r.role_name, u.name, u.profile_image from users u inner join roles r on r.role_id = u.role_id where username = $1 and u.deleted_at is null and r.deleted_at is null",
            &[&username],
        )
        .await;

    match result {
        Ok(row) => Some(User {
            user_id: row.get("user_id"),
            username: row.get("username"),
            password: row.get("password"),
            role_name: row.get("role_name"),
            name: row.get("name"),
            profile_image: row.get("profile_image"),
        }),
        Err(_) => None,
    }
}
