use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub address_id: i32,
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewAddress {
    pub street_address: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

pub async fn get_address(user_id: i32, client: &Client) -> Result<Option<Address>, Error> {
    let row=  client.query_one(
        "select address_id, street_address, city, state, postal_code, country from user_addresses where user_id = $1 and deleted_at is null",
        &[&user_id],
    ).await?;
    if row.is_empty() {
        return Ok(None);
    }
    Ok(Some(Address {
        address_id: row.get("address_id"),
        street_address: row.get("street_address"),
        city: row.get("city"),
        state: row.get("state"),
        postal_code: row.get("postal_code"),
        country: row.get("country"),
    }))
}
