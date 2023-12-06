use tokio_postgres::{Client, Error};

use crate::utils::setting;

pub async fn get_payment_types(amount: f64, client: &Client) -> Result<Vec<String>, Error> {
    let max_cash_on_delivery_amount = setting::get_max_cash_on_delivery_amount();

    let mut query = "select description from payment_types where deleted_at is null".to_string();
    if amount >= max_cash_on_delivery_amount {
        query = format!("{query} and description != 'Cash on Delivery'")
    }
    query = format!("{query} order by description");

    let rows = client.query(&query, &[]).await?;
    Ok(rows.iter().map(|row| row.get("description")).collect())
}
