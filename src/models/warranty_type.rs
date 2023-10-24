use serde::Serialize;
use tokio_postgres::Client;

#[derive(Serialize)]
pub struct WarrantyType {
    pub warranty_type_id: i32,
    pub description: String,
}

pub async fn get_warranty_types(
    client: &Client,
) -> Result<Vec<WarrantyType>, Box<dyn std::error::Error>> {
    let rows = client
        .query(
            "select warranty_type_id, description from warranty_types where deleted_at is null order by description",
            &[],
        )
        .await?;
    Ok(rows
        .iter()
        .map(|row| WarrantyType {
            warranty_type_id: row.get("warranty_type_id"),
            description: row.get("description"),
        })
        .collect())
}
