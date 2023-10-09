use reqwest;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

pub async fn send_notification(
    title: &str,
    body: &str,
    fcmtoken: &str,
    data: Option<HashMap<String, Value>>,
) -> Result<Value, reqwest::Error> {
    let firebase_fcm_url = env::var("FIREBASE_FCM_URL")
        .unwrap_or_else(|_| String::from("https://fcm.googleapis.com/fcm/send"));
    let firebase_fcm_auth =
        env::var("FIREBASE_FCM_AUTH").unwrap_or_else(|_| String::from("key=AAAAbqDfgxU:APA91bFyK7L-58Gz9IJfMY_5BIZY5KTDRHKLHz3di5oh_Jab0G730E0OmP3tZgDobMLg82jFWKoVDQtKprqEm8fgN-dh4ZnBG_vJXrQh9tVkx2WmSvuo9oEuDCbLGxBilBLgvKhYVUqw"));

    // Sample data for the notification
    let mut notification = HashMap::new();
    notification.insert("title".to_string(), Value::String(title.to_string()));
    notification.insert("body".to_string(), Value::String(body.to_string()));

    let fcm_data = match data {
        Some(d) => d,
        None => HashMap::new(),
    };
    let client = reqwest::Client::new();
    let response = client
        .post(&firebase_fcm_url)
        .header("Authorization", firebase_fcm_auth)
        .header("Content-Type", "application/json")
        .json(&json!({
            "notification": notification,
            "to": fcmtoken,
            "data": fcm_data
        }))
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}
