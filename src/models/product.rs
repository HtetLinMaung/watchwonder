use std::fs;

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
    vector_finder::add_vector,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub product_id: i32,
    pub model: String,
    pub description: String,
    pub color: String,
    pub strap_material: String,
    pub strap_color: String,
    pub case_material: String,
    pub dial_color: String,
    pub movement_type: String,
    pub water_resistance: String,
    pub warranty_period: String,
    pub dimensions: String,
    pub price: f64,
    pub stock_quantity: i32,
    pub is_top_model: bool,
    pub product_images: Vec<String>,
    pub shop_id: i32,
    pub shop_name: String,
    pub category_id: i32,
    pub category_name: String,
    pub brand_id: i32,
    pub brand_name: String,
    pub currency_id: i32,
    pub currency_code: String,
    pub symbol: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_products(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    shop_id: Option<i32>,
    category_id: Option<i32>,
    brands: &Option<Vec<i32>>,
    models: &Option<Vec<String>>,
    from_price: Option<f64>,
    to_price: Option<f64>,
    is_top_model: Option<bool>,
    products: &Option<Vec<i32>>,
    role: &str,
    user_id: i32,
    client: &Client,
) -> Result<PaginationResult<Product>, Error> {
    let mut base_query = "from products p inner join brands b on b.brand_id = p.brand_id inner join categories c on p.category_id = c.category_id inner join shops s on s.shop_id = p.shop_id inner join currencies cur on cur.currency_id = p.currency_id where p.deleted_at is null and b.deleted_at is null and c.deleted_at is null and s.deleted_at is null and cur.deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    if role == "agent" {
        params.push(Box::new(user_id));
        base_query = format!("{base_query} and p.creator_id = ${}", params.len());
    }

    if let Some(s) = shop_id {
        params.push(Box::new(s));
        base_query = format!("{base_query} and p.shop_id = ${}", params.len());
    }

    if let Some(c) = category_id {
        params.push(Box::new(c));
        base_query = format!("{base_query} and p.category_id = ${}", params.len());
    }

    if let Some(brand_list) = brands {
        if !brand_list.is_empty() {
            if brand_list.len() > 1 {
                let mut placeholders: Vec<String> = vec![];
                for brand in brand_list {
                    params.push(Box::new(brand));
                    placeholders.push(format!("${}", params.len()));
                }
                base_query = format!(
                    "{base_query} and p.brand_id in ({})",
                    placeholders.join(", ")
                );
            } else {
                let brand = brand_list[0];
                params.push(Box::new(brand));
                base_query = format!("{base_query} and p.brand_id = ${}", params.len());
            }
        }
    }

    if let Some(model_list) = models {
        if !model_list.is_empty() {
            if model_list.len() > 1 {
                let mut placeholders: Vec<String> = vec![];
                for model in model_list {
                    params.push(Box::new(model));
                    placeholders.push(format!("${}", params.len()));
                }
                base_query = format!("{base_query} and p.model in ({})", placeholders.join(", "));
            } else {
                let model = model_list[0].clone();
                params.push(Box::new(model));
                base_query = format!("{base_query} and p.model = ${}", params.len());
            }
        }
    }

    if from_price.is_some() && to_price.is_some() {
        base_query = format!(
            "{base_query} and p.price between {} and {}",
            from_price.unwrap(),
            to_price.unwrap()
        );
    }

    if let Some(top_model) = is_top_model {
        params.push(Box::new(top_model));
        base_query = format!("{base_query} and p.is_top_model = ${}", params.len());
    }

    if let Some(product_list) = products {
        if !product_list.is_empty() {
            if product_list.len() > 1 {
                let mut placeholders: Vec<String> = vec![];
                for product_id in product_list {
                    params.push(Box::new(product_id));
                    placeholders.push(format!("${}", params.len()));
                }
                base_query = format!(
                    "{base_query} and p.product_id in ({})",
                    placeholders.join(", ")
                );
            } else {
                let product_id = product_list[0];
                params.push(Box::new(product_id));
                base_query = format!("{base_query} and p.product_id = ${}", params.len());
            }
        }
    }

    let order_options = match role {
        "user" => "p.model asc, p.created_at desc".to_string(),
        _ => "p.created_at desc".to_string(),
    };

    let result=  generate_pagination_query(PaginationOptions {
        select_columns: "p.product_id, b.brand_id, b.name brand_name, p.model, p.description, p.color, p.strap_material, p.strap_color, p.case_material, p.dial_color, p.movement_type, p.water_resistance, p.warranty_period, p.dimensions, p.price::text, p.currency_id, cur.currency_code, cur.symbol, p.stock_quantity, p.is_top_model, c.category_id, c.name category_name, s.shop_id, s.name shop_name, p.created_at",
        base_query: &base_query,
        search_columns: vec!["b.name", "p.model", "p.description", "p.color", "p.strap_material", "p.strap_color", "p.case_material", "p.dial_color", "p.movement_type", "p.water_resistance", "p.warranty_period", "p.dimensions", "b.name", "c.name", "s.name"],
        search: search.as_deref(),
        order_options: Some(&order_options),
        page,
        per_page,
    });

    let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();

    let row = client.query_one(&result.count_query, &params_slice).await?;
    let total: i64 = row.get("total");

    let mut page_counts = 0;
    let mut current_page = 0;
    let mut limit = 0;
    if page.is_some() && per_page.is_some() {
        current_page = page.unwrap();
        limit = per_page.unwrap();
        page_counts = (total as f64 / limit as f64).ceil() as usize;
    }

    let rows = client.query(&result.query, &params_slice[..]).await?;

    let mut products: Vec<Product> = Vec::new();

    for row in &rows {
        let product_id: i32 = row.get("product_id");
        let image_rows = client
            .query(
                "select image_url from product_images where product_id = $1 and deleted_at is null",
                &[&product_id],
            )
            .await?;

        let product_images: Vec<String> = image_rows.iter().map(|r| r.get("image_url")).collect();

        let price: String = row.get("price");
        let price: f64 = price.parse().unwrap();

        products.push(Product {
            product_id: row.get("product_id"),
            model: row.get("model"),
            description: row.get("description"),
            color: row.get("color"),
            strap_material: row.get("strap_material"),
            strap_color: row.get("strap_color"),
            case_material: row.get("case_material"),
            dial_color: row.get("dial_color"),
            movement_type: row.get("movement_type"),
            water_resistance: row.get("water_resistance"),
            warranty_period: row.get("warranty_period"),
            dimensions: row.get("dimensions"),
            price,
            stock_quantity: row.get("stock_quantity"),
            is_top_model: row.get("is_top_model"),
            product_images,
            brand_id: row.get("brand_id"),
            brand_name: row.get("brand_name"),
            category_id: row.get("category_id"),
            category_name: row.get("category_name"),
            shop_id: row.get("shop_id"),
            shop_name: row.get("shop_name"),
            currency_id: row.get("currency_id"),
            currency_code: row.get("currency_code"),
            symbol: row.get("symbol"),
            created_at: row.get("created_at"),
        });
    }

    Ok(PaginationResult {
        data: products,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_models(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    client: &Client,
) -> Result<PaginationResult<String>, Error> {
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    let result = generate_pagination_query(PaginationOptions {
        select_columns: "p.model",
        base_query:
            "from (select model from products where deleted_at is null group by model) as p where 1 = 1",
        search_columns: vec!["p.model"],
        search: search.as_deref(),
        order_options: Some("p.model"),
        page,
        per_page,
    });

    let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();

    let row = client.query_one(&result.count_query, &params_slice).await?;
    let total: i64 = row.get("total");

    let mut page_counts = 0;
    let mut current_page = 0;
    let mut limit = 0;
    if page.is_some() && per_page.is_some() {
        current_page = page.unwrap();
        limit = per_page.unwrap();
        page_counts = (total as f64 / limit as f64).ceil() as usize;
    }

    let models = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| row.get("model"))
        .collect();

    Ok(PaginationResult {
        data: models,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

#[derive(Debug, Deserialize)]
pub struct ProductRequest {
    pub shop_id: i32,
    pub category_id: i32,
    pub brand_id: i32,
    pub model: String,
    pub description: String,
    pub color: String,
    pub strap_material: String,
    pub strap_color: String,
    pub case_material: String,
    pub dial_color: String,
    pub movement_type: String,
    pub water_resistance: String,
    pub warranty_period: String,
    pub dimensions: String,
    pub price: f64,
    pub stock_quantity: i32,
    pub is_top_model: bool,
    pub product_images: Vec<String>,
    pub currency_id: Option<i32>,
}

pub async fn add_product(
    data: &ProductRequest,
    currency_id: i32,
    creator_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = format!("insert into products (shop_id, category_id, brand_id, model, description, color, strap_material, strap_color, case_material, dial_color, movement_type, water_resistance, warranty_period, dimensions, price, stock_quantity, is_top_model, currency_id, creator_id) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, {}, $15, $16, $17, $18) returning product_id", &data.price);
    let result = client
        .query_one(
            &query,
            &[
                &data.shop_id,
                &data.category_id,
                &data.brand_id,
                &data.model,
                &data.description,
                &data.color,
                &data.strap_material,
                &data.strap_color,
                &data.case_material,
                &data.dial_color,
                &data.movement_type,
                &data.water_resistance,
                &data.warranty_period,
                &data.dimensions,
                &data.stock_quantity,
                &data.is_top_model,
                &currency_id,
                &creator_id,
            ],
        )
        .await?;
    let product_id: i32 = result.get("product_id");
    for product_image in &data.product_images {
        client
            .execute(
                "insert into product_images (product_id, image_url) values ($1, $2)",
                &[&product_id, &product_image],
            )
            .await?;
        // Clone necessary data for the async block
        let product_id_clone = product_id.clone();
        let product_image_clone = product_image.clone();

        tokio::spawn(async move {
            match add_vector(
                &product_id_clone.to_string(),
                &product_image_clone.replace("/images", "images"),
            )
            .await
            {
                Ok(response) => {
                    // println!("Vector added successfully.");
                    println!("{:?}", response);
                }
                Err(err) => {
                    println!("Error adding vector: {:?}", err);
                }
            }
        });
    }
    Ok(())
}

pub async fn get_product_by_id(product_id: i32, client: &Client) -> Option<Product> {
    let result = client
        .query_one(
            "select p.product_id, b.brand_id, b.name brand_name, p.model, p.description, p.color, p.strap_material, p.strap_color, p.case_material, p.dial_color, p.movement_type, p.water_resistance, p.warranty_period, p.dimensions, p.price::text, p.currency_id, cur.currency_code, cur.symbol, p.stock_quantity, p.is_top_model, c.category_id, c.name category_name, s.shop_id, s.name shop_name, p.created_at from products p inner join brands b on b.brand_id = p.brand_id inner join categories c on p.category_id = c.category_id inner join shops s on s.shop_id = p.shop_id inner join currencies cur on cur.currency_id = p.currency_id where p.deleted_at is null and b.deleted_at is null and c.deleted_at is null and s.deleted_at is null and cur.deleted_at is null and p.product_id = $1",
            &[&product_id],
        )
        .await;

    let product_images: Vec<String> = match client
        .query(
            "select image_url from product_images where product_id = $1 and deleted_at is null",
            &[&product_id],
        )
        .await
    {
        Ok(image_rows) => image_rows.iter().map(|r| r.get("image_url")).collect(),
        Err(_) => vec![],
    };
    match result {
        Ok(row) => {
            let price: String = row.get("price");
            let price: f64 = price.parse().unwrap();
            Some(Product {
                product_id: row.get("product_id"),
                model: row.get("model"),
                description: row.get("description"),
                color: row.get("color"),
                strap_material: row.get("strap_material"),
                strap_color: row.get("strap_color"),
                case_material: row.get("case_material"),
                dial_color: row.get("dial_color"),
                movement_type: row.get("movement_type"),
                water_resistance: row.get("water_resistance"),
                warranty_period: row.get("warranty_period"),
                dimensions: row.get("dimensions"),
                price,
                stock_quantity: row.get("stock_quantity"),
                is_top_model: row.get("is_top_model"),
                product_images,
                brand_id: row.get("brand_id"),
                brand_name: row.get("brand_name"),
                category_id: row.get("category_id"),
                category_name: row.get("category_name"),
                shop_id: row.get("shop_id"),
                shop_name: row.get("shop_name"),
                currency_id: row.get("currency_id"),
                currency_code: row.get("currency_code"),
                symbol: row.get("symbol"),
                created_at: row.get("created_at"),
            })
        }
        Err(_) => None,
    }
}

pub async fn update_product(
    product_id: i32,
    old_product_images: &Vec<String>,
    data: &ProductRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let query = format!("update products set shop_id = $1, category_id = $2, brand_id = $3, model = $4, description = $5, color = $6, strap_material = $7, strap_color = $8, case_material = $9, dial_color = $10, movement_type = $11, water_resistance = $12, warranty_period = $13, dimensions = $14, price = {}, stock_quantity = $15, is_top_model = $16 where product_id = $17", &data.price);
    client
        .execute(
            &query,
            &[
                &data.shop_id,
                &data.category_id,
                &data.brand_id,
                &data.model,
                &data.description,
                &data.color,
                &data.strap_material,
                &data.strap_color,
                &data.case_material,
                &data.dial_color,
                &data.movement_type,
                &data.water_resistance,
                &data.warranty_period,
                &data.dimensions,
                &data.stock_quantity,
                &data.is_top_model,
                &product_id,
            ],
        )
        .await?;
    client
        .execute(
            "update product_images set deleted_at = CURRENT_TIMESTAMP where product_id = $1",
            &[&product_id],
        )
        .await?;
    for product_image in &data.product_images {
        client
            .execute(
                "insert into product_images (product_id, image_url) values ($1, $2)",
                &[&product_id, &product_image],
            )
            .await?;
        // Clone necessary data for the async block
        let product_id_clone = product_id.clone();
        let product_image_clone = product_image.clone();

        tokio::spawn(async move {
            match add_vector(
                &product_id_clone.to_string(),
                &product_image_clone.replace("/images", "images"),
            )
            .await
            {
                Ok(response) => {
                    // println!("Vector added successfully.");
                    println!("{:?}", response);
                }
                Err(err) => {
                    println!("Error adding vector: {:?}", err);
                }
            }
        });
    }
    for old_product_image in old_product_images {
        if !data.product_images.contains(old_product_image) {
            match fs::remove_file(old_product_image) {
                Ok(_) => println!("File deleted successfully!"),
                Err(e) => println!("Error deleting file: {}", e),
            };
        }
    }
    Ok(())
}

pub async fn delete_product(
    product_id: i32,
    old_product_images: &Vec<String>,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update products set deleted_at = CURRENT_TIMESTAMP where product_id = $1",
            &[&product_id],
        )
        .await?;
    client
        .execute(
            "update product_images set deleted_at = CURRENT_TIMESTAMP where product_id = $1",
            &[&product_id],
        )
        .await?;
    for old_product_image in old_product_images {
        match fs::remove_file(old_product_image) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
    }
    Ok(())
}

#[derive(Serialize)]
pub struct ProductAndShopName {
    pub product_name: String,
    pub shop_name: String,
}

pub async fn get_product_and_shop_names(
    product_id_list: &Vec<i32>,
    client: &Client,
) -> Result<Vec<ProductAndShopName>, Box<dyn std::error::Error>> {
    if product_id_list.is_empty() {
        return Ok(vec![]);
    }
    let query = format!("select (b.name || ' ' || p.model) as product_name, s.name shop_name from products p inner join brands b on b.brand_id = p.brand_id inner join shops s on s.shop_id = p.shop_id where p.product_id in ({}) and p.deleted_at is null and s.deleted_at is null and b.deleted_at is null", product_id_list.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(", "));
    Ok(client
        .query(&query, &[])
        .await?
        .iter()
        .map(|row| ProductAndShopName {
            product_name: row.get("product_name"),
            shop_name: row.get("shop_name"),
        })
        .collect())
}

pub async fn get_recommended_products_for_product(
    product_id: i32,
    client: &Client,
) -> Result<Vec<i32>, Error> {
    let query = "
        WITH UsersWhoBoughtThisProduct AS (
            SELECT o.user_id
            FROM orders o
            JOIN order_items oi ON o.order_id = oi.order_id
            WHERE oi.product_id = $1
        ),
        ProductsBoughtByTheseUsers AS (
            SELECT oi.product_id, COUNT(DISTINCT o.user_id) as user_count
            FROM orders o
            JOIN order_items oi ON o.order_id = oi.order_id
            WHERE o.user_id IN (SELECT user_id FROM UsersWhoBoughtThisProduct)
            AND oi.product_id <> $1
            GROUP BY oi.product_id
        )
        SELECT product_id
        FROM ProductsBoughtByTheseUsers
        ORDER BY user_count DESC
        LIMIT 10;
    ";

    let rows = client.query(query, &[&product_id]).await?;
    let product_ids: Vec<i32> = rows.iter().map(|row| row.get(0)).collect();

    Ok(product_ids)
}

// pub async fn get_top_products_for_user(client: &Client, user_id: i32) -> Result<Vec<i32>, Error> {
//     let query = "
//         SELECT product_id
//         FROM order_items
//         WHERE order_id IN (SELECT order_id FROM orders WHERE user_id = $1)
//         GROUP BY product_id
//         ORDER BY COUNT(*) DESC
//         LIMIT 10;
//     ";

//     let rows = client.query(query, &[&user_id]).await?;
//     let product_ids: Vec<i32> = rows.iter().map(|row| row.get(0)).collect();

//     Ok(product_ids)
// }

pub async fn get_recommended_products_for_user(
    user_id: i32,
    client: &Client,
) -> Result<Vec<i32>, Error> {
    let query = "
        WITH UserProducts AS (
            SELECT product_id
            FROM order_items
            WHERE order_id IN (SELECT order_id FROM orders WHERE user_id = $1)
        ),
        SimilarUsers AS (
            SELECT o.user_id
            FROM orders o
            JOIN order_items oi ON o.order_id = oi.order_id
            WHERE oi.product_id IN (SELECT product_id FROM UserProducts)
            AND o.user_id <> $1
        ),
        RecommendedProducts AS (
            SELECT oi.product_id, COUNT(DISTINCT o.user_id) as user_count
            FROM orders o
            JOIN order_items oi ON o.order_id = oi.order_id
            WHERE o.user_id IN (SELECT user_id FROM SimilarUsers)
            AND oi.product_id NOT IN (SELECT product_id FROM UserProducts)
            GROUP BY oi.product_id
        )
        SELECT product_id
        FROM RecommendedProducts
        ORDER BY user_count DESC
        LIMIT 10;
    ";

    let rows = client.query(query, &[&user_id]).await?;
    let product_ids: Vec<i32> = rows.iter().map(|row| row.get(0)).collect();

    Ok(product_ids)
}

pub async fn get_product_creator_id(product_id: i32, client: &Client) -> Option<i32> {
    match client
        .query_one(
            "select creator_id from products where product_id = $1 and creator_id is not null",
            &[&product_id],
        )
        .await
    {
        Ok(row) => Some(row.get("creator_id")),
        Err(_) => None,
    }
}
