use serde::Deserialize;
use tokio_postgres::Client;

use super::notification;

#[derive(Deserialize)]
pub struct RefundReasonRequest {
    pub order_id: i32,
    pub reason_type_id: i32,
    pub comment: String,
}

pub async fn add_refund_reason(
    data: &RefundReasonRequest,
    user_id: i32,
    client: Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into refund_reasons (order_id, reason_type_id, comment, user_id) values ($1, $2, $3, $4)",
            &[&data.order_id, &data.reason_type_id, &data.comment, &user_id],
        )
        .await?;
    let row = client
        .query_one(
            "select name from users where user_id = $1 and deleted_at is null",
            &[&user_id],
        )
        .await?;
    let name: &str = row.get("name");
    let title = format!("Refund Request Submitted");
    let message = format!(
        "Refund requested by {name} for Order ID #{} - please review and process.",
        &data.order_id
    );
    match notification::add_notification_to_admins(&title, &message, &client).await {
        Ok(()) => {
            println!("Notification added successfully.");
        }
        Err(err) => {
            println!("Error adding notification: {:?}", err);
        }
    };
    Ok(())
}
