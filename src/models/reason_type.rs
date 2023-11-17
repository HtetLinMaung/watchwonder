use serde::Serialize;
use tokio_postgres::Client;

#[derive(Serialize)]
pub struct ReasonType {
    pub reason_type_id: i32,
    pub description: String,
}

pub async fn get_reason_types(
    client: &Client,
) -> Result<Vec<ReasonType>, Box<dyn std::error::Error>> {
    let rows=  client.query("select reason_type_id, description from reason_types where deleted_at is null order by description", &[]).await?;

    Ok(rows
        .iter()
        .map(|row| ReasonType {
            reason_type_id: row.get("reason_type_id"),
            description: row.get("description"),
        })
        .collect())
}
