use serde::Serialize;
use tokio_postgres::{Client, Error};

#[derive(Serialize)]
pub struct OtherAccessoriesType {
    pub other_accessories_type_id: i32,
    pub description: String,
}

pub async fn get_other_accessories_types(
    client: &Client,
) -> Result<Vec<OtherAccessoriesType>, Error> {
    let rows=  client.query("select other_accessories_type_id, description from other_accessories_types where deleted_at is null order by description", &[]).await?;

    Ok(rows
        .iter()
        .map(|row| OtherAccessoriesType {
            other_accessories_type_id: row.get("other_accessories_type_id"),
            description: row.get("description"),
        })
        .collect())
}
