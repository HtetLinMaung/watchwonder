use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SellerRegistrationFee {
    pub fee_id: i32,
    pub description: String,
    pub amount: f64,
    pub is_percent: bool,
    pub currency_id: i32,
    pub symbol: String,
    pub created_at: Option<NaiveDateTime>,
}

pub async fn get_seller_registration_fees(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<SellerRegistrationFee>, Error> {
    let base_query = "from seller_registration_fees f join currencies c on c.currency_id = f.currency_id where f.deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = if role == "admin" {
        "f.created_at desc"
    } else {
        "f.description asc"
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "f.fee_id, f.description, f.amount::text, f.is_percent, f.currency_id, c.symbol, f.created_at",
        base_query: &base_query,
        search_columns: vec!["f.fee_id::text", "f.description"],
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

    let seller_registration_fees: Vec<SellerRegistrationFee> = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| {
            let amount: String = row.get("amount");
            let amount: f64 = amount.parse().unwrap();
            SellerRegistrationFee {
                fee_id: row.get("fee_id"),
                description: row.get("description"),
                amount,
                is_percent: row.get("is_percent"),
                currency_id: row.get("currency_id"),
                symbol: row.get("symbol"),
                created_at: row.get("created_at"),
            }
        })
        .collect();

    Ok(PaginationResult {
        data: seller_registration_fees,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Deserialize)]
pub struct SellerRegistrationFeeRequest {
    pub description: String,
    pub amount: f64,
    pub is_percent: bool,
    pub currency_id: i32,
}

pub async fn add_seller_registration_fee(
    data: &SellerRegistrationFeeRequest,
    client: &Client,
) -> Result<(), Error> {
    let query = format!("insert into seller_registration_fees (description, amount, is_percent, currency_id) values ($1, {}, $2, $3)", data.amount);
    client
        .execute(
            &query,
            &[&data.description, &data.is_percent, &data.currency_id],
        )
        .await?;
    Ok(())
}

pub async fn update_seller_registration_fee(
    data: &SellerRegistrationFeeRequest,
    fee_id: i32,
    client: &Client,
) -> Result<(), Error> {
    let query = format!("update seller_registration_fees set description = $1, amount = {}, is_percent = $2, currency_id = $3 where fee_id = $4 and deleted_at is null", data.amount);
    client
        .execute(
            &query,
            &[
                &data.description,
                &data.is_percent,
                &data.currency_id,
                &fee_id,
            ],
        )
        .await?;
    Ok(())
}

pub async fn get_seller_registration_fee_by_id(
    fee_id: i32,
    client: &Client,
) -> Option<SellerRegistrationFee> {
    let result = client
        .query_one(
            "select f.fee_id, f.description, f.amount::text, f.is_percent, f.currency_id, c.symbol, f.created_at from seller_registration_fees f join currencies c on c.currency_id = f.currency_id where f.fee_id = $1 and f.deleted_at is null",
            &[&fee_id],
        )
        .await;

    match result {
        Ok(row) => {
            let amount: String = row.get("amount");
            let amount: f64 = amount.parse().unwrap();
            Some(SellerRegistrationFee {
                fee_id: row.get("fee_id"),
                description: row.get("description"),
                amount,
                is_percent: row.get("is_percent"),
                currency_id: row.get("currency_id"),
                symbol: row.get("symbol"),
                created_at: row.get("created_at"),
            })
        }
        Err(_) => None,
    }
}

pub async fn delete_seller_registration_fee(fee_id: i32, client: &Client) -> Result<(), Error> {
    client.execute(
        "update seller_registration_fees set deleted_at = CURRENT_TIMESTAMP where fee_id = $1 and deleted_at is null",
        &[&fee_id],
    ).await?;
    Ok(())
}
