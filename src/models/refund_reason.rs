use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

use super::order;

#[derive(Deserialize)]
pub struct RefundReasonRequest {
    pub order_id: i32,
    pub reason_type_id: i32,
    pub comment: String,
}

#[derive(Serialize)]
pub struct RefundReason {
    customer_name: String,
    reason_type_description: String,
    comment: String,
    created_at: NaiveDateTime,
}

pub async fn get_refund_reason_by_order_id(order_id: i32, client: &Client) -> Option<RefundReason> {
    match client.query_one("select u.name customer_name, rt.description reason_type_description, rs.comment, rs.created_at from refund_reasons rs join users u on u.user_id = rs.user_id join reason_types rt on rt.reason_type_id = rs.reason_type_id where rs.order_id = $1 and rs.deleted_at is null and rt.deleted_at is null and u.deleted_at is null", &[&order_id]).await {
    Ok(row) => Some(RefundReason {
        customer_name: row.get("customer_name"),
        reason_type_description: row.get("reason_type_description"),
        comment: row.get("comment"),
        created_at: row.get("created_at"),
    }),
    Err(err) => {
        println!("{:?}",err);
        None
    }
  }
}

pub async fn add_refund_reason(
    data: &RefundReasonRequest,
    user_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client.execute("BEGIN", &[]).await?;

    let row = client
        .query_one(
            "select status from orders where order_id = $1 and deleted_at is null for update",
            &[&data.order_id],
        )
        .await?;
    let old_status: &str = row.get("status");
    if old_status != "Returned" && old_status != "Refunded" {
        order::update_order(data.order_id, "Returned", client).await?;
        client
            .execute(
                "insert into refund_reasons (order_id, reason_type_id, comment, user_id) values ($1, $2, $3, $4)",
                &[&data.order_id, &data.reason_type_id, &data.comment, &user_id],
            )
            .await?;
    }

    client.execute("COMMIT", &[]).await?;
    Ok(())
}
