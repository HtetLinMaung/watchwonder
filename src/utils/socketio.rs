use reqwest;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

pub async fn emit(
    event: &str,
    rooms: &Vec<i32>,
    payload: Option<HashMap<String, Value>>,
) -> Result<Value, reqwest::Error> {
    let instant_io_url = env::var("INSTANT_IO_URL")
        .unwrap_or_else(|_| String::from("http://localhost:3000/instantio/emit"));

    let payload = match payload {
        Some(p) => p,
        None => HashMap::new(),
    };

    println!("instant_io_url: {instant_io_url}");
    println!("event: {event}");
    println!("rooms: {:?}", rooms);
    println!("payload: {:?}", payload);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // Only if you're sure about the security implications
        .build()?;
    let response = client
        .post(&instant_io_url)
        .header("Content-Type", "application/json")
        .json(&json!({
            "event": event,
            "rooms": rooms,
            "payload": payload
        }))
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", response);

    Ok(response)
}
