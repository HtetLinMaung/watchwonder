use serde::Serialize;
use tokio_postgres::{Client, Error};

#[derive(Serialize)]
pub struct Currency {
    pub currency_id: i32,
    pub currency_code: String,
    pub currency_name: String,
    pub symbol: String,
}

pub async fn get_currencies(client: &Client) -> Result<Vec<Currency>, Error> {
    let rows=  client.query("select currency_id, currency_code, currency_name, symbol from currencies where deleted_at is null order by currency_code", &[]).await?;
    Ok(rows
        .iter()
        .map(|row| Currency {
            currency_id: row.get("currency_id"),
            currency_code: row.get("currency_code"),
            currency_name: row.get("currency_name"),
            symbol: row.get("symbol"),
        })
        .collect())
}

pub async fn get_default_currency_id(client: &Client) -> Result<i32, Error> {
    let row = client
        .query_one(
            "select currency_id from currencies where currency_code = 'MMK' and deleted_at is null",
            &[],
        )
        .await?;
    Ok(row.get("currency_id"))
}
