use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, Error};

#[derive(Serialize, Deserialize)]
pub struct Fcm {
    pub token: String,
    pub device_type: String,
}

pub async fn add_fcm_token(user_id: i32, fcm: &Fcm, client: &Client) -> Result<(), Error> {
    client.execute("insert into fcm_tokens (user_id, token, device_type) 
    values ($1, $2, $3) 
    on conflict (user_id, device_type)
    do update set user_id = excluded.user_id, token = excluded.token, device_type = excluded.device_type", &[&user_id, &fcm.token, &fcm.device_type]).await?;
    Ok(())
}

pub async fn get_fcm_tokens(user_id: i32, client: &Client) -> Result<Vec<String>, Error> {
    let rows = client
        .query(
            "select token from fcm_tokens where user_id = $1",
            &[&user_id],
        )
        .await?;
    Ok(rows.iter().map(|row| row.get("token")).collect())
}

pub async fn get_admin_fcm_tokens(client: &Client) -> Result<Vec<String>, Error> {
    let rows = client
        .query(
            "select f.token from fcm_tokens f inner join users u on u.user_id = f.user_id where u.role = 'admin' and u.deleted_at is null",
            &[],
        )
        .await?;
    Ok(rows.iter().map(|row| row.get("token")).collect())
}
