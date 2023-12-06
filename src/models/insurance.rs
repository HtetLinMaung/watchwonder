use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct InsuranceRule {
    pub rule_id: i32,
    pub description: String,
    pub commission_percentage: f64,
    pub min_order_amount: f64,
    pub max_order_amount: f64,
    pub effective_from: NaiveDateTime,
    pub effective_to: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

pub async fn get_insurance_rules(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    amount: Option<f64>,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<InsuranceRule>, Error> {
    let mut base_query = "from commission_rules where deleted_at is null".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    let order_options = match role {
        "admin" => "created_at desc",
        _ => "description",
    };

    if let Some(a) = amount {
        base_query = format!("{base_query} and min_order_amount <= {a} and max_order_amount >= {a}")
    }

    if role != "admin" {
        base_query = format!(
            "{base_query} and now()::date between effective_from::date AND effective_to::date"
        )
    }

    let result=  generate_pagination_query(PaginationOptions {
        select_columns: "rule_id, description, commission_percentage::text, min_order_amount::text, max_order_amount::text, effective_from, effective_to, created_at",
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

    let insurance_rules: Vec<InsuranceRule> = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| {
            let commission_percentage: String = row.get("commission_percentage");
            let min_order_amount: String = row.get("min_order_amount");
            let max_order_amount: String = row.get("max_order_amount");

            InsuranceRule {
                rule_id: row.get("rule_id"),
                description: row.get("description"),
                commission_percentage: commission_percentage.parse().unwrap(),
                min_order_amount: min_order_amount.parse().unwrap(),
                max_order_amount: max_order_amount.parse().unwrap(),
                effective_from: row.get("effective_from"),
                effective_to: row.get("effective_to"),
                created_at: row.get("created_at"),
            }
        })
        .collect();

    Ok(PaginationResult {
        data: insurance_rules,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct InsuranceRuleRequest {
    pub description: String,
    pub commission_percentage: f64,
    pub min_order_amount: f64,
    pub max_order_amount: f64,
    pub effective_from: DateTime<Utc>,
    pub effective_to: DateTime<Utc>,
}

pub async fn add_insurance_rule(data: &InsuranceRuleRequest, client: &Client) -> Result<(), Error> {
    let query = format!("insert into commission_rules (description, commission_percentage, min_order_amount, max_order_amount, effective_from, effective_to) values ($1, {}, {}, {}, $2, $3)", &data.commission_percentage, &data.min_order_amount, &data.max_order_amount);
    client
        .execute(
            &query,
            &[
                &data.description,
                &data.effective_from.naive_utc(),
                &data.effective_to.naive_utc(),
            ],
        )
        .await?;
    Ok(())
}

pub async fn get_insurance_rule_by_id(rule_id: i32, client: &Client) -> Option<InsuranceRule> {
    let result = client
        .query_one(
            "select rule_id, description, commission_percentage::text, min_order_amount::text, max_order_amount::text, effective_from, effective_to, created_at from commission_rules where deleted_at is null and rule_id = $1",
            &[&rule_id],
        )
        .await;

    match result {
        Ok(row) => {
            let commission_percentage: String = row.get("commission_percentage");
            let min_order_amount: String = row.get("min_order_amount");
            let max_order_amount: String = row.get("max_order_amount");

            Some(InsuranceRule {
                rule_id: row.get("rule_id"),
                description: row.get("description"),
                commission_percentage: commission_percentage.parse().unwrap(),
                min_order_amount: min_order_amount.parse().unwrap(),
                max_order_amount: max_order_amount.parse().unwrap(),
                effective_from: row.get("effective_from"),
                effective_to: row.get("effective_to"),
                created_at: row.get("created_at"),
            })
        }
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}

pub async fn update_insurance_rule(
    rule_id: i32,
    data: &InsuranceRuleRequest,
    client: &Client,
) -> Result<(), Error> {
    let query = format!("update commission_rules set description = $1, commission_percentage = {}, min_order_amount = {}, max_order_amount = {}, effective_from = $2, effective_to = $3 where rule_id = $4", &data.commission_percentage, &data.min_order_amount, &data.max_order_amount);
    client
        .execute(
            &query,
            &[
                &data.description,
                &data.effective_from.naive_utc(),
                &data.effective_to.naive_utc(),
                &rule_id,
            ],
        )
        .await?;
    Ok(())
}

pub async fn delete_insurance_rule(rule_id: i32, client: &Client) -> Result<(), Error> {
    client
        .execute(
            "update commission_rules set deleted_at = CURRENT_TIMESTAMP where rule_id = $1",
            &[&rule_id],
        )
        .await?;
    Ok(())
}
