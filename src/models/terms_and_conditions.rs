use tokio_postgres::{Client, Error};

pub async fn add_terms_and_conditions(content: &str, client: &Client) -> Result<(), Error> {
    let rows = client
        .query("select id from terms_and_conditions limit 1", &[])
        .await?;

    if rows.len() == 0 {
        client
            .execute(
                "insert into terms_and_conditions (content) values ($1)",
                &[&content],
            )
            .await?;
    } else {
        let id: i32 = rows[0].get("id");
        client
            .execute(
                "update terms_and_conditions set content = $1 where id = $2",
                &[&content, &id],
            )
            .await?;
    }
    Ok(())
}

pub async fn get_terms_and_conditions(client: &Client) -> String {
    match client
        .query_one("select content from terms_and_conditions limit 1", &[])
        .await
    {
        Ok(row) => row.get("content"),
        Err(_) => "".to_string(),
    }
}
