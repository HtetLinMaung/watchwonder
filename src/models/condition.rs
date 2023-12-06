use serde::Serialize;
use tokio_postgres::{Client, Error};

#[derive(Serialize)]
pub struct Condition {
    pub condition_id: i32,
    pub description: String,
}

pub async fn get_conditions(client: &Client) -> Result<Vec<Condition>, Error> {
    let rows=  client.query("select condition_id, description from conditions where deleted_at is null order by description", &[]).await?;

    Ok(rows
        .iter()
        .map(|row| Condition {
            condition_id: row.get("condition_id"),
            description: row.get("description"),
        })
        .collect())
}
