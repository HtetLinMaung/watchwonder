use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

use super::address::NewAddress;

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    order_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct OrderItem {
    pub product_id: i32,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct NewOrder {
    pub order_items: Vec<OrderItem>,
    pub address: NewAddress,
}

pub async fn add_order(
    order: &NewOrder,
    user_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client.execute("insert into user_addresses (user_id, street_address, city, state, postal_code, country) values ($1, $2, $3, $4, $5, $6) on conflict (user_id, deleted_at) do update set street_address = excluded.street_address, city = excluded.city, state = excluded.state, postal_code = excluded.postal_code, country = excluded.country", &[&user_id, &order.address.street_address, &order.address.city, &order.address.state, &order.address.postal_code, &order.address.country]).await?;

    let address_row = client.query_one("insert into order_addresses (street_address, city, state, postal_code, country) values ($1, $2, $3, $4, $5) returning address_id", &[&order.address.street_address, &order.address.city, &order.address.state, &order.address.postal_code, &order.address.country]).await?;

    let shipping_address_id: i32 = address_row.get("address_id");

    let order_row = client
        .query_one(
            "insert into orders (user_id, shipping_address_id) values ($1, $2) returning order_id",
            &[&user_id, &shipping_address_id],
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
            "update orders set order_total = (select sum(price) from order_items where order_id = $1 and deleted_at is null) where order_id = $2 and deleted_at is null",
            &[&order_id, &order_id],
        )
        .await?;
    Ok(())
}
