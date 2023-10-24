use serde::Serialize;
use tokio_postgres::Client;

#[derive(Serialize)]
pub struct BankAccount {
    pub account_holder_name: String,
    pub account_number: String,
    pub bank_logo: String,
}

pub async fn get_bank_accounts(
    client: &Client,
) -> Result<Vec<BankAccount>, Box<dyn std::error::Error>> {
    let rows= client.query(
        "select account_holder_name, account_number, bank_logo from bank_accounts where deleted_at is null order by account_holder_name, account_number",
        &[],
    ).await?;
    Ok(rows
        .iter()
        .map(|row| BankAccount {
            account_holder_name: row.get("account_holder_name"),
            account_number: row.get("account_number"),
            bank_logo: row.get("bank_logo"),
        })
        .collect())
}
