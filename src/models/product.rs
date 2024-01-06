use std::{fs, path::Path};

use crate::utils::{
    common_struct::PaginationResult,
    setting::{get_demo_platform, get_demo_user_id, get_min_demo_version},
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
    pub discount_percent: f64,
    pub discounted_price: f64,
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
    pub condition: String,
    pub warranty_type_id: i32,
    pub warranty_type_description: String,
    pub dial_glass_type_id: i32,
    pub dial_glass_type_description: String,
    pub other_accessories_type_id: i32,
    pub other_accessories_type_description: String,
    pub gender_id: i32,
    pub gender_description: String,
    pub waiting_time: String,
    pub case_diameter: String,
    pub case_depth: String,
    pub case_width: String,
    pub movement_caliber: String,
    pub movement_country: String,
    pub is_preorder: bool,
    pub creator_id: i32,
    pub discount_expiration: Option<NaiveDateTime>,
    pub discount_reason: String,
    pub discount_type: String,
    pub discount_updated_by: String,
    pub level: i32,
    pub created_at: NaiveDateTime,
}

pub async fn get_products(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    platform: &str,
    shop_id: Option<i32>,
    category_id: Option<i32>,
    brands: &Option<Vec<i32>>,
    models: &Option<Vec<String>>,
    from_price: Option<f64>,
    to_price: Option<f64>,
    is_top_model: Option<bool>,
    products: &Option<Vec<i32>>,
    view: &Option<String>,
    role: &str,
    user_id: i32,
    version: i32,
    client: &Client,
) -> Result<PaginationResult<Product>, Error> {
    let mut base_query = "from products p inner join brands b on b.brand_id = p.brand_id inner join categories c on p.category_id = c.category_id inner join shops s on s.shop_id = p.shop_id inner join currencies cur on cur.currency_id = p.currency_id inner join warranty_types wt on wt.warranty_type_id = p.warranty_type_id inner join dial_glass_types dgt on dgt.dial_glass_type_id = p.dial_glass_type_id inner join other_accessories_types oat on oat.other_accessories_type_id = p.other_accessories_type_id inner join genders g on g.gender_id = p.gender_id where p.deleted_at is null and b.deleted_at is null and c.deleted_at is null and s.deleted_at is null and cur.deleted_at is null and wt.deleted_at is null and dgt.deleted_at is null and oat.deleted_at is null and g.deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    let mut screen_view = "admin";
    if let Some(v) = view {
        screen_view = v.as_str();
    }

    if role == "agent" {
        params.push(Box::new(user_id));
        if screen_view == "user" {
            base_query = format!("{base_query} and p.creator_id != ${}", params.len());
        } else {
            base_query = format!("{base_query} and p.creator_id = ${}", params.len());
        }
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
            "{base_query} and (p.price - (p.price * p.discount_percent / 100)) between {} and {}",
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

    let demo_user_id = get_demo_user_id();
    let min_demo_version = get_min_demo_version();
    if platform == get_demo_platform().as_str()
        || platform == "ios" && version >= min_demo_version
        || (demo_user_id > 0 && user_id == demo_user_id)
    {
        base_query = format!("{base_query} and p.is_demo = true");
    } else {
        base_query = format!("{base_query} and p.is_demo = false");
    }

    let order_options = if role == "user" || (role == "agent" && screen_view == "user") {
        "p.level desc, p.model asc, p.created_at desc".to_string()
    } else {
        "p.created_at desc".to_string()
    };

    let result=  generate_pagination_query(PaginationOptions {
        select_columns: "p.product_id, b.brand_id, b.name brand_name, p.model, p.description, p.color, p.strap_material, p.strap_color, p.case_material, p.dial_color, p.movement_type, p.water_resistance, p.warranty_period, p.dimensions, p.price::text, p.discount_percent::text, case when p.discount_type = 'No Discount' then p.price::text when p.discount_type = 'Discount by Specific Amount' and (p.discount_expiration is null or now()::timestamp < p.discount_expiration) then p.discounted_price::text else case when p.discount_expiration is null then (p.price - (p.price * p.discount_percent / 100))::text when now()::timestamp >= p.discount_expiration then p.price::text else (p.price - (p.price * p.discount_percent / 100))::text end end as discounted_price, p.currency_id, cur.currency_code, cur.symbol, p.stock_quantity, p.is_top_model, c.category_id, c.name category_name, s.shop_id, s.name shop_name, p.condition, p.warranty_type_id, wt.description warranty_type_description, p.dial_glass_type_id, dgt.description dial_glass_type_description, p.other_accessories_type_id, oat.description other_accessories_type_description, p.gender_id, g.description gender_description, p.waiting_time, p.case_diameter, p.case_depth, p.case_width, p.movement_caliber, p.movement_country, p.is_preorder, coalesce(p.creator_id, 0) as creator_id, p.discount_expiration, p.discount_reason, p.discount_type, p.discount_updated_by, p.level, p.created_at",
        base_query: &base_query,
        search_columns: vec!["b.name", "p.model", "p.description", "p.color", "p.strap_material", "p.strap_color", "p.case_material", "p.dial_color", "p.movement_type", "p.water_resistance", "p.warranty_period", "p.dimensions", "b.name", "c.name", "s.name", "p.condition", "wt.description", "dgt.description", "oat.description", "g.description", "p.movement_caliber", "p.movement_country"],
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

        let discount_percent: String = row.get("discount_percent");
        let discount_percent = discount_percent.parse().unwrap();

        let discounted_price: String = row.get("discounted_price");
        let discounted_price = discounted_price.parse().unwrap();

        products.push(Product {
            product_id,
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
            discount_percent,
            discounted_price,
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
            condition: row.get("condition"),
            warranty_type_id: row.get("warranty_type_id"),
            warranty_type_description: row.get("warranty_type_description"),
            dial_glass_type_id: row.get("dial_glass_type_id"),
            dial_glass_type_description: row.get("dial_glass_type_description"),
            other_accessories_type_id: row.get("other_accessories_type_id"),
            other_accessories_type_description: row.get("other_accessories_type_description"),
            gender_id: row.get("gender_id"),
            gender_description: row.get("gender_description"),
            waiting_time: row.get("waiting_time"),
            case_diameter: row.get("case_diameter"),
            case_depth: row.get("case_depth"),
            case_width: row.get("case_width"),
            movement_caliber: row.get("movement_caliber"),
            movement_country: row.get("movement_country"),
            is_preorder: row.get("is_preorder"),
            creator_id: row.get("creator_id"),
            discount_expiration: row.get("discount_expiration"),
            discount_reason: row.get("discount_reason"),
            discount_type: row.get("discount_type"),
            discount_updated_by: row.get("discount_updated_by"),
            level: row.get("level"),
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
    pub discount_percent: Option<f64>,
    pub stock_quantity: i32,
    pub is_top_model: bool,
    pub product_images: Vec<String>,
    pub currency_id: Option<i32>,
    pub condition: Option<String>,
    pub warranty_type_id: Option<i32>,
    pub dial_glass_type_id: Option<i32>,
    pub other_accessories_type_id: Option<i32>,
    pub gender_id: Option<i32>,
    pub waiting_time: Option<String>,
    pub case_diameter: Option<String>,
    pub case_depth: Option<String>,
    pub case_width: Option<String>,
    pub is_preorder: Option<bool>,
    pub movement_caliber: Option<String>,
    pub movement_country: Option<String>,
    pub creator_id: Option<i32>,
    pub discount_expiration: Option<String>,
    pub discount_reason: Option<String>,
    pub discount_type: Option<String>,
    pub discounted_price: Option<f64>,
    pub level: Option<i32>,
}

pub async fn add_product(
    data: &ProductRequest,
    currency_id: i32,
    creator_id: i32,
    role: &str,
    client: &Client,
) -> Result<i32, Error> {
    let mut condition = "".to_string();
    if let Some(c) = &data.condition {
        condition = c.to_string();
    }
    let mut warranty_type_id = 1;
    if let Some(wt_id) = data.warranty_type_id {
        warranty_type_id = wt_id;
    }
    let mut dial_glass_type_id = 1;
    if let Some(dgt_id) = data.dial_glass_type_id {
        dial_glass_type_id = dgt_id;
    }
    let mut other_accessories_type_id = 1;
    if let Some(oat_id) = data.other_accessories_type_id {
        other_accessories_type_id = oat_id;
    }
    let mut gender_id = 1;
    if let Some(g_id) = data.gender_id {
        gender_id = g_id;
    }
    let mut waiting_time = "".to_string();
    if let Some(wt) = &data.waiting_time {
        waiting_time = wt.to_string();
    }
    let mut case_diameter = "".to_string();
    if let Some(cd) = &data.case_diameter {
        case_diameter = cd.to_string();
    }
    let mut case_depth = "".to_string();
    if let Some(cd) = &data.case_depth {
        case_depth = cd.to_string();
    }
    let mut case_width = "".to_string();
    if let Some(cw) = &data.case_width {
        case_width = cw.to_string();
    }
    let mut is_preorder = false;
    if let Some(yes) = data.is_preorder {
        is_preorder = yes;
    }
    let mut movement_caliber = "".to_string();
    if let Some(mc) = &data.movement_caliber {
        movement_caliber = mc.to_string();
    }
    let mut movement_country = "".to_string();
    if let Some(mc) = &data.movement_country {
        movement_country = mc.to_string();
    }
    let mut discount_percent: f64 = 0.0;
    if let Some(dp) = data.discount_percent {
        discount_percent = dp;
    }
    let discount_reason: &str = if let Some(dr) = &data.discount_reason {
        dr
    } else {
        ""
    };
    let discount_expiration = if let Some(de) = &data.discount_expiration {
        format!("'{de}'")
    } else {
        "null".to_string()
    };
    let discount_type = if let Some(dt) = &data.discount_type {
        dt
    } else {
        "Discount by Specific Percentage"
    };
    let discounted_price = if let Some(dp) = data.discounted_price {
        dp
    } else {
        0.0
    };
    let discount_updated_by = if discount_type == "No Discount" {
        "none"
    } else {
        "product"
    };

    let level = if let Some(l) = data.level {
        if role == "admin" {
            l
        } else {
            0
        }
    } else {
        0
    };
    let query = format!("insert into products (shop_id, category_id, brand_id, model, description, color, strap_material, strap_color, case_material, dial_color, movement_type, water_resistance, warranty_period, dimensions, price, discount_percent, stock_quantity, is_top_model, currency_id, condition, warranty_type_id, dial_glass_type_id, other_accessories_type_id, gender_id, waiting_time, case_diameter, case_depth, case_width, movement_caliber, movement_country, is_preorder, creator_id, discount_expiration, discount_reason, discount_type, discounted_price, discount_updated_by, level) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, {}, {}, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, {discount_expiration}, $31, $32, {discounted_price}, '{discount_updated_by}', $33) returning product_id", &data.price, &discount_percent);
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
                &condition,
                &warranty_type_id,
                &dial_glass_type_id,
                &other_accessories_type_id,
                &gender_id,
                &waiting_time,
                &case_diameter,
                &case_depth,
                &case_width,
                &movement_caliber,
                &movement_country,
                &is_preorder,
                &creator_id,
                &discount_reason,
                &discount_type,
                &level,
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
    Ok(product_id)
}

pub async fn get_product_by_id(product_id: i32, client: &Client) -> Option<Product> {
    let result = client
        .query_one(
            "select p.product_id, b.brand_id, b.name brand_name, p.model, p.description, p.color, p.strap_material, p.strap_color, p.case_material, p.dial_color, p.movement_type, p.water_resistance, p.warranty_period, p.dimensions, p.price::text, p.discount_percent::text, case when p.discount_type = 'No Discount' then p.price::text when p.discount_type = 'Discount by Specific Amount' and (p.discount_expiration is null or now()::timestamp < p.discount_expiration) then p.discounted_price::text else case when p.discount_expiration is null then (p.price - (p.price * p.discount_percent / 100))::text when now()::timestamp >= p.discount_expiration then p.price::text else (p.price - (p.price * p.discount_percent / 100))::text end end as discounted_price, p.currency_id, cur.currency_code, cur.symbol, p.stock_quantity, p.is_top_model, c.category_id, c.name category_name, s.shop_id, s.name shop_name, p.condition, p.warranty_type_id, wt.description warranty_type_description, p.dial_glass_type_id, dgt.description dial_glass_type_description, p.other_accessories_type_id, oat.description other_accessories_type_description, p.gender_id, g.description gender_description, p.waiting_time, p.case_diameter, p.case_depth, p.case_width, p.movement_caliber, p.movement_country, p.is_preorder, coalesce(p.creator_id, 0) as creator_id, p.discount_expiration, p.discount_reason, p.discount_type, p.discount_updated_by, p.level, p.created_at from products p inner join brands b on b.brand_id = p.brand_id inner join categories c on p.category_id = c.category_id inner join shops s on s.shop_id = p.shop_id inner join currencies cur on cur.currency_id = p.currency_id inner join warranty_types wt on wt.warranty_type_id = p.warranty_type_id inner join dial_glass_types dgt on dgt.dial_glass_type_id = p.dial_glass_type_id inner join other_accessories_types oat on oat.other_accessories_type_id = p.other_accessories_type_id inner join genders g on g.gender_id = p.gender_id where p.deleted_at is null and b.deleted_at is null and c.deleted_at is null and s.deleted_at is null and cur.deleted_at is null and wt.deleted_at is null and dgt.deleted_at is null and oat.deleted_at is null and g.deleted_at is null and p.product_id = $1",
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

            let discount_percent: String = row.get("discount_percent");
            let discount_percent = discount_percent.parse().unwrap();

            let discounted_price: String = row.get("discounted_price");
            let discounted_price = discounted_price.parse().unwrap();

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
                discount_percent,
                discounted_price,
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
                condition: row.get("condition"),
                warranty_type_description: row.get("warranty_type_description"),
                warranty_type_id: row.get("warranty_type_id"),
                dial_glass_type_id: row.get("dial_glass_type_id"),
                dial_glass_type_description: row.get("dial_glass_type_description"),
                other_accessories_type_id: row.get("other_accessories_type_id"),
                other_accessories_type_description: row.get("other_accessories_type_description"),
                gender_id: row.get("gender_id"),
                gender_description: row.get("gender_description"),
                waiting_time: row.get("waiting_time"),
                case_diameter: row.get("case_diameter"),
                case_depth: row.get("case_depth"),
                case_width: row.get("case_width"),
                movement_caliber: row.get("movement_caliber"),
                movement_country: row.get("movement_country"),
                is_preorder: row.get("is_preorder"),
                creator_id: row.get("creator_id"),
                discount_expiration: row.get("discount_expiration"),
                discount_reason: row.get("discount_reason"),
                discount_type: row.get("discount_type"),
                discount_updated_by: row.get("discount_updated_by"),
                level: row.get("level"),
                created_at: row.get("created_at"),
            })
        }
        Err(err) => {
            println!("err: {:?}", err);
            None
        }
    }
}

pub async fn update_product(
    product_id: i32,
    data: &ProductRequest,
    currency_id: i32,
    old_product: &Product,
    role: &str,
    client: &Client,
) -> Result<(), Error> {
    let mut condition = "".to_string();
    if let Some(c) = &data.condition {
        condition = c.to_string();
    }
    let mut warranty_type_id = 1;
    if let Some(wt_id) = data.warranty_type_id {
        warranty_type_id = wt_id;
    }
    let mut dial_glass_type_id = 1;
    if let Some(dgt_id) = data.dial_glass_type_id {
        dial_glass_type_id = dgt_id;
    }
    let mut other_accessories_type_id = 1;
    if let Some(oat_id) = data.other_accessories_type_id {
        other_accessories_type_id = oat_id;
    }
    let mut gender_id = 1;
    if let Some(g_id) = data.gender_id {
        gender_id = g_id;
    }
    let mut waiting_time = "".to_string();
    if let Some(wt) = &data.waiting_time {
        waiting_time = wt.to_string();
    }
    let mut case_diameter = "".to_string();
    if let Some(cd) = &data.case_diameter {
        case_diameter = cd.to_string();
    }
    let mut case_depth = "".to_string();
    if let Some(cd) = &data.case_depth {
        case_depth = cd.to_string();
    }
    let mut case_width = "".to_string();
    if let Some(cw) = &data.case_width {
        case_width = cw.to_string();
    }
    let mut is_preorder = false;
    if let Some(yes) = data.is_preorder {
        is_preorder = yes;
    }
    let mut movement_caliber = "".to_string();
    if let Some(mc) = &data.movement_caliber {
        movement_caliber = mc.to_string();
    }
    let mut movement_country = "".to_string();
    if let Some(mc) = &data.movement_country {
        movement_country = mc.to_string();
    }
    let mut discount_percent: f64 = 0.0;
    if let Some(dp) = data.discount_percent {
        discount_percent = dp;
    }
    let discount_reason: &str = if let Some(dr) = &data.discount_reason {
        dr
    } else {
        ""
    };
    let discount_expiration = if let Some(de) = &data.discount_expiration {
        format!("'{de}'")
    } else {
        "null".to_string()
    };
    let discount_type = if let Some(dt) = &data.discount_type {
        dt
    } else {
        "Discount by Specific Percentage"
    };
    let discounted_price = if let Some(dp) = data.discounted_price {
        dp
    } else {
        0.0
    };

    // println!(
    //     "old expiration: {}",
    //     old_product.discount_expiration.unwrap().date().to_string()
    // );
    // println!("new expiration: {}", discount_expiration.replace("'", ""));
    let old_discount_expiration = if let Some(de) = old_product.discount_expiration {
        de.date().to_string()
    } else {
        "null".to_string()
    };
    let is_same_expiration = (old_product.discount_expiration.is_none()
        && data.discount_expiration.is_none())
        || (old_discount_expiration == discount_expiration.replace("'", ""));

    let is_same_discounted_price = if old_product.discount_type == discount_type
        && discount_type == "Discount by Specific Amount"
    {
        if old_product.discounted_price == discounted_price {
            true
        } else {
            false
        }
    } else {
        true
    };

    // println!("is_same_expiration: {}", is_same_expiration);
    // println!(
    //     "old_product.discount_percent == discount_percent: {}",
    //     old_product.discount_percent == discount_percent
    // );
    // println!(
    //     "old_product.discount_reason == discount_reason: {}",
    //     old_product.discount_reason == discount_reason
    // );
    // println!("is_same_discounted_price: {}", is_same_discounted_price);
    // println!(
    //     "old_product.discount_type == discount_type: {}",
    //     old_product.discount_type == discount_type
    // );

    let discount_updated_by = if discount_type == "No Discount" {
        "none"
    } else if old_product.discount_percent == discount_percent
        && old_product.discount_reason == discount_reason
        && is_same_discounted_price
        && old_product.discount_type == discount_type
        && is_same_expiration
    {
        &old_product.discount_updated_by
    } else {
        "product"
    };

    let level = if let Some(l) = data.level {
        if role == "admin" {
            l
        } else {
            old_product.level
        }
    } else {
        old_product.level
    };
    // println!("discount_updated_by: {discount_updated_by}");
    let query = format!("update products set shop_id = $1, category_id = $2, brand_id = $3, model = $4, description = $5, color = $6, strap_material = $7, strap_color = $8, case_material = $9, dial_color = $10, movement_type = $11, water_resistance = $12, warranty_period = $13, dimensions = $14, price = {}, discount_percent = {}, stock_quantity = $15, is_top_model = $16, currency_id = $17, condition = $18, warranty_type_id = $19, dial_glass_type_id = $20, other_accessories_type_id = $21, gender_id = $22, waiting_time = $23, case_diameter = $24, case_depth = $25, case_width = $26, is_preorder = $27, movement_caliber = $28, movement_country = $29, discount_expiration = {discount_expiration}, discount_reason = $30, discount_type = $31, discounted_price = {discounted_price}, discount_updated_by = '{discount_updated_by}', level = $32 where product_id = $33", &data.price, &discount_percent);
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
                &currency_id,
                &condition,
                &warranty_type_id,
                &dial_glass_type_id,
                &other_accessories_type_id,
                &gender_id,
                &waiting_time,
                &case_diameter,
                &case_depth,
                &case_width,
                &is_preorder,
                &movement_caliber,
                &movement_country,
                &discount_reason,
                &discount_type,
                &level,
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
    for old_product_image in &old_product.product_images {
        if !data.product_images.contains(old_product_image) {
            match fs::remove_file(old_product_image) {
                Ok(_) => println!("File deleted successfully!"),
                Err(e) => println!("Error deleting file: {}", e),
            };
            let path = Path::new(&old_product_image);
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            let extension = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            match fs::remove_file(format!("{stem}_original.{extension}")) {
                Ok(_) => println!("Original file deleted successfully!"),
                Err(e) => println!("Error deleting original file: {}", e),
            };
        }
    }
    Ok(())
}

pub async fn delete_product(
    product_id: i32,
    old_product_images: &Vec<String>,
    client: &Client,
) -> Result<(), Error> {
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
        let path = Path::new(&old_product_image);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        match fs::remove_file(format!("{stem}_original.{extension}")) {
            Ok(_) => println!("Original file deleted successfully!"),
            Err(e) => println!("Error deleting original file: {}", e),
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
) -> Result<Vec<ProductAndShopName>, Error> {
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

pub async fn get_product_creator_id_from_order_id(order_id: i32, client: &Client) -> i32 {
    match  client.query_one("select p.creator_id from products p join order_items oi on oi.product_id = p.product_id where oi.order_id = $1 and deleted_at is null limit 1", &[&order_id]).await {
        Ok(row) => row.get("creator_id"),
        Err(err) => {
            println!("{:?}", err);
            0
        }
    }
}

pub async fn is_products_exist(key: &str, id: i32, client: &Client) -> Result<bool, Error> {
    let query =
        format!("select count(*) as total from products where {key} = $1 and deleted_at is null");
    let row = client.query_one(&query, &[&id]).await?;
    let total: i64 = row.get("total");
    Ok(total > 0)
}

pub async fn get_product_images(client: &Client) -> Vec<String> {
    match client
        .query("select image_url from product_images", &[])
        .await
    {
        Ok(rows) => rows.iter().map(|row| row.get("image_url")).collect(),
        Err(err) => {
            println!("{:?}", err);
            vec![]
        }
    }
}

#[derive(Serialize)]
pub struct ProductForHtml {
    pub unique_id: String,
    pub product_name: String,
    pub description: String,
    pub post_image: String,
    pub product_images: Vec<String>,
    pub stock_quantity: i32,
    pub color: String,
    pub strap_material: String,
    pub strap_color: String,
    pub case_material: String,
    pub case_diameter: String,
    pub case_depth: String,
    pub case_width: String,
    pub dial_glass_type: String,
    pub dial_color: String,
    pub condition: String,
    pub movement_country: String,
    pub movement_type: String,
    pub movement_caliber: String,
    pub water_resistance: String,
    pub warranty_period: String,
    pub warranty_type: String,
    pub other_accessories_type: String,
    pub gender: String,
    pub is_preorder: String,
    pub discounted_price: String,
    pub symbol: String,
    pub discount_percent: f64,
    pub price: String,
    pub discount_reason: String,
    pub discount_type: String,
}

pub async fn get_product_for_html(product_id: i32, client: &Client) -> Option<ProductForHtml> {
    let result = client
    .query_one(
        "select p.product_id, (b.name || ' ' || p.model) as product_name, p.description, p.color, p.strap_material, p.strap_color, p.case_material, p.dial_color, p.movement_type, p.water_resistance, p.warranty_period, p.dimensions, to_char(p.price, 'FM999,999,999.00') as price, p.discount_percent::text, case when p.discount_type = 'No Discount' then to_char(p.price, 'FM999,999,999.00') when p.discount_type = 'Discount by Specific Amount' and (p.discount_expiration is null or now()::timestamp < p.discount_expiration) then to_char(p.discounted_price, 'FM999,999,999.00') else case when p.discount_expiration is null then to_char(p.price - (p.price * p.discount_percent / 100), 'FM999,999,999.00') when now()::timestamp >= p.discount_expiration then to_char(p.price, 'FM999,999,999.00') else to_char(p.price - (p.price * p.discount_percent / 100), 'FM999,999,999.00') end end as discounted_price, cur.symbol, p.stock_quantity, p.condition, wt.description warranty_type, dgt.description dial_glass_type, oat.description other_accessories_type, g.description gender, p.case_diameter, p.case_depth, p.case_width, p.movement_caliber, p.movement_country, p.is_preorder, p.discount_reason, p.discount_type from products p inner join brands b on b.brand_id = p.brand_id inner join categories c on p.category_id = c.category_id inner join shops s on s.shop_id = p.shop_id inner join currencies cur on cur.currency_id = p.currency_id inner join warranty_types wt on wt.warranty_type_id = p.warranty_type_id inner join dial_glass_types dgt on dgt.dial_glass_type_id = p.dial_glass_type_id inner join other_accessories_types oat on oat.other_accessories_type_id = p.other_accessories_type_id inner join genders g on g.gender_id = p.gender_id where b.deleted_at is null and c.deleted_at is null and s.deleted_at is null and cur.deleted_at is null and wt.deleted_at is null and dgt.deleted_at is null and oat.deleted_at is null and g.deleted_at is null and p.product_id = $1",
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
            let discount_percent: String = row.get("discount_percent");
            let discount_percent = discount_percent.parse().unwrap();

            let product_name: String = row.get("product_name");
            let unique_id = format!(
                "{}-{product_id}",
                product_name.replace(" ", "-").to_lowercase()
            );
            let post_image = if product_images.is_empty() {
                ""
            } else {
                &product_images[0]
            };
            let is_preorder: bool = row.get("is_preorder");
            let is_preorder = if is_preorder {
                "Yes".to_string()
            } else {
                "No".to_string()
            };
            Some(ProductForHtml {
                unique_id,
                product_name,
                description: row.get("description"),
                post_image: post_image.to_string(),
                product_images,
                stock_quantity: row.get("stock_quantity"),
                color: row.get("color"),
                strap_material: row.get("strap_material"),
                strap_color: row.get("strap_color"),
                case_material: row.get("case_material"),
                case_diameter: row.get("case_diameter"),
                case_depth: row.get("case_depth"),
                case_width: row.get("case_width"),
                dial_glass_type: row.get("dial_glass_type"),
                dial_color: row.get("dial_color"),
                condition: row.get("condition"),
                movement_country: row.get("movement_country"),
                movement_type: row.get("movement_type"),
                movement_caliber: row.get("movement_caliber"),
                water_resistance: row.get("water_resistance"),
                warranty_period: row.get("warranty_period"),
                warranty_type: row.get("warranty_type"),
                other_accessories_type: row.get("other_accessories_type"),
                gender: row.get("gender"),
                is_preorder,
                discounted_price: row.get("discounted_price"),
                discount_percent,
                symbol: row.get("symbol"),
                price: row.get("price"),
                discount_reason: row.get("discount_reason"),
                discount_type: row.get("discount_type"),
            })
        }
        Err(err) => {
            println!("err: {:?}", err);
            None
        }
    }
}

pub async fn get_products_for_html(client: &Client) -> Result<Vec<ProductForHtml>, Error> {
    let rows = client
    .query(
        "select p.product_id, (b.name || ' ' || p.model) as product_name, p.description, p.color, p.strap_material, p.strap_color, p.case_material, p.dial_color, p.movement_type, p.water_resistance, p.warranty_period, p.dimensions, to_char(p.price, 'FM999,999,999.00') as price, p.discount_percent::text, case when p.discount_type = 'No Discount' then to_char(p.price, 'FM999,999,999.00') when p.discount_type = 'Discount by Specific Amount' and (p.discount_expiration is null or now()::timestamp < p.discount_expiration) then to_char(p.discounted_price, 'FM999,999,999.00') else case when p.discount_expiration is null then to_char(p.price - (p.price * p.discount_percent / 100), 'FM999,999,999.00') when now()::timestamp >= p.discount_expiration then to_char(p.price, 'FM999,999,999.00') else to_char(p.price - (p.price * p.discount_percent / 100), 'FM999,999,999.00') end end as discounted_price, cur.symbol, p.stock_quantity, p.condition, wt.description warranty_type, dgt.description dial_glass_type, oat.description other_accessories_type, g.description gender, p.case_diameter, p.case_depth, p.case_width, p.movement_caliber, p.movement_country, p.is_preorder, p.discount_reason, p.discount_type from products p inner join brands b on b.brand_id = p.brand_id inner join categories c on p.category_id = c.category_id inner join shops s on s.shop_id = p.shop_id inner join currencies cur on cur.currency_id = p.currency_id inner join warranty_types wt on wt.warranty_type_id = p.warranty_type_id inner join dial_glass_types dgt on dgt.dial_glass_type_id = p.dial_glass_type_id inner join other_accessories_types oat on oat.other_accessories_type_id = p.other_accessories_type_id inner join genders g on g.gender_id = p.gender_id where p.deleted_at is null and b.deleted_at is null and c.deleted_at is null and s.deleted_at is null and cur.deleted_at is null and wt.deleted_at is null and dgt.deleted_at is null and oat.deleted_at is null and g.deleted_at is null",
        &[],
    )
    .await?;

    let mut products = vec![];
    for row in &rows {
        let product_id: i32 = row.get("product_id");

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

        let discount_percent: String = row.get("discount_percent");
        let discount_percent = discount_percent.parse().unwrap();

        let product_name: String = row.get("product_name");
        let unique_id = format!(
            "{}-{product_id}",
            product_name.replace(" ", "-").to_lowercase()
        );
        let post_image = if product_images.is_empty() {
            ""
        } else {
            &product_images[0]
        };
        let is_preorder: bool = row.get("is_preorder");
        let is_preorder = if is_preorder {
            "Yes".to_string()
        } else {
            "No".to_string()
        };
        products.push(ProductForHtml {
            unique_id,
            product_name,
            description: row.get("description"),
            post_image: post_image.to_string(),
            product_images,
            stock_quantity: row.get("stock_quantity"),
            color: row.get("color"),
            strap_material: row.get("strap_material"),
            strap_color: row.get("strap_color"),
            case_material: row.get("case_material"),
            case_diameter: row.get("case_diameter"),
            case_depth: row.get("case_depth"),
            case_width: row.get("case_width"),
            dial_glass_type: row.get("dial_glass_type"),
            dial_color: row.get("dial_color"),
            condition: row.get("condition"),
            movement_country: row.get("movement_country"),
            movement_type: row.get("movement_type"),
            movement_caliber: row.get("movement_caliber"),
            water_resistance: row.get("water_resistance"),
            warranty_period: row.get("warranty_period"),
            warranty_type: row.get("warranty_type"),
            other_accessories_type: row.get("other_accessories_type"),
            gender: row.get("gender"),
            is_preorder,
            discounted_price: row.get("discounted_price"),
            discount_percent,
            symbol: row.get("symbol"),
            price: row.get("price"),
            discount_reason: row.get("discount_reason"),
            discount_type: row.get("discount_type"),
        });
    }
    Ok(products)
}

pub fn generate_product_html(
    product: &ProductForHtml,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = fs::read_to_string("./products/template.html")?;
    html = html
        .replace("[unique_id]", product.unique_id.as_str())
        .replace("[product_name]", product.product_name.as_str())
        .replace("[description]", product.description.as_str())
        .replace("[post_image]", product.post_image.as_str())
        .replace(
            "[stock_quantity]",
            product.stock_quantity.to_string().as_str(),
        )
        .replace("[color]", product.color.as_str())
        .replace("[strap_material]", product.strap_material.as_str())
        .replace("[strap_color]", product.strap_color.as_str())
        .replace("[case_material]", product.case_material.as_str())
        .replace("[case_diameter]", product.case_diameter.as_str())
        .replace("[case_depth]", product.case_depth.as_str())
        .replace("[case_width]", product.case_width.as_str())
        .replace("[dial_glass_type]", product.dial_glass_type.as_str())
        .replace("[dial_color]", product.dial_color.as_str())
        .replace("[condition]", product.condition.as_str())
        .replace("[movement_country]", product.movement_country.as_str())
        .replace("[movement_type]", product.movement_type.as_str())
        .replace("[movement_caliber]", product.movement_caliber.as_str())
        .replace("[water_resistance]", product.water_resistance.as_str())
        .replace("[warranty_period]", product.warranty_period.as_str())
        .replace("[warranty_type]", product.warranty_type.as_str())
        .replace(
            "[other_accessories_type]",
            product.other_accessories_type.as_str(),
        )
        .replace("[gender]", product.gender.as_str())
        .replace("[is_preorder]", product.is_preorder.as_str())
        .replace("[discounted_price]", product.discounted_price.as_str())
        .replace("[symbol]", product.symbol.as_str());

    let mut image_containers = "".to_string();
    let out_of_stock_tag = if product.stock_quantity <= 0 {
        "<div class='out-of-stock-label'>Out of Stock</div>"
    } else {
        ""
    };

    for product_image in &product.product_images {
        let path = Path::new(&product_image);
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let original_image = format!("/images/{stem}_original.{extension}");
        image_containers = format!("{image_containers}<div class='image-container' style='background-image: url(..{product_image})' onclick='setTimeout(() => location.href=\"..{original_image}\", 300)'>{out_of_stock_tag}</div>");
    }
    html = html.replace("[image_cards]", &image_containers);

    if product.price != product.discounted_price {
        html = html.replace(
            "[original_price]",
            &format!(
                "<span style='text-decoration: line-through; margin-right: 0.2rem'>{}</span>",
                product.price
            ),
        );
    } else {
        html = html.replace("[original_price]", "");
    }
    let html_path = format!("./products/{}.html", product.unique_id);
    fs::write(&html_path, &html)?;
    Ok(product.unique_id.to_string())
}

pub async fn delete_product_html(product_id: i32, client: &Client) -> Result<(), Error> {
    if let Some(product) = get_product_for_html(product_id, client).await {
        match fs::remove_file(&format!("./products/{}.html", product.unique_id)) {
            Ok(_) => println!("Product html deleted successfully!"),
            Err(e) => println!("Error deleting product html: {}", e),
        }
    }
    Ok(())
}
