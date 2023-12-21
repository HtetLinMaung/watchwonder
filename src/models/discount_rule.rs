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
            "dr.rule_id, dr.discount_for, dr.discount_for_id, dr.discount_percent::text, dr.discount_expiration, dr.discount_reason, dr.discounted_price::text, dr.discount_type, dr.shop_id, s.name shop_name, dr.created_at",
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
            "select dr.rule_id, dr.discount_for, dr.discount_for_id, dr.discount_percent::text, dr.discount_expiration, dr.discount_reason, dr.discounted_price::text, dr.discount_type, dr.shop_id, s.name shop_name, dr.created_at from discount_rules dr join shops s on dr.shop_id = s.shop_id where dr.deleted_at is null and dr.rule_id = $1",
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

// pub async fn get_discount_rules_for_calculation(
//     client: &Client,
// ) -> Result<Vec<DiscountRule>, Error> {
//     let rows=  client.query("select rule_id, discount_for, discount_for_id, discount_percent::text, discount_expiration, discount_reason, discounted_price::text, discount_type, shop_id, created_at from discount_rules
//     where (discount_expiration is null or discount_expiration >= CURRENT_DATE) and deleted_at is null
//     ", &[]).await?;
//     Ok(rows
//         .iter()
//         .map(|row| {
//             let discount_percent: &str = row.get("discount_percent");
//             let discount_percent: f64 = discount_percent.parse().unwrap();

//             let discounted_price: &str = row.get("discounted_price");
//             let discounted_price: f64 = discounted_price.parse().unwrap();

//             DiscountRule {
//                 rule_id: row.get("rule_id"),
//                 discount_for: row.get("discount_for"),
//                 discount_for_id: row.get("discount_for_id"),
//                 discount_percent,
//                 discount_expiration: row.get("discount_expiration"),
//                 discount_reason: row.get("discount_reason"),
//                 discounted_price,
//                 discount_type: row.get("discount_type"),
//                 shop_id: row.get("shop_id"),
//                 shop_name: "".to_string(),
//                 created_at: row.get("created_at"),
//             }
//         })
//         .collect())
// }

pub struct DiscountCalculationResult {
    pub discount_percent: f64,
    pub discounted_price: f64,
    pub discount_reason: String,
    pub discount_type: String,
}

pub async fn calculate_discounted_price(
    price: f64,
    product_id: i32,
    shop_id: i32,
    category_id: i32,
    brand_id: i32,
    client: &Client,
) -> DiscountCalculationResult {
    let query = "
    SELECT 
    rule_id, 
    discount_for, 
    discount_for_id, 
    discount_percent::text, 
    discount_expiration, 
    discount_reason, 
    discounted_price::text, 
    discount_type, 
    shop_id, 
    created_at 
FROM discount_rules 
WHERE 
    deleted_at IS NULL 
    AND (discount_expiration IS NULL OR discount_expiration >= CURRENT_DATE)
    AND shop_id = $1 
    AND (
        (discount_for = 'product' AND discount_for_id = $2) 
        OR (discount_for = 'brand' AND discount_for_id = $3) 
        OR (discount_for = 'category' AND discount_for_id = $4) 
        OR discount_for = 'all'
    ) 
ORDER BY 
    CASE 
        WHEN discount_for = 'product' THEN 1 
        WHEN discount_for = 'brand' THEN 2 
        WHEN discount_for = 'category' THEN 3 
        WHEN discount_for = 'all' THEN 4 
    END 
LIMIT 1;
    ";
    match client
        .query_one(query, &[&shop_id, &product_id, &brand_id, &category_id])
        .await
    {
        Ok(row) => {
            let discount_percent: &str = row.get("discount_percent");
            let discount_percent: f64 = discount_percent.parse().unwrap();

            let discounted_price: &str = row.get("discounted_price");
            let discounted_price: f64 = discounted_price.parse().unwrap();

            let rule = DiscountRule {
                rule_id: row.get("rule_id"),
                discount_for: row.get("discount_for"),
                discount_for_id: row.get("discount_for_id"),
                discount_percent,
                discount_expiration: row.get("discount_expiration"),
                discount_reason: row.get("discount_reason"),
                discounted_price,
                discount_type: row.get("discount_type"),
                shop_id: row.get("shop_id"),
                shop_name: "".to_string(),
                created_at: row.get("created_at"),
            };

            return calculate_discount(price, &rule);
        }
        Err(err) => {
            println!("Discount calculation error: {:?}", err);
            return DiscountCalculationResult {
                discount_percent: 0.0,
                discounted_price: price,
                discount_reason: String::new(),
                discount_type: "No Discount".to_string(),
            };
        }
    };
}

fn calculate_discount(price: f64, rule: &DiscountRule) -> DiscountCalculationResult {
    match rule.discount_type.as_str() {
        "Discount by Specific Percentage" => DiscountCalculationResult {
            discount_percent: rule.discount_percent,
            discounted_price: price * (1.0 - rule.discount_percent / 100.0),
            discount_reason: rule.discount_reason.clone(),
            discount_type: rule.discount_type.clone(),
        },
        "Discount by Specific Amount" => DiscountCalculationResult {
            discount_percent: 0.0,
            discounted_price: rule.discounted_price,
            discount_reason: rule.discount_reason.clone(),
            discount_type: rule.discount_type.clone(),
        },
        _ => DiscountCalculationResult {
            discount_percent: 0.0,
            discounted_price: price,
            discount_reason: String::new(),
            discount_type: rule.discount_type.clone(),
        },
    }
}

// pub fn calculate_discounted_price(
//     price: f64,
//     product_id: i32,
//     shop_id: i32,
//     category_id: i32,
//     brand_id: i32,
//     discount_rules: &[DiscountRule],
// ) -> (f64, String) {
//     let mut applicable_rules: Vec<&DiscountRule> = discount_rules
//         .iter()
//         .filter(|rule| rule.shop_id == shop_id)
//         .collect();

//     // Sort rules by priority
//     applicable_rules.sort_by_key(|rule| match rule.discount_for.as_str() {
//         "product" => 1,
//         "brand" => 2,
//         "category" => 3,
//         "all" => 4,
//         _ => 5,
//     });

//     for rule in applicable_rules {
//         match rule.discount_for.as_str() {
//             "product" if rule.discount_for_id == product_id => {
//                 return calculate_discount(price, rule)
//             }
//             "brand" if rule.discount_for_id == brand_id => return calculate_discount(price, rule),
//             "category" if rule.discount_for_id == category_id => {
//                 return calculate_discount(price, rule)
//             }
//             "all" => return calculate_discount(price, rule),
//             _ => (),
//         }
//     }

//     (price, String::new()) // default case if no rule matches
// }
