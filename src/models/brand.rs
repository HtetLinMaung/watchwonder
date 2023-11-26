use std::{fs, path::Path};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    setting::{get_demo_platform, get_demo_user_id, get_min_demo_version},
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Brand {
    pub brand_id: i32,
    pub name: String,
    pub description: String,
    pub logo_url: String,
    pub created_at: Option<NaiveDateTime>,
}

pub async fn get_brands(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    platform: &str,
    user_id: i32,
    version: i32,
    client: &Client,
) -> Result<PaginationResult<Brand>, Error> {
    let mut base_query = "from brands where deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    // let order_options = match role {
    //     "user" => "name asc, created_at desc",
    //     _ => "created_at desc",
    // };
    let order_options = "name asc";

    let demo_user_id = get_demo_user_id();
    let min_demo_version = get_min_demo_version();
    if platform == get_demo_platform().as_str()
        || platform == "ios" && version >= min_demo_version
        || (demo_user_id > 0 && user_id == demo_user_id)
    {
        base_query = format!("{base_query} and is_demo = true");
    } else {
        base_query = format!("{base_query} and is_demo = false");
    }

    // if role == "agent" {
    //     params.push(Box::new(user_id));
    //     base_query = format!("{base_query} and creator_id = ${}", params.len());
    // }

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "brand_id, name, description, logo_url, created_at",
        base_query: &base_query,
        search_columns: vec!["name", "description"],
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

    let brands: Vec<Brand> = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| Brand {
            brand_id: row.get("brand_id"),
            name: row.get("name"),
            description: row.get("description"),
            logo_url: row.get("logo_url"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: brands,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn add_brand(
    name: &str,
    description: &str,
    logo_url: &str,
    creator_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    // Insert the new brands into the database
    client
        .execute(
            "INSERT INTO brands (name, description, logo_url, creator_id) VALUES ($1, $2, $3, $4)",
            &[&name, &description, &logo_url, &creator_id],
        )
        .await?;
    Ok(())
}

pub async fn update_brand(
    brand_id: i32,
    name: &str,
    description: &str,
    logo_url: &str,
    old_logo_url: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client.execute(
            "update brands set name = $1, description = $2, logo_url = $3 where brand_id = $4 and deleted_at is null",
            &[&name, &description, &logo_url ,&brand_id],
        ).await?;
    if logo_url != old_logo_url {
        match fs::remove_file(old_logo_url) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
        let path = Path::new(&old_logo_url);
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

pub async fn get_brand_by_id(brand_id: i32, client: &Client) -> Option<Brand> {
    let result = client
        .query_one(
            "select brand_id, name, description, logo_url, created_at from brands where brand_id = $1 and deleted_at is null",
            &[&brand_id],
        )
        .await;

    match result {
        Ok(row) => Some(Brand {
            brand_id: row.get("brand_id"),
            name: row.get("name"),
            description: row.get("description"),
            logo_url: row.get("logo_url"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn delete_brand(
    brand_id: i32,
    old_logo_url: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client.execute(
        "update brands set deleted_at = CURRENT_TIMESTAMP where brand_id = $1 and deleted_at is null",
        &[&brand_id],
    ).await?;
    match fs::remove_file(old_logo_url) {
        Ok(_) => println!("File deleted successfully!"),
        Err(e) => println!("Error deleting file: {}", e),
    };
    let path = Path::new(&old_logo_url);
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

pub async fn get_logo_urls(client: &Client) -> Vec<String> {
    match client.query("select logo_url from brands", &[]).await {
        Ok(rows) => rows.iter().map(|row| row.get("logo_url")).collect(),
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    }
}
