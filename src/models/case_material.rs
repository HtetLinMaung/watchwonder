use tokio_postgres::{Client, Error};

pub async fn get_case_materials(client: &Client) -> Result<Vec<String>, Error> {
    let rows = client
        .query(
            "select description from case_materials where deleted_at is null order by description",
            &[],
        )
        .await?;
    Ok(rows.iter().map(|row| row.get("description")).collect())
}
