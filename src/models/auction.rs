use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

#[derive(Deserialize)]
pub struct AuctionRequest {
    pub product_id: i32,
    pub start_time: String,
    pub end_time: String,
    pub start_bid: f64,
    pub reserve_price: f64,
    pub buy_it_now_available: bool,
}

pub async fn add_auction(data: &AuctionRequest, client: &Client) -> Result<i32, Error> {
    let query: String =format!("insert into auctions (product_id, start_time, end_time, start_bid, reserve_price, buy_it_now_available) values ($1, '{}', '{}', {}, {}, $2) returning auction_id", data.start_time, data.end_time, data.start_bid, data.reserve_price);
    let row = client
        .query_one(&query, &[&data.product_id, &data.buy_it_now_available])
        .await?;
    let auction_id = row.get("auction_id");
    Ok(auction_id)
}

#[derive(Serialize)]
pub struct Auction {
    pub auction_id: i32,
    pub product_id: i32,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub start_bid: f64,
    pub current_bid: f64,
    pub reserve_price: f64,
    pub buy_it_now_available: bool,
    pub created_at: NaiveDateTime,
}

pub async fn get_auctions(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    client: &Client,
) -> Result<PaginationResult<Auction>, Error> {
    let base_query = "from auctions where deleted_at is null and status = 'active'".to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let order_options = "created_at desc";

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "auction_id, product_id, start_time, end_time, start_bid::text, current_bid::text, reserve_price::text, buy_it_now_available, created_at",
        base_query: &base_query,
        search_columns: vec!["auction_id::text"],
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

    let auctions: Vec<Auction> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| {
            let start_bid: String = row.get("start_bid");
            let start_bid: f64 = start_bid.parse().unwrap();

            let current_bid: String = row.get("current_bid");
            let current_bid: f64 = current_bid.parse().unwrap();

            let reserve_price: String = row.get("reserve_price");
            let reserve_price: f64 = reserve_price.parse().unwrap();

            Auction {
                auction_id: row.get("auction_id"),
                product_id: row.get("product_id"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                start_bid,
                current_bid,
                reserve_price,
                buy_it_now_available: row.get("buy_it_now_available"),
                created_at: row.get("created_at"),
            }
        })
        .collect();

    Ok(PaginationResult {
        data: auctions,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_auction_by_id(auction_id: i32, client: &Client) -> Option<Auction> {
    match client
        .query_one(
            "select auction_id, product_id, start_time, end_time, start_bid::text, current_bid::text, reserve_price::text, buy_it_now_available, created_at from auctions where auction_id = $1 and deleted_at is null",
            &[&auction_id],
        )
        .await
    {
        Ok(row) => {
            let start_bid: String = row.get("start_bid");
            let start_bid: f64 = start_bid.parse().unwrap();

            let current_bid: String = row.get("current_bid");
            let current_bid: f64 = current_bid.parse().unwrap();

            let reserve_price: String = row.get("reserve_price");
            let reserve_price: f64 = reserve_price.parse().unwrap();

            Some(Auction {
                auction_id: row.get("auction_id"),
                product_id: row.get("product_id"),
                start_time: row.get("start_time"),
                end_time: row.get("end_time"),
                start_bid,
                current_bid,
                reserve_price,
                buy_it_now_available: row.get("buy_it_now_available"),
                created_at: row.get("created_at"),
        })},
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}

pub async fn update_auction(
    auction_id: i32,
    data: &AuctionRequest,
    client: &Client,
) -> Result<(), Error> {
    // product_id, start_time, end_time, start_bid, reserve_price, buy_it_now_available
    let query = format!("update auctions set product_id = $1, start_time = '{}', end_time = '{}', start_bid = {}, reserve_price = {}, buy_it_now_available = $2 where auction_id = $3", data.start_time, data.end_time, data.start_bid, data.reserve_price);
    client
        .execute(
            &query,
            &[&data.product_id, &data.buy_it_now_available, &auction_id],
        )
        .await?;
    Ok(())
}

pub async fn delete_auction(auction_id: i32, client: &Client) -> Result<(), Error> {
    client
        .execute(
            "update auctions set deleted_at = CURRENT_TIMESTAMP where auction_id = $1",
            &[&auction_id],
        )
        .await?;
    Ok(())
}
