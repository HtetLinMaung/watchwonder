use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Serialize)]
pub struct DiscountRule {
    pub rule_id: i32,
    pub discount_for: String,
    pub discount_for_id: i32,
    pub discount_percent: f64,
    pub discount_expiration: Option<NaiveDateTime>,
    pub discount_reason: String,
    pub discounted_price: f64,
    pub discount_type: String,
    pub shop_id: i32,
    pub shop_name: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_discount_rules(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    shop_id: Option<i32>,
    creator_id: i32,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<DiscountRule>, Error> {
    let mut base_query =
        "from discount_rules dr join shops s on dr.shop_id = s.shop_id where dr.deleted_at is null"
            .to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = "created_at desc";

    if role == "admin" {
        if let Some(sid) = shop_id {
            params.push(Box::new(sid));
            base_query = format!("{base_query} and dr.shop_id = ${}", params.len());
        }
    } else {
        params.push(Box::new(creator_id));
        base_query = format!("{base_query} and dr.creator_id = ${}", params.len())
    }

    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "dr.rule_id, dr.discount_for, dr.discount_for_id, dr.discount_percent, dr.discount_expiration, dr.discount_reason, dr.discounted_price, dr.discount_type, dr.shop_id, s.name shop_name, dr.created_at",
        base_query: &base_query,
        search_columns: vec!["dr.rule_id::text", "dr.discount_for", "dr.discount_reason", "dr.discount_type", "s.name"],
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

    let discount_rules: Vec<DiscountRule> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| {
            let discount_percent: &str = row.get("discount_percent");
            let discount_percent: f64 = discount_percent.parse().unwrap();

            let discounted_price: &str = row.get("discounted_price");
            let discounted_price: f64 = discounted_price.parse().unwrap();

            DiscountRule {
                rule_id: row.get("rule_id"),
                discount_for: row.get("discount_for"),
                discount_for_id: row.get("discount_for_id"),
                discount_percent,
                discount_expiration: row.get("discount_expiration"),
                discount_reason: row.get("discount_reason"),
                discounted_price,
                discount_type: row.get("discount_type"),
                shop_id: row.get("shop_id"),
                shop_name: row.get("shop_name"),
                created_at: row.get("created_at"),
            }
        })
        .collect();

    Ok(PaginationResult {
        data: discount_rules,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct DiscountRuleRequest {
    pub discount_for: String,
    pub discount_for_id: i32,
    pub discount_percent: f64,
    pub discount_expiration: Option<String>,
    pub discount_reason: String,
    pub discounted_price: f64,
    pub discount_type: String,
    pub shop_id: i32,
}

pub async fn add_discount_rule(
    data: &DiscountRuleRequest,
    creator_id: i32,
    role: &str,
    client: &Client,
) -> Result<(), Error> {
    let discount_expiration = if let Some(de) = &data.discount_expiration {
        format!("'{de}'")
    } else {
        "null".to_string()
    };

    let query = format!("insert into discount_rules (discount_for, discount_for_id, discount_percent, discount_expiration, discount_reason, discounted_price, discount_type, creator_id, shop_id) values ($1, $2, {}, {}, $3, {}, $4, $5, $6)", data.discount_percent, discount_expiration, data.discounted_price);
    client
        .execute(
            &query,
            &[
                &data.discount_for,
                &data.discount_for_id,
                &data.discount_reason,
                &data.discount_type,
                &creator_id,
                &data.shop_id,
            ],
        )
        .await?;
    Ok(())
}

pub async fn get_discount_rule_by_id(rule_id: i32, client: &Client) -> Option<DiscountRule> {
    let result = client
        .query_one(
            "select dr.rule_id, dr.discount_for, dr.discount_for_id, dr.discount_percent, dr.discount_expiration, dr.discount_reason, dr.discounted_price, dr.discount_type, dr.shop_id, s.name shop_name, dr.created_at from discount_rules dr join shops s on dr.shop_id = s.shop_id where dr.deleted_at is null and dr.rule_id = $1",
            &[&rule_id],
        )
        .await;

    match result {
        Ok(row) => {
            let discount_percent: &str = row.get("discount_percent");
            let discount_percent: f64 = discount_percent.parse().unwrap();

            let discounted_price: &str = row.get("discounted_price");
            let discounted_price: f64 = discounted_price.parse().unwrap();

            Some(DiscountRule {
                rule_id: row.get("rule_id"),
                discount_for: row.get("discount_for"),
                discount_for_id: row.get("discount_for_id"),
                discount_percent,
                discount_expiration: row.get("discount_expiration"),
                discount_reason: row.get("discount_reason"),
                discounted_price,
                discount_type: row.get("discount_type"),
                shop_id: row.get("shop_id"),
                shop_name: row.get("shop_name"),
                created_at: row.get("created_at"),
            })
        }
        Err(_) => None,
    }
}

pub async fn update_discount_rule(
    rule_id: i32,
    data: &DiscountRuleRequest,
    role: &str,
    client: &Client,
) -> Result<(), Error> {
    let discount_expiration = if let Some(de) = &data.discount_expiration {
        format!("'{de}'")
    } else {
        "null".to_string()
    };

    let query = format!("update discount_rules set discount_for = $1, discount_for_id = $2, discount_percent = {}, discount_expiration = {}, discount_reason = $3, discounted_price = {}, discount_type = $4, shop_id = $5 where rule_id = $6", data.discount_percent, discount_expiration, data.discounted_price);
    client
        .execute(
            &query,
            &[
                &data.discount_for,
                &data.discount_for_id,
                &data.discount_reason,
                &data.discount_type,
                &data.shop_id,
                &rule_id,
            ],
        )
        .await?;

    Ok(())
}

pub async fn delete_discount_rule(rule_id: i32, client: &Client) -> Result<(), Error> {
    client
        .execute(
            "update discount_rules set deleted_at = CURRENT_TIMESTAMP where rule_id = $1",
            &[&rule_id],
        )
        .await?;
    Ok(())
}
