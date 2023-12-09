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

    pub facebook_profile_image: String,
    pub shop_or_page_name: String,
    pub facebook_page_image: String,
    pub bussiness_phone: String,
    pub address: String,
    pub nrc: String,
    pub nrc_front_image: String,
    pub nrc_back_image: String,
    pub bank_code: String,
    pub bank_account: String,
    pub bank_account_image: String,
    pub wallet_type: String,
    pub wallet_account: String,
    pub fee_id: i32,
    pub monthly_transaction_screenshot: String,
}

#[derive(Deserialize, Debug)]
pub struct SellerInformationRequest {
    pub company_name: String,
    pub professional_title: String,
    pub location: String,
    pub offline_trader: bool,

    pub facebook_profile_image: Option<String>,
    pub shop_or_page_name: Option<String>,
    pub facebook_page_image: Option<String>,
    pub bussiness_phone: Option<String>,
    pub address: Option<String>,
    pub nrc: Option<String>,
    pub nrc_front_image: Option<String>,
    pub nrc_back_image: Option<String>,
    pub bank_code: Option<String>,
    pub bank_account: Option<String>,
    pub bank_account_image: Option<String>,
    pub wallet_type: Option<String>,
    pub wallet_account: Option<String>,
    pub fee_id: Option<i32>,
    pub monthly_transaction_screenshot: Option<String>,
}

pub async fn get_seller_information(user_id: i32, client: &Client) -> Option<SellerInformation> {
    let product_counts_query =
        "select count(*) from products where creator_id = $1 and deleted_at is null";
    let sold_product_counts_query = "select coalesce(sum(oi.quantity), 0) from order_items oi inner join products p on p.product_id = oi.product_id inner join orders o on o.order_id = oi.order_id where oi.deleted_at is null and p.deleted_at is null and o.deleted_at is null and o.status = 'Completed' and p.creator_id = $2";
    let query = format!("select si.company_name, si.professional_title, si.active_since_year, si.location, si.offline_trader, ({}) as product_counts, ({}) as sold_product_counts, u.name seller_name, u.profile_image seller_profile_image, si.facebook_profile_image, si.shop_or_page_name, si.facebook_page_image, si.bussiness_phone, si.address, si.nrc, si.nrc_front_image, si.nrc_back_image, si.bank_code, si.bank_account, si.bank_account_image, si.wallet_type, si.wallet_account, si.fee_id, si.monthly_transaction_screenshot from seller_informations si join users u on u.user_id = si.user_id where si.user_id = $3 and si.deleted_at is null", product_counts_query, sold_product_counts_query);
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
            facebook_profile_image: row.get("facebook_profile_image"),
            shop_or_page_name: row.get("shop_or_page_name"),
            facebook_page_image: row.get("facebook_page_image"),
            bussiness_phone: row.get("bussiness_phone"),
            address: row.get("address"),
            nrc: row.get("nrc"),
            nrc_front_image: row.get("nrc_front_image"),
            nrc_back_image: row.get("nrc_back_image"),
            bank_code: row.get("bank_code"),
            bank_account: row.get("bank_account"),
            bank_account_image: row.get("bank_account_image"),
            wallet_type: row.get("wallet_type"),
            wallet_account: row.get("wallet_account"),
            fee_id: row.get("fee_id"),
            monthly_transaction_screenshot: row.get("monthly_transaction_screenshot"),
        }),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}
