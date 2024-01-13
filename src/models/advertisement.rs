use std::{fs, path::Path};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Deserialize)]
pub struct AdvertisementRequest {
    pub media_type: String,
    pub media_url: String,
    pub level: i32,
}

pub async fn add_advertisement(data: &AdvertisementRequest, client: &Client) -> Result<i32, Error> {
    let row= client
        .query_one(
            "insert into advertisements (media_type, media_url) values ($1, $2) returning advertisement_id",
            &[&data.media_type, &data.media_url],
        )
        .await?;
    let advertisement_id = row.get("advertisement_id");
    Ok(advertisement_id)
}

#[derive(Serialize)]
pub struct Advertisement {
    pub advertisement_id: i32,
    pub media_type: String,
    pub media_url: String,
    pub level: i32,
    pub created_at: NaiveDateTime,
}

pub async fn get_advertisements(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    client: &Client,
) -> Result<PaginationResult<Advertisement>, Error> {
    let base_query = "from advertisements where deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = "level desc";

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "advertisement_id, media_type, media_url, level, created_at",
        base_query: &base_query,
        search_columns: vec!["advertisement_id::text"],
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

    let advertisements: Vec<Advertisement> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| Advertisement {
            advertisement_id: row.get("advertisement_id"),
            media_type: row.get("media_type"),
            media_url: row.get("media_url"),
            level: row.get("level"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: advertisements,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_advertisement_by_id(
    advertisement_id: i32,
    client: &Client,
) -> Option<Advertisement> {
    match client
        .query_one(
            "select advertisement_id, media_type, media_url, level, created_at from advertisements where advertisement_id = $1 and deleted_at is null",
            &[&advertisement_id],
        )
        .await
    {
        Ok(row) => Some(Advertisement {
            advertisement_id: row.get("advertisement_id"),
            media_type: row.get("media_type"),
            media_url: row.get("media_url"),
            level: row.get("level"),
            created_at: row.get("created_at"),
        }),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}

pub async fn update_advertisement(
    advertisement_id: i32,
    data: &AdvertisementRequest,
    old_media_url: &str,
    client: &Client,
) -> Result<(), Error> {
    client.execute("update advertisements set media_type = $1, media_url = $2, level = $3 where advertisement_id = $4", &[&data.media_type, &data.media_url, &data.level, &advertisement_id]).await?;
    if old_media_url != &data.media_url {
        match fs::remove_file(old_media_url.replace("/images", "./images")) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
        let path = Path::new(&old_media_url);
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

pub async fn delete_advertisement(
    advertisement_id: i32,
    old_media_url: &str,
    client: &Client,
) -> Result<(), Error> {
    client
        .execute(
            "update advertisements set deleted_at = CURRENT_TIMESTAMP where advertisement_id = $1",
            &[&advertisement_id],
        )
        .await?;
    match fs::remove_file(old_media_url.replace("/images", "./images")) {
        Ok(_) => println!("File deleted successfully!"),
        Err(e) => println!("Error deleting file: {}", e),
    };
    let path = Path::new(&old_media_url);
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


