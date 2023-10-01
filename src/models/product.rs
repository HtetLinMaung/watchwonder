use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::sql::{generate_pagination_query, PaginationOptions};

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub product_id: i32,
    pub model: String,
    pub description: String,
    pub color: String,
    pub strap_material: String,
    pub strap_color: String,
    pub case_material: String,
    pub dial_color: String,
    pub movement_type: String,
    pub water_resistance: String,
    pub warranty_period: String,
    pub dimensions: String,
    pub price: f64,
    pub stock_quantity: i32,
    pub is_top_model: bool,
    pub product_images: Vec<String>,
    pub shop_name: String,
    pub category_name: String,
    pub brand_name: String,
}

pub struct GetProductsResult {
    pub products: Vec<Product>,
    pub total: i64,
    pub page: usize,
    pub per_page: usize,
    pub page_counts: usize,
}

pub async fn get_products(
    search: Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: String,
    client: &Client,
) -> Result<GetProductsResult, Error> {
    let base_query = "from products p inner join brands b on b.brand_id = p.brand_id inner join categories c on p.category_id = c.category_id inner join shops s on s.shop_id = p.shop_id where p.deleted_at is null and b.deleted_at is null and c.deleted_at is null and s.deleted_at is null".to_string();
    let order_options = match role.as_str() {
        "user" => "model asc, created_at desc".to_string(),
        "admin" => "created_at desc".to_string(),
        _ => "".to_string(),
    };

    let result=  generate_pagination_query(PaginationOptions {
        select_columns: "p.product_id, p.brand_id, b.name brand_name, p.model, p.description, p.color, p.strap_material, p.strap_color, p.case_material, p.dial_color, p.movement_type, p.water_resistance, p.warranty_period, p.dimensions, p.price::text, p.stock_quantity, p.is_top_model, c.name category_name, s.name shop_name".to_string(),
        base_query,
        search_columns: vec!["b.name", "p.model", "p.description", "p.color", "p.strap_material", "p.strap_color", "p.case_material", "p.dial_color", "p.movement_type", "p.water_resistance", "p.warranty_period", "p.dimensions", "b.name", "c.name", "s.name"].into_iter().map(String::from).collect(),
        search,
        order_options: Some(order_options),
        page,
        per_page,
    });
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

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

    let products: Vec<Product> = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| Product {
            brand_id: row.get("brand_id"),
            name: row.get("name"),
            description: row.get("description"),
            logo_url: row.get("logo_url"),
        })
        .collect();

    Ok(GetProductsResult {
        products,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}
