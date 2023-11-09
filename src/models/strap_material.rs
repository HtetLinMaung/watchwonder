use tokio_postgres::Client;

pub async fn get_strap_materials(
    client: &Client,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let rows = client
        .query(
            "select description from strap_materials where deleted_at is null order by description",
            &[],
        )
        .await?;
    Ok(rows.iter().map(|row| row.get("description")).collect())
}
