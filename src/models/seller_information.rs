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
    pub seller_name: String,
    pub seller_profile_image: String,
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
    let query = format!("select si.company_name, si.professional_title, si.active_since_year, si.location, si.offline_trader, ({}) as product_counts, ({}) as sold_product_counts, u.name seller_name, u.profile_image seller_profile_image from seller_informations si join users u on u.user_id = si.user_id where si.user_id = $3 and si.deleted_at is null", product_counts_query, sold_product_counts_query);
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
            seller_name: row.get("seller_name"),
            seller_profile_image: row.get("seller_profile_image"),
        }),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}
