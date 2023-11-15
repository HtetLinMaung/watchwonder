use tokio_postgres::Client;

pub async fn get_payment_types(client: &Client) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let rows = client
        .query(
            "select description from payment_types where deleted_at is null order by description",
            &[],
        )
        .await?;

    Ok(rows.iter().map(|row| row.get("description")).collect())
}
