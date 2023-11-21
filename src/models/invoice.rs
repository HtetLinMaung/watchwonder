use std::{collections::HashMap, fs};

use serde_json::Value;
use tokio_postgres::Client;

use crate::utils::{report_forge, socketio};

pub async fn export_invoice(
    order_id: i32,
    user_id: i32,
    client: &Client,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let row= client.query_one("select u.name customer_name, u.phone, oa.home_address, oa.street_address, oa.city, oa.state, oa.postal_code, oa.country, oa.township, oa.ward, s.name shop_name, u2.phone seller_phone, s.address shop_address, to_char(o.created_at, 'DD Month YYYY') as order_date, oi.order_id, o.invoice_id, o.payment_type, oa.note
    from order_items oi join orders o on o.order_id = oi.order_id join users u on u.user_id = o.user_id join products p on oi.product_id = p.product_id join shops s on s.shop_id = p.shop_id join users u2 on u2.user_id = p.creator_id join order_addresses oa on oa.address_id = o.shipping_address_id
    where o.order_id = $1", &[&order_id]).await?;
    let mut contents = fs::read_to_string("./images/invoice.html")?;

    let home_address: &str = row.get("home_address");
    let street_address: &str = row.get("street_address");
    let ward: &str = row.get("ward");
    let township: &str = row.get("township");
    let city: &str = row.get("city");
    let state: &str = row.get("state");
    let country: &str = row.get("country");
    let customer_address = vec![
        home_address,
        street_address,
        ward,
        township,
        city,
        state,
        country,
    ]
    .join(", ");
    contents = contents
        .replace("[customer_name]", row.get("customer_name"))
        .replace("[customer_phone]", row.get("phone"))
        .replace("[customer_address]", &customer_address)
        .replace("[shop_name]", row.get("shop_name"))
        .replace("[seller_phone]", row.get("seller_phone"))
        .replace("[seller_address]", row.get("shop_address"))
        .replace("[order_date]", row.get("order_date"))
        .replace("[order_id]", row.get("order_id"))
        .replace("[invoice_id]", row.get("invoice_id"))
        .replace("[payment_type]", row.get("payment_type"))
        .replace("[note]", row.get("note"));

    let rows= client.query("select b.name brand_name, p.model, oi.quantity, c.symbol, oi.price::text, (oi.quantity * oi.price)::text as total from order_items oi join products p on p.product_id = oi.product_id join brands b on b.brand_id = p.brand_id join currencies c on c.currency_id = oi.currency_id where oi.order_id = $1 order by b.name, p.model", &[&order_id]).await?;

    let mut order_items = String::new();
    for row in &rows {
        let brand_name: &str = row.get("brand_name");
        let model: &str = row.get("model");
        let quantity: i32 = row.get("quantity");
        let symbol: &str = row.get("symbol");
        let price: &str = row.get("price");
        let total: &str = row.get("total");

        order_items = format!(
            "{order_items} <div class=\"flex row\"><div style=\"text-align: left\">{} {}</div><div>{}</div><div>{} {}</div><div>{} {}</div></div>", brand_name, model, quantity, symbol, price, symbol, total
        );
    }
    contents = contents.replace("[order_items]", &order_items);

    let row = client
        .query_one(
            "select sum(quantity * price)::text as sub_total from order_items where order_id = $1",
            &[&order_id],
        )
        .await?;
    let sub_total: &str = row.get("sub_total");
    contents = contents.replace("[sub_total]", sub_total);

    println!("contents: {}", contents);
    match report_forge::site_to_pdf(&contents).await {
        Ok(response) => {
            let code = response.get("code").and_then(Value::as_i64).unwrap_or(0) as i32;
            if code == 200 {
                let invoice_url = response
                    .get("data")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                println!("invoice_url: {}", invoice_url);
                client
                    .execute(
                        "update orders set invoice_url = $1 where order_id = $2",
                        &[&invoice_url, &order_id],
                    )
                    .await?;
                let mut payload: HashMap<String, Value> = HashMap::new();
                payload.insert(
                    "invoice_url".to_string(),
                    Value::String(invoice_url.clone()),
                );
                match socketio::emit("new-invoice", &vec![user_id], Some(payload)).await {
                    Ok(_) => {
                        println!("event sent successfully.");
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                };
                Ok(Some(invoice_url))
            } else {
                Ok(None)
            }
        }
        Err(err) => {
            println!("{:?}", err);
            Ok(None)
        }
    }
}
