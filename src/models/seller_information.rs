use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct SellerInformation {
    pub company_name: String,
    pub professional_title: String,
    pub active_since_year: i32,
    pub location: String,
    pub offline_trader: bool,
    pub product_counts: i64,
    pub sold_product_counts: i64,
}

#[derive(Deserialize, Debug)]
pub struct SellerInformationRequest {
    pub company_name: String,
    pub professional_title: String,
    pub location: String,
    pub offline_trader: bool,
}

pub async fn get_seller_information(user_id: i32, client: &Client) -> Option<SellerInformation> {
    let product_counts_query =
        "select count(*) from products where creator_id = $1 and deleted_at is null";
    let sold_product_counts_query = "select coalesce(sum(oi.quantity), 0) from order_items oi inner join products p on p.product_id = oi.product_id inner join orders o on o.order_id = oi.order_id where oi.deleted_at is null and p.deleted_at is null and o.deleted_at is null and o.status = 'Completed' and p.creator_id = $2";
    let query = format!("select company_name, professional_title, active_since_year, location, offline_trader, ({}) as product_counts, ({}) as sold_product_counts from seller_informations where user_id = $3 and deleted_at is null", product_counts_query, sold_product_counts_query);
    match client
        .query_one(&query, &[&user_id, &user_id, &user_id])
        .await
    {
        Ok(row) => Some(SellerInformation {
            company_name: row.get("company_name"),
            professional_title: row.get("professional_title"),
            active_since_year: row.get("active_since_year"),
            location: row.get("location"),
            offline_trader: row.get("offline_trader"),
            product_counts: row.get("product_counts"),
            sold_product_counts: row.get("sold_product_counts"),
        }),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}
