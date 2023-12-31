use std::{fs, path::Path};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub category_id: i32,
    pub name: String,
    pub description: String,
    pub cover_image: String,
    pub level: i32,
    pub created_at: NaiveDateTime,
}

pub async fn get_categories(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    client: &Client,
) -> Result<PaginationResult<Category>, Error> {
    let base_query = "from categories where deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    // let order_options = match role {
    //     "user" => "name asc, created_at desc",
    //     _ => "created_at desc",
    // };
    let order_options = "level desc, name";

    // if role == "agent" {
    //     params.push(Box::new(user_id));
    //     base_query = format!("{base_query} and creator_id = ${}", params.len());
    // }

    // let demo_user_id = get_demo_user_id().await;
    // if demo_user_id > 0 && user_id == demo_user_id {
    //     base_query = format!("{base_query} and is_demo = true");
    // } else {
    //     base_query = format!("{base_query} and is_demo = false");
    // }

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "category_id, name, description, cover_image, level, created_at",
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

    let categories: Vec<Category> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| Category {
            category_id: row.get("category_id"),
            name: row.get("name"),
            description: row.get("description"),
            cover_image: row.get("cover_image"),
            level: row.get("level"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: categories,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct CategoryRequest {
    pub name: String,
    pub description: String,
    pub cover_image: String,
    pub level: Option<i32>,
}

pub async fn add_category(
    data: &CategoryRequest,
    creator_id: i32,
    role: &str,
    client: &Client,
) -> Result<(), Error> {
    let level = if let Some(l) = data.level {
        if role == "admin" {
            l
        } else {
            0
        }
    } else {
        0
    };
    client
        .execute(
            "insert into categories (name, description, cover_image, creator_id, level) values ($1, $2, $3, $4, $5)",
            &[&data.name, &data.description, &data.cover_image, &creator_id, &level],
        )
        .await?;
    Ok(())
}

pub async fn get_category_by_id(category_id: i32, client: &Client) -> Option<Category> {
    let result = client
        .query_one(
            "select category_id, name, description, cover_image, level, created_at from categories where deleted_at is null and category_id = $1",
            &[&category_id],
        )
        .await;

    match result {
        Ok(row) => Some(Category {
            category_id: row.get("category_id"),
            name: row.get("name"),
            description: row.get("description"),
            cover_image: row.get("cover_image"),
            level: row.get("level"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_category(
    category_id: i32,
    old_cover_image: &str,
    data: &CategoryRequest,
    old_level: i32,
    role: &str,
    client: &Client,
) -> Result<(), Error> {
    let level = if let Some(l) = data.level {
        if role == "admin" {
            l
        } else {
            old_level
        }
    } else {
        old_level
    };
    client
        .execute(
            "update categories set name = $1, description = $2, cover_image = $3, level = $4 where category_id = $5",
            &[
                &data.name,
                &data.description,
                &data.cover_image,
                &level,
                &category_id,
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

pub async fn delete_category(
    category_id: i32,
    old_cover_image: &str,
    client: &Client,
) -> Result<(), Error> {
    client
        .execute(
            "update categories set deleted_at = CURRENT_TIMESTAMP where category_id = $1",
            &[&category_id],
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
    match client
        .query("select cover_image from categories", &[])
        .await
    {
        Ok(rows) => rows.iter().map(|row| row.get("cover_image")).collect(),
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    }
}
