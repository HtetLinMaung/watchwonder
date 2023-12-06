use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, Error};

#[derive(Deserialize)]
pub struct SellerReviewRequest {
    pub shop_id: i32,
    pub rating: f64,
    pub comment: String,
}

pub async fn add_review(
    data: &SellerReviewRequest,
    user_id: i32,
    client: &Client,
) -> Result<(), Error> {
    match client.query_one("select review_id from seller_reviews where user_id = $1 and shop_id = $2 and deleted_at is null", &[&user_id, &data.shop_id]).await {
        Ok(row) => {
            let review_id: i32 = row.get("review_id");
            let query = format!("update seller_reviews set rating = {}, comment = $1 where review_id = $2 and deleted_at is null", &data.rating);
            client.execute(&query, &[&data.comment, &review_id]).await?;
        },
        Err(_) => {
           let query = format!("insert into seller_reviews (shop_id, user_id, rating, comment) values ($1, $2, {}, $3)", &data.rating);
            client.execute(&query, &[&data.shop_id, &user_id, &data.comment]).await?;
        }
    };
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SellerReview {
    pub review_id: i32,
    pub name: String,
    pub profile_image: String,
    pub rating: f64,
    pub comment: String,
    pub review_date: NaiveDateTime,
}

pub async fn get_seller_reviews(shop_id: i32, client: &Client) -> Result<Vec<SellerReview>, Error> {
    let rows= client.query(
        "select r.review_id, u.name, u.profile_image, r.rating::text, r.comment, r.review_date from seller_reviews r inner join users u on u.user_id = r.user_id where r.deleted_at is null and u.deleted_at is null and r.shop_id = $1",
        &[&shop_id],
    ).await?;
    Ok(rows
        .iter()
        .map(|row| {
            let rating: String = row.get("rating");
            let rating: f64 = rating.parse().unwrap();
            SellerReview {
                review_id: row.get("review_id"),
                name: row.get("name"),
                profile_image: row.get("profile_image"),
                rating,
                comment: row.get("comment"),
                review_date: row.get("review_date"),
            }
        })
        .collect())
}

pub async fn is_user_already_review(shop_id: i32, user_id: i32, client: &Client) -> bool {
    match client.query_one("select count(*) as total from seller_reviews where user_id = $1 and shop_id = $2 and deleted_at is null", &[&user_id, &shop_id]).await {
    Ok(row) => {
        let total: i64 = row.get("total");
        total > 0
    },
    Err(err) => {
        println!("{:?}", err);
        false
    }
   }
}
