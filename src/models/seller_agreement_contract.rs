use std::fs;

use tokio_postgres::{Client, Error};

pub async fn save_seller_agreement_contract(file_path: &str, client: &Client) -> Result<(), Error> {
    let result = client
        .query_one(
            "select file_path from seller_agreement_contract limit 1",
            &[],
        )
        .await;
    match result {
        Ok(row) => {
            let old_file_path: &str = row.get("file_path");
            if old_file_path != file_path {
                match fs::remove_file(old_file_path.replace("/images", "./images")) {
                    Ok(_) => println!("File deleted successfully!"),
                    Err(e) => println!("Error deleting file: {}", e),
                };
            }
            client
                .execute(
                    "update seller_agreement_contract set file_path = $1",
                    &[&file_path],
                )
                .await?;
        }
        Err(err) => {
            println!("{:?}", err);
            client
                .execute(
                    "insert into seller_agreement_contract (file_path) values ($1)",
                    &[&file_path],
                )
                .await?;
        }
    }

    Ok(())
}

pub async fn get_seller_agreement_contract(client: &Client) -> Option<String> {
    match client
        .query_one(
            "select file_path from seller_agreement_contract limit 1",
            &[],
        )
        .await
    {
        Ok(row) => Some(row.get("file_path")),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}
