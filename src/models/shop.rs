use std::{fs, path::Path};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client};

use crate::utils::{
    common_struct::PaginationResult,
    setting::{get_demo_platform, get_demo_user_id},
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
    pub creator_id: i32,
    pub created_at: NaiveDateTime,
}

pub async fn get_shops(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    platform: &str,
    status: &Option<String>,
    view: &Option<String>,
    role: &str,
    user_id: i32,
    client: &Client,
) -> Result<PaginationResult<Shop>, Box<dyn std::error::Error>> {
    let mut base_query = "from shops where deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let mut screen_view = "admin";
    if let Some(v) = view {
        screen_view = v.as_str();
    }

    let order_options = if role == "user" || (role == "agent" && screen_view == "user") {
        "name asc, created_at desc"
    } else {
        "created_at desc"
    };

    if role == "agent" {
        params.push(Box::new(user_id));
        if screen_view == "user" {
            base_query = format!(
                "{base_query} and creator_id != ${} and status != 'Pending Approval'",
                params.len()
            );
        } else {
            base_query = format!("{base_query} and creator_id = ${}", params.len());
        }
    } else if role == "admin" {
        if let Some(s) = status {
            params.push(Box::new(s));
            base_query = format!("{base_query} and status = ${}", params.len());
        }
    } else {
        base_query = format!("{base_query} and status != 'Pending Approval'");
    }

    let demo_user_id = get_demo_user_id();
    if platform == get_demo_platform().as_str() || (demo_user_id > 0 && user_id == demo_user_id) {
        base_query = format!("{base_query} and is_demo = true");
    } else {
        base_query = format!("{base_query} and is_demo = false");
    }

    let result=  generate_pagination_query(PaginationOptions {
        select_columns: "shop_id, name, description, cover_image, address, city, state, postal_code, country, phone, email, website_url, operating_hours, status, creator_id, created_at",
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
    // let shops: Vec<Shop> = client
    // .query(&result.query, &params_slice[..])
    // .await?
    // .iter()
    // .map(|row| {})
    // .collect();

    let mut shops: Vec<Shop> = vec![];
    for row in client.query(&result.query, &params_slice[..]).await? {
        let shop_id: i32 = row.get("shop_id");
        shops.push(Shop {
            shop_id,
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
            creator_id: row.get("creator_id"),
            created_at: row.get("created_at"),
        });
    }

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
    creator_id: i32,
    role: &str,
    client: &Client,
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut status = data.status.as_str();
    if role == "agent" {
        status = "Pending Approval";
    }
    let row= client
        .query_one(
            "insert into shops (name, description, cover_image, address, city, state, postal_code, country, phone, email, website_url, operating_hours, status, creator_id) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) returning shop_id",
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
                &status,
                &creator_id
            ],
        )
        .await?;
    Ok(row.get("shop_id"))
}

pub async fn get_shop_by_id(shop_id: i32, client: &Client) -> Option<Shop> {
    let result = client
        .query_one(
            "select shop_id, name, description, cover_image, address, city, state, postal_code, country, phone, email, website_url, operating_hours, status, creator_id, created_at from shops where deleted_at is null and shop_id = $1",
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
            creator_id: row.get("creator_id"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_shop(
    shop_id: i32,
    old_cover_image: &str,
    data: &ShopRequest,
    role: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut status = data.status.as_str();
    if role == "agent" {
        status = "Pending Approval";
    }
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
                &status,
                &shop_id,
            ],
        )
        .await?;
    if old_cover_image != &data.cover_image {
        match fs::remove_file(old_cover_image) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
        let path = Path::new(&old_cover_image);
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
    let path = Path::new(&old_cover_image);
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
    Ok(())
}

pub async fn get_cover_images(client: &Client) -> Vec<String> {
    match client.query("select cover_image from shops", &[]).await {
        Ok(rows) => rows.iter().map(|row| row.get("cover_image")).collect(),
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    }
}

pub async fn get_creator_id_from_shop(shop_id: i32, client: &Client) -> Option<i32> {
    match client
        .query_one(
            "select creator_id from shops where shop_id = $1",
            &[&shop_id],
        )
        .await
    {
        Ok(row) => Some(row.get("creator_id")),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}
