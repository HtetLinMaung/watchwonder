extern crate dotenv;

use std::{env, sync::Arc};

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use tokio_postgres::NoTls;

mod api;
mod models;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let port: u16 = env::var("PORT")
        .unwrap_or(String::from("8080"))
        .parse()
        .expect("Port must be number");
    let conn = env::var("DB_CONNECTION").expect("DB_CONNECTION must be set");
    let (client, connection) = tokio_postgres::connect(conn.as_str(), NoTls).await.unwrap();
    let client = Arc::new(client);

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    HttpServer::new(move || {
        if !std::fs::metadata("./images").is_ok() {
            if let Err(err) = std::fs::create_dir_all("./images") {
                println!("{:?}", err);
            }
        }
        if !std::fs::metadata("./products").is_ok() {
            if let Err(err) = std::fs::create_dir_all("./products") {
                println!("{:?}", err);
            }
        }
        // let default_size = env::var("DEFAULT_REQUEST_SIZE")
        //     .unwrap_or_else(|_| "2097152".to_string())
        //     .parse::<usize>()
        //     .unwrap_or(2097152);
        let cors = Cors::permissive() // This allows all origins. Be careful with this in production!
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(client.clone()))
            .configure(api::init)
            .service(fs::Files::new("/images", "./images").show_files_listing())
            .service(fs::Files::new("/products", "./products").show_files_listing())
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
