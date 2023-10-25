use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct SellerInformation {
    pub company_name: String,
    pub professional_title: String,
    pub active_since_year: i32,
    pub location: String,
    pub offline_trader: bool,
}

#[derive(Deserialize, Debug)]
pub struct SellerInformationRequest {
    pub company_name: String,
    pub professional_title: String,
    pub location: String,
    pub offline_trader: bool,
}

pub async fn get_seller_information(user_id: i32, client: &Client) -> Option<SellerInformation> {
    match client.query_one("select company_name, professional_title, active_since_year, location, offline_trader from seller_informations where user_id = $1 and deleted_at is null", &[&user_id]).await {
        Ok(row) => Some(SellerInformation {
    company_name: row.get("company_name"),
    professional_title: row.get("professional_title"),
    active_since_year: row.get("active_since_year"),
    location: row.get("location"),
    offline_trader: row.get("offline_trader"),
}),
Err(err) => {
    println!("{:?}", err);
    None
}
  }
}
