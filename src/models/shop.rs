use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

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
}

pub struct GetShopsResult {
    pub shops: Vec<Shop>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub page_counts: usize,
}

pub async fn get_shops(
    search: &Option<String>,
    page: &Option<u32>,
    per_page: &Option<u32>,
    client: &Client,
) -> Result<GetShopsResult, Error> {
    let mut query =
        "select shop_id, name, description, cover_image, address, city, state, postal_code, country, phone, email, website_url, operating_hours, status from shops where deleted_at is null".to_string();
    let mut count_sql =
        String::from("select count(*) as total from shops where deleted_at is null");
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    if let Some(s) = search {
        query = format!(
            "{query} and (name like '%{s}%' or description like '%{s}%' or address like '%{s}%' or city like '%{s}%' or state like '%{s}%' or postal_code like '%{s}%' or country like '%{s}%' or phone like '%{s}%' or operating_hours like '%{s}%' or status like '%{s}%')"
        );
        count_sql = format!("{count_sql} and (name like '%{s}%' or description like '%{s}%' or address like '%{s}%' or city like '%{s}%' or state like '%{s}%' or postal_code like '%{s}%' or country like '%{s}%' or phone like '%{s}%' or operating_hours like '%{s}%' or status like '%{s}%')");
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

    let shops: Vec<Shop> = client
        .query(&query, &params_slice[..])
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
        })
        .collect();

    Ok(GetShopsResult {
        shops,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}
