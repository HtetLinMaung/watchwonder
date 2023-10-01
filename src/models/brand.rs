use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Brand {
    pub brand_id: i32,
    pub name: String,
    pub description: String,
    pub logo_url: String,
}

pub struct GetBrandsResult {
    pub brands: Vec<Brand>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub page_counts: usize,
}

pub async fn get_brands(
    search: &Option<String>,
    page: &Option<u32>,
    per_page: &Option<u32>,
    client: &Client,
) -> Result<GetBrandsResult, Error> {
    let mut query =
        "select brand_id, name, description, logo_url from brands where deleted_at is null"
            .to_string();
    let mut count_sql =
        String::from("select count(*) as total from brands where deleted_at is null");
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    if let Some(s) = search {
        query = format!("{query} and (name like '%{s}%' or description like '%{s}%')");
        count_sql = format!("{count_sql} and (name like '%{s}%' or description like '%{s}%')");
    }

    query = format!("{query} order by name asc, created_at desc");

    let mut current_page = 0;
    let mut limit = 0;
    let mut page_counts = 0;
    let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();
    let row = client.query_one(&count_sql, &params_slice).await?;
    let total: i64 = row.get("total");
    if page.is_some() && per_page.is_some() {
        current_page = page.unwrap();
        limit = per_page.unwrap();
        let offset = (current_page - 1) * limit;
        query = format!("{query} limit {limit} offset {offset}");
        page_counts = (total as f64 / f64::from(limit)).ceil() as usize;
    }

    let brands: Vec<Brand> = client
        .query(&query, &params_slice[..])
        .await?
        .iter()
        .map(|row| Brand {
            brand_id: row.get("brand_id"),
            name: row.get("name"),
            description: row.get("description"),
            logo_url: row.get("logo_url"),
        })
        .collect();

    Ok(GetBrandsResult {
        brands,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}
