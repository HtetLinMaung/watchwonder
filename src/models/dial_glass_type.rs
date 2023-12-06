use serde::Serialize;
use tokio_postgres::{Client, Error};

#[derive(Serialize)]
pub struct DialGlassType {
    pub dial_glass_type_id: i32,
    pub description: String,
}

pub async fn get_dial_glass_types(client: &Client) -> Result<Vec<DialGlassType>, Error> {
    let rows=  client.query("select dial_glass_type_id, description from dial_glass_types where deleted_at is null order by description", &[]).await?;

    Ok(rows
        .iter()
        .map(|row| DialGlassType {
            dial_glass_type_id: row.get("dial_glass_type_id"),
            description: row.get("description"),
        })
        .collect())
}
