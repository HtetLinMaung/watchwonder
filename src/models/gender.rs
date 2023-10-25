use serde::Serialize;
use tokio_postgres::Client;

#[derive(Serialize)]
pub struct Gender {
    pub gender_id: i32,
    pub description: String,
}

pub async fn get_genders(client: &Client) -> Result<Vec<Gender>, Box<dyn std::error::Error>> {
    let rows=  client.query("select gender_id, description from genders where deleted_at is null order by description", &[]).await?;

    Ok(rows
        .iter()
        .map(|row| Gender {
            gender_id: row.get("gender_id"),
            description: row.get("description"),
        })
        .collect())
}
