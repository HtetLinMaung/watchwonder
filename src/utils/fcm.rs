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
        env::var("FIREBASE_FCM_AUTH").unwrap_or_else(|_| String::from("key=AAAAP89u1s0:APA91bHf2dDH0XrJt1u71o8UrNsmOt57A4TJhQzj_MtSygfHoBJ_6VXvjriacwhcNeLSHp4Ix947YmtZO_f2IwJL_9zqU2UkKH6gSzbpJ86YXXghiCfLoLpJ9Iz4Hsj8SMQ8XhjrcAC1"));

    // Sample data for the notification
    let mut notification = HashMap::new();
    notification.insert("title".to_string(), Value::String(title.to_string()));
    notification.insert("body".to_string(), Value::String(body.to_string()));

    let fcm_data = match data {
        Some(d) => d,
        None => HashMap::new(),
    };
    println!("firebase_fcm_url: {firebase_fcm_url}");
    println!("firebase_fcm_auth: {firebase_fcm_auth}");
    println!("notification: {:?}", notification);
    println!("to: {fcmtoken}");
    println!("data: {:?}", fcm_data);
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // Only if you're sure about the security implications
        .build()?;
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
    println!("{:?}", response);

    Ok(response)
}
