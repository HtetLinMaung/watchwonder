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
    pub created_at: NaiveDateTime,
}

pub async fn get_categories(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Category>, Error> {
    let base_query = "from categories where deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    let order_options = match role {
        "user" => "name asc, created_at desc",
        "admin" => "created_at desc",
        _ => "",
    };
    let result = generate_pagination_query(PaginationOptions {
        select_columns: "category_id, name, description, cover_image, created_at",
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
