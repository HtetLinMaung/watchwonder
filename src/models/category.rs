use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub category_id: i32,
    pub name: String,
    pub description: String,
    pub cover_image: String,
}

pub struct GetCategoriesResult {
    pub categories: Vec<Category>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub page_counts: usize,
}

pub async fn get_categories(
    search: &Option<String>,
    page: &Option<u32>,
    per_page: &Option<u32>,
    client: &Client,
) -> Result<GetCategoriesResult, Error> {
    let mut query =
        "select category_id, name, description, cover_image from categories where deleted_at is null"
            .to_string();
    let mut count_sql =
        String::from("select count(*) as total from categories where deleted_at is null");
    if let Some(s) = search {
        query = format!("{query} and (name like '%{s}%' or description like '%{s}%')");
        count_sql = format!("{count_sql} and (name like '%{s}%' or description like '%{s}%')");
    }

    query = format!("{query} order by name asc, created_at desc");

    let mut current_page = 0;
    let mut limit = 0;
    let mut page_counts = 0;
    let row = client.query_one(&count_sql, &[]).await?;
    let total: i64 = row.get("total");
    if page.is_some() && per_page.is_some() {
        current_page = page.unwrap();
        limit = per_page.unwrap();
        let offset = (current_page - 1) * limit;
        query = format!("{query} limit {limit} offset {offset}");
        page_counts = (total as f64 / f64::from(limit)).ceil() as usize;
    }

    let categories: Vec<Category> = client
        .query(&query, &[])
        .await?
        .iter()
        .map(|row| Category {
            category_id: row.get("category_id"),
            name: row.get("name"),
            description: row.get("description"),
            cover_image: row.get("cover_image"),
        })
        .collect();

    Ok(GetCategoriesResult {
        categories,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}
