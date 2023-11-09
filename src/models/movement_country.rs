use tokio_postgres::Client;

pub async fn get_movement_countries(
    client: &Client,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let rows = client
        .query(
            "select description from movement_countries where deleted_at is null order by description",
            &[],
        )
        .await?;
    Ok(rows.iter().map(|row| row.get("description")).collect())
}
