use reqwest;
use serde_json::{json, Value};
use std::env;

pub async fn add_vector(label: &str, image_path: &str) -> Result<Value, reqwest::Error> {
    let pyvecfinder_url =
        env::var("PYVECFINDER_URL").unwrap_or_else(|_| String::from("http://pyvecfinder:5000"));

    let url = format!("{pyvecfinder_url}/vec-finder/add-vector");

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&json!({
            "label": label,
            "image_path": image_path,
        }))
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}

pub async fn search_vectors(image_path: &str) -> Result<Value, reqwest::Error> {
    let pyvecfinder_url =
        env::var("PYVECFINDER_URL").unwrap_or_else(|_| String::from("http://pyvecfinder:5000"));

    let url = format!("{pyvecfinder_url}/vec-finder/search");

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&json!({
            "image_path": image_path,
        }))
        .send()
        .await?
        .json()
        .await?;

    Ok(response)
}
