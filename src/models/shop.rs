use std::fs;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Shop {
    pub shop_id: i32,
    pub name: String,
    pub description: String,
    pub cover_image: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub phone: String,
    pub email: String,
    pub website_url: String,
    pub operating_hours: String,
    pub status: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_shops(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    user_id: i32,
    client: &Client,
) -> Result<PaginationResult<Shop>, Error> {
    let mut base_query = "from shops where deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];
    let order_options = match role {
        "user" => "name asc, created_at desc",
        _ => "created_at desc",
    };

    if role == "agent" {
        params.push(Box::new(user_id));
        base_query = format!("{base_query} and creator_id = ${}", params.len());
    }

    let result=  generate_pagination_query(PaginationOptions {
        select_columns: "shop_id, name, description, cover_image, address, city, state, postal_code, country, phone, email, website_url, operating_hours, status, created_at",
        base_query: &base_query,
        search_columns: vec![ "name", "description", "address", "city", "state", "postal_code", "country", "phone", "email", "operating_hours", "status"],
        search: search.as_deref(),
        order_options: Some(&order_options),
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

    let shops: Vec<Shop> = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| Shop {
            shop_id: row.get("shop_id"),
            name: row.get("name"),
            description: row.get("description"),
            cover_image: row.get("cover_image"),
            address: row.get("address"),
            city: row.get("city"),
            state: row.get("state"),
            postal_code: row.get("postal_code"),
            country: row.get("country"),
            phone: row.get("phone"),
            email: row.get("email"),
            website_url: row.get("website_url"),
            operating_hours: row.get("operating_hours"),
            status: row.get("status"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: shops,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct ShopRequest {
    pub name: String,
    pub description: String,
    pub cover_image: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub phone: String,
    pub email: String,
    pub website_url: String,
    pub operating_hours: String,
    pub status: String,
}

pub async fn add_shop(
    data: &ShopRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into shops (name, description, cover_image, address, city, state, postal_code, country, phone, email, website_url, operating_hours, status) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            &[
                &data.name,
                &data.description,
                &data.cover_image,
                &data.address,
                &data.city,
                &data.state,
                &data.postal_code,
                &data.country,
                &data.phone,
                &data.email,
                &data.website_url,
                &data.operating_hours,
                &data.status,
            ],
        )
        .await?;
    Ok(())
}

pub async fn get_shop_by_id(shop_id: i32, client: &Client) -> Option<Shop> {
    let result = client
        .query_one(
            "select shop_id, name, description, cover_image, address, city, state, postal_code, country, phone, email, website_url, operating_hours, status, created_at from shops where deleted_at is null and shop_id = $1",
            &[&shop_id],
        )
        .await;

    match result {
        Ok(row) => Some(Shop {
            shop_id: row.get("shop_id"),
            name: row.get("name"),
            description: row.get("description"),
            cover_image: row.get("cover_image"),
            address: row.get("address"),
            city: row.get("city"),
            state: row.get("state"),
            postal_code: row.get("postal_code"),
            country: row.get("country"),
            phone: row.get("phone"),
            email: row.get("email"),
            website_url: row.get("website_url"),
            operating_hours: row.get("operating_hours"),
            status: row.get("status"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_shop(
    shop_id: i32,
    old_cover_image: &str,
    data: &ShopRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update shops set name = $1, description = $2, cover_image = $3, address = $4, city = $5, state = $6, postal_code = $7, country = $8, phone = $9, email = $10, website_url = $11, operating_hours = $12, status = $13 where shop_id = $14",
            &[
                &data.name,
                &data.description,
                &data.cover_image,
                &data.address,
                &data.city,
                &data.state,
                &data.postal_code,
                &data.country,
                &data.phone,
                &data.email,
                &data.website_url,
                &data.operating_hours,
                &data.status,
                &shop_id,
            ],
        )
        .await?;
    if old_cover_image != &data.cover_image {
        match fs::remove_file(old_cover_image) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
    }

    Ok(())
}

pub async fn delete_shop(
    shop_id: i32,
    old_cover_image: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update shops set deleted_at = CURRENT_TIMESTAMP where shop_id = $1",
            &[&shop_id],
        )
        .await?;
    match fs::remove_file(old_cover_image) {
        Ok(_) => println!("File deleted successfully!"),
        Err(e) => println!("Error deleting file: {}", e),
    };
    Ok(())
}
