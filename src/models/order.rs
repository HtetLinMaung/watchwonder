use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

use super::address::NewAddress;

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_id: i32,
    pub user_name: String,
    pub phone: String,
    pub email: String,
    pub home_address: String,
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
    pub township: String,
    pub ward: String,
    pub note: String,
    pub status: String,
    pub order_total: f64,
    pub item_counts: i32,
    pub payment_type: String,
    pub payslip_screenshot_path: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItem {
    pub order_item_id: i32,
    pub order_id: i32,
    pub brand: String,
    pub model: String,
    pub quantity: i32,
    pub price: f64,
    pub amount: f64,
    pub product_images: Vec<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct NewOrderItem {
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct NewOrder {
    pub order_items: Vec<NewOrderItem>,
    pub address: NewAddress,
    pub payment_type: String,
    pub payslip_screenshot_path: String,
}

pub async fn add_order(
    order: &NewOrder,
    user_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client.execute("INSERT INTO user_addresses (user_id, street_address, city, state, postal_code, country, township, home_address, ward) 
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
    ON CONFLICT (user_id) WHERE deleted_at IS NULL
    DO UPDATE SET street_address = excluded.street_address, city = excluded.city, state = excluded.state, postal_code = excluded.postal_code, country = excluded.country;
    ", 
    &[&user_id, &order.address.street_address, &order.address.city, &order.address.state, &order.address.postal_code, &order.address.country, &order.address.township, &order.address.home_address, &order.address.ward]
).await?;

    let address_row = client.query_one("insert into order_addresses (street_address, city, state, postal_code, country, township, home_address, ward, note) values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning address_id", &[&order.address.street_address, &order.address.city, &order.address.state, &order.address.postal_code, &order.address.country, &order.address.township,&order.address.home_address, &order.address.ward, &order.address.note]).await?;

    let shipping_address_id: i32 = address_row.get("address_id");

    let order_row = client
        .query_one(
            "insert into orders (user_id, shipping_address_id, payment_type, payslip_screenshot_path) values ($1, $2, $3, $4) returning order_id",
            &[&user_id, &shipping_address_id, &order.payment_type, &order.payslip_screenshot_path],
        )
        .await?;
    let order_id: i32 = order_row.get("order_id");

    for item in &order.order_items {
        let query = format!("insert into order_items (order_id, product_id, quantity, price) values ($1, $2, $3, (select (coalesce(price, 0.0)) from products where product_id = $4 and deleted_at is null))");
        client
            .execute(
                &query,
                &[
                    &order_id,
                    &item.product_id,
                    &item.quantity,
                    &item.product_id,
                ],
            )
            .await?;
    }

    client
        .execute(
            "update orders set order_total = (select sum(price * quantity) from order_items where order_id = $1 and deleted_at is null), item_counts = (select count(*) from order_items where order_id = $2 and deleted_at is null) where order_id = $3 and deleted_at is null",
            &[&order_id, &order_id, &order_id],
        )
        .await?;
    Ok(())
}

pub async fn get_orders(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    from_date: &Option<NaiveDate>,
    to_date: &Option<NaiveDate>,
    from_amount: &Option<f64>,
    to_amount: &Option<f64>,
    payment_type: &Option<String>,
    user_id: i32,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<Order>, Error> {
    let mut base_query =
        "from orders o inner join users u on o.user_id = u.user_id inner join order_addresses a on o.shipping_address_id = a.address_id where o.deleted_at is null and u.deleted_at is null and a.deleted_at is null"
            .to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    if role == "user" {
        params.push(Box::new(user_id));
        base_query = format!("{base_query} and o.user_id = ${}", params.len());
    }

    if from_date.is_some() && to_date.is_some() {
        params.push(Box::new(from_date.unwrap()));
        params.push(Box::new(to_date.unwrap()));
        base_query = format!(
            "{base_query} and o.created_at::date between ${} and ${}",
            params.len() - 1,
            params.len()
        );
    }

    if from_amount.is_some() && to_amount.is_some() {
        base_query = format!(
            "{base_query} and o.order_total between {} and {}",
            from_amount.unwrap(),
            to_amount.unwrap()
        );
    }

    if let Some(pt) = payment_type {
        params.push(Box::new(pt));
        base_query = format!("{base_query} and o.payment_type = ${}", params.len());
    }

    let order_options = "o.created_at desc".to_string();

    let result=  generate_pagination_query(PaginationOptions {
        select_columns: "o.order_id, u.name user_name, u.phone, u.email, a.home_address, a.street_address, a.city, a.state, a.postal_code, a.country, a.township, a.ward, a.note, o.created_at, o.status, o.order_total::text, o.item_counts, o.payment_type, o.payslip_screenshot_path",
        base_query: &base_query,
        search_columns: vec![ "u.name", "u.phone", "u.email", "a.home_address", "a.street_address", "a.city", "a.state", "a.postal_code", "a.country", "a.township", "a.ward", "a.note","o.status", "o.payment_type"],
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

    let orders = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| {
            let order_total: String = row.get("order_total");
            let order_total: f64 = order_total.parse().unwrap();
            return Order {
                order_id: row.get("order_id"),
                user_name: row.get("user_name"),
                phone: row.get("phone"),
                email: row.get("email"),
                home_address: row.get("home_address"),
                street_address: row.get("street_address"),
                city: row.get("city"),
                state: row.get("state"),
                postal_code: row.get("postal_code"),
                country: row.get("country"),
                township: row.get("township"),
                ward: row.get("ward"),
                note: row.get("note"),
                status: row.get("status"),
                order_total,
                created_at: row.get("created_at"),
                item_counts: row.get("item_counts"),
                payment_type: row.get("payment_type"),
                payslip_screenshot_path: row.get("payslip_screenshot_path"),
            };
        })
        .collect();

    Ok(PaginationResult {
        data: orders,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_order_items(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    from_date: &Option<NaiveDate>,
    to_date: &Option<NaiveDate>,
    order_id: Option<i32>,
    user_id: i32,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<OrderItem>, Error> {
    let mut base_query = "from order_items oi inner join orders o on oi.order_id = o.order_id inner join products p on p.product_id = oi.product_id inner join brands b on b.brand_id = p.brand_id where oi.deleted_at is null and o.deleted_at is null and p.deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    if role == "user" {
        params.push(Box::new(user_id));
        base_query = format!("{base_query} and o.user_id = ${}", params.len());
    }

    if let Some(oi) = order_id {
        params.push(Box::new(oi));
        base_query = format!("{base_query} and oi.order_id = ${}", params.len());
    }

    if from_date.is_some() && to_date.is_some() {
        params.push(Box::new(from_date.unwrap()));
        params.push(Box::new(to_date.unwrap()));
        base_query = format!(
            "{base_query} and oi.created_at::date between ${} and ${}",
            params.len() - 1,
            params.len()
        );
    }

    let order_options = match role {
        "admin" => "oi.created_at desc".to_string(),
        "user" => "b.name, p.model".to_string(),
        _ => "".to_string(),
    };

    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "oi.order_item_id, o.order_id, b.name brand, p.product_id, p.model, oi.quantity, oi.price::text, (oi.price * oi.quantity)::text as amount, oi.created_at",
        base_query: &base_query,
        search_columns: vec!["b.name", "p.model"],
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

    let rows = client.query(&result.query, &params_slice[..]).await?;

    let mut order_items = Vec::new();
    for row in &rows {
        let price: String = row.get("price");
        let price: f64 = price.parse().unwrap();
        let amount: String = row.get("amount");
        let amount: f64 = amount.parse().unwrap();
        let product_id: i32 = row.get("product_id");
        let image_rows = client
            .query(
                "select image_url from product_images where product_id = $1 and deleted_at is null",
                &[&product_id],
            )
            .await?;
        let product_images: Vec<String> = image_rows.iter().map(|r| r.get("image_url")).collect();
        order_items.push(OrderItem {
            order_item_id: row.get("order_item_id"),
            order_id: row.get("order_id"),
            brand: row.get("brand"),
            model: row.get("model"),
            quantity: row.get("quantity"),
            price,
            amount,
            created_at: row.get("created_at"),
            product_images,
        });
    }

    Ok(PaginationResult {
        data: order_items,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn update_order(
    order_id: i32,
    status: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update orders set status = $1 where order_id = $2 and deleted_at is null",
            &[&status, &order_id],
        )
        .await?;
    Ok(())
}

pub async fn order_exists(order_id: i32, client: &Client) -> Result<bool, Error> {
    // Execute a query to check if the username exists in the users table
    let row = client
        .query_one(
            "select order_id from orders where order_id = $1 and deleted_at is null",
            &[&order_id],
        )
        .await;

    // Return whether the user exists
    Ok(row.is_ok())
}
