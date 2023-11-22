use reqwest;
use serde_json::{json, Value};
use std::env;

pub async fn site_to_pdf(content: &str) -> Result<Value, reqwest::Error> {
    let report_forge_url = env::var("REPORT_FORGE_URL")
        .unwrap_or_else(|_| String::from("http://localhost:3000/api/site-to-pdf"));

    println!("report_forge_url: {report_forge_url}");
    println!("content: {content}");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true) // Only if you're sure about the security implications
        .build()?;
    let response = client
        .post(&report_forge_url)
        .header("Content-Type", "application/json")
        .json(&json!({
            "content": content,
            "image": true,
        }))
        .send()
        .await?
        .json()
        .await?;
    println!("{:?}", response);
    Ok(response)
}
