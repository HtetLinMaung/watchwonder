use reqwest;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

pub async fn send_notification(
    notification: HashMap<String, Value>,
    fcmtoken: &str,
    data: HashMap<String, Value>,
) -> Result<Value, reqwest::Error> {
    let firebase_fcm_url =
        env::var("FIREBASE_FCM_URL").unwrap_or_else(|_| String::from("YOUR_DEFAULT_URL"));
    let firebase_fcm_auth =
        env::var("FIREBASE_FCM_AUTH").unwrap_or_else(|_| String::from("key=YOUR_DEFAULT_KEY"));

    let client = reqwest::Client::new();
    let response = client
        .post(&firebase_fcm_url)
        .header("Authorization", firebase_fcm_auth)
        .header("Content-Type", "application/json")
        .json(&json!({
            "notification": notification,
            "to": fcmtoken,
            "data": data
        }))
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}
