use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Serialize)]
pub struct BuyerProtection {
    pub buyer_protection_id: i32,
    pub description: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_buyer_protections(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    client: &Client,
) -> Result<PaginationResult<BuyerProtection>, Box<dyn std::error::Error>> {
    let base_query = "from buyer_protections where deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    let order_options = "created_at";

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "buyer_protection_id, description, created_at",
        base_query: &base_query,
        search_columns: vec!["description"],
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

    let mut buyer_protections: Vec<BuyerProtection> = vec![];
    for row in client.query(&result.query, &params_slice[..]).await? {
        buyer_protections.push(BuyerProtection {
            buyer_protection_id: row.get("buyer_protection_id"),
            description: row.get("description"),
            created_at: row.get("created_at"),
        });
    }

    Ok(PaginationResult {
        data: buyer_protections,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct BuyerProtectionRequest {
    pub description: String,
}

pub async fn add_buyer_protection(
    data: &BuyerProtectionRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into buyer_protections (description) values ($1)",
            &[&data.description],
        )
        .await?;
    Ok(())
}

pub async fn get_buyer_protection_by_id(
    buyer_protection_id: i32,
    client: &Client,
) -> Option<BuyerProtection> {
    let result = client
        .query_one(
            "select buyer_protection_id, description, created_at from buyer_protections where deleted_at is null and buyer_protection_id = $1",
            &[&buyer_protection_id],
        )
        .await;

    match result {
        Ok(row) => Some(BuyerProtection {
            buyer_protection_id: row.get("buyer_protection_id"),
            description: row.get("description"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_buyer_protection(
    buyer_protection_id: i32,
    data: &BuyerProtectionRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update buyer_protections set description = $1 where buyer_protection_id = $2",
            &[&data.description, &buyer_protection_id],
        )
        .await?;
    Ok(())
}

pub async fn delete_buyer_protection(
    buyer_protection_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update buyer_protections set deleted_at = CURRENT_TIMESTAMP where buyer_protection_id = $1",
            &[&buyer_protection_id],
        )
        .await?;
    Ok(())
}
