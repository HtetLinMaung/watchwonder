use std::{fs, path::Path};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Serialize)]
pub struct BankAccount {
    pub account_id: i32,
    pub account_type: String,
    pub account_holder_name: String,
    pub account_number: String,
    pub bank_logo: String,
    pub shop_id: Option<i32>,
    pub shop_name: Option<String>,
    pub created_at: NaiveDateTime,
}

pub async fn get_bank_accounts(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    account_type: &Option<String>,
    shop_id: Option<i32>,
    role: &str,
    user_id: i32,
    client: &Client,
) -> Result<PaginationResult<BankAccount>, Error> {
    let mut base_query = "from bank_accounts where deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = "account_holder_name, account_number";

    if let Some(at) = account_type {
        params.push(Box::new(at));
        base_query = format!("{base_query} and account_type = ${}", params.len());
    }

    if let Some(sid) = shop_id {
        params.push(Box::new(sid));
        base_query = format!("{base_query} and shop_id = ${}", params.len());
    }

    if role == "agent" {
        params.push(Box::new(user_id));
        base_query = format!(
            "{base_query} and shop_id in (select shop_id from shops where creator_id = ${})",
            params.len()
        );
    }

    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "account_id, account_type, account_holder_name, account_number, bank_logo, created_at, shop_id",
        base_query: &base_query,
        search_columns: vec!["account_type", "account_holder_name", "account_number"],
        search: search.as_deref(),
        order_options: Some(&order_options),
        page,
        per_page,
    });

    let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();

    let row = client.query_one(&result.count_query, &params_slice).await?;
    let total: i64 = row.get("total");

    let mut page_counts = 0;
    let mut current_page = 0;
    let mut limit = 0;
    if page.is_some() && per_page.is_some() {
        current_page = page.unwrap();
        limit = per_page.unwrap();
        page_counts = (total as f64 / limit as f64).ceil() as usize;
    }

    let categories: Vec<BankAccount> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| BankAccount {
            account_id: row.get("account_id"),
            account_type: row.get("account_type"),
            account_holder_name: row.get("account_holder_name"),
            account_number: row.get("account_number"),
            bank_logo: row.get("bank_logo"),
            shop_id: row.get("shop_id"),
            shop_name: None,
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: categories,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct BankAccountRequest {
    pub account_type: String,
    pub account_holder_name: String,
    pub account_number: String,
    pub bank_logo: String,
    pub shop_id: Option<i32>,
}

pub async fn add_bank_account(data: &BankAccountRequest, client: &Client) -> Result<(), Error> {
    client
        .execute(
            "insert into bank_accounts (account_type, account_holder_name, account_number, bank_logo, shop_id) values ($1, $2, $3, $4, $5)",
            &[&data.account_type, &data.account_holder_name, &data.account_number, &data.bank_logo, &data.shop_id],
        )
        .await?;
    Ok(())
}

pub async fn get_bank_account_by_id(account_id: i32, client: &Client) -> Option<BankAccount> {
    let result = client
        .query_one(
            "select b.account_id, b.account_type, b.account_holder_name, b.account_number, b.bank_logo, b.created_at, b.shop_id, s.name shop_name from bank_accounts b left join shops s on s.shop_id = b.shop_id where b.deleted_at is null and b.account_id = $1",
            &[&account_id],
        )
        .await;

    match result {
        Ok(row) => Some(BankAccount {
            account_id: row.get("account_id"),
            account_type: row.get("account_type"),
            account_holder_name: row.get("account_holder_name"),
            account_number: row.get("account_number"),
            bank_logo: row.get("bank_logo"),
            shop_id: row.get("shop_id"),
            shop_name: row.get("shop_name"),
            created_at: row.get("created_at"),
        }),
        Err(_) => None,
    }
}

pub async fn update_bank_account(
    account_id: i32,
    old_bank_logo: &str,
    data: &BankAccountRequest,
    client: &Client,
) -> Result<(), Error> {
    client
        .execute(
            "update bank_accounts set account_type = $1, account_holder_name = $2, account_number = $3, bank_logo = $4, shop_id = $5 where account_id = $6",
            &[
                &data.account_type,
                &data.account_holder_name,
                &data.account_number,
                &data.bank_logo,
                &data.shop_id,
                &account_id,
            ],
        )
        .await?;
    if old_bank_logo != &data.bank_logo {
        match fs::remove_file(old_bank_logo) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
        let path = Path::new(&old_bank_logo);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        match fs::remove_file(format!("{stem}_original.{extension}")) {
            Ok(_) => println!("Original file deleted successfully!"),
            Err(e) => println!("Error deleting original file: {}", e),
        };
    }

    Ok(())
}

pub async fn delete_bank_account(
    account_id: i32,
    old_bank_logo: &str,
    client: &Client,
) -> Result<(), Error> {
    client
        .execute(
            "update bank_accounts set deleted_at = CURRENT_TIMESTAMP where account_id = $1",
            &[&account_id],
        )
        .await?;
    match fs::remove_file(old_bank_logo) {
        Ok(_) => println!("File deleted successfully!"),
        Err(e) => println!("Error deleting file: {}", e),
    };
    let path = Path::new(&old_bank_logo);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    match fs::remove_file(format!("{stem}_original.{extension}")) {
        Ok(_) => println!("Original file deleted successfully!"),
        Err(e) => println!("Error deleting original file: {}", e),
    };
    Ok(())
}

pub async fn get_bank_logos(client: &Client) -> Vec<String> {
    match client
        .query("select bank_logo from bank_accounts", &[])
        .await
    {
        Ok(rows) => rows.iter().map(|row| row.get("bank_logo")).collect(),
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    }
}
