use std::collections::HashMap;

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct GoogleTokenInfo {
    pub aud: String,
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}

pub async fn verify_id_token_with_google_api_original(
    id_token: &str,
) -> Result<HashMap<String, Value>, reqwest::Error> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        id_token
    );
    let res = reqwest::get(&url)
        .await?
        .json::<HashMap<String, Value>>()
        .await?;
    Ok(res)
}

pub async fn verify_id_token_with_google_api(
    id_token: &str,
) -> Result<GoogleTokenInfo, reqwest::Error> {
    let url = format!(
        "https://oauth2.googleapis.com/tokeninfo?id_token={}",
        id_token
    );
    let res = reqwest::get(&url).await?.json::<GoogleTokenInfo>().await?;

    Ok(res)
}
