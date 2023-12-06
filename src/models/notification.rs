use std::collections::HashMap;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    fcm::send_notification,
    sql::{generate_pagination_query, PaginationOptions},
};

use super::{fcm::get_fcm_tokens, user::get_admin_user_ids};

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub notification_id: i32,
    pub name: String,
    pub username: String,
    pub title: String,
    pub message: String,
    pub status: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_notifications(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    user_id: i32,
    status_list: &Option<Vec<String>>,
    client: &Client,
) -> Result<PaginationResult<Notification>, Error> {
    let mut base_query =
        "from notifications n inner join users u on n.user_id = u.user_id where n.deleted_at is null and u.deleted_at is null and n.user_id = $1"
            .to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(user_id)];

    if let Some(slist) = status_list {
        if !slist.is_empty() {
            if slist.len() > 1 {
                let mut placeholders: Vec<String> = vec![];
                for status in slist {
                    params.push(Box::new(status));
                    placeholders.push(format!("${}", params.len()));
                }
                base_query = format!("{base_query} and n.status in ({})", placeholders.join(", "));
            } else {
                let status = slist[0].clone();
                params.push(Box::new(status));
                base_query = format!("{base_query} and n.status = ${}", params.len());
            }
        }
    }

    let order_options = "n.created_at desc";
    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "n.notification_id, n.title, n.message, n.status, u.name, u.username, n.created_at",
        base_query: &base_query,
        search_columns: vec!["n.title", "n.message", "n.status", "u.name", "u.username"],
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

    let notifications: Vec<Notification> = client
        .query(&result.query, &params_slice[..])
        .await?
        .iter()
        .map(|row| Notification {
            notification_id: row.get("notification_id"),
            name: row.get("name"),
            username: row.get("username"),
            title: row.get("title"),
            message: row.get("message"),
            status: row.get("status"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: notifications,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_unread_counts(user_id: i32, client: &Client) -> Result<i64, Error> {
    let result = client
        .query_one(
            "select count(*) as total from notifications where user_id = $1 and status = 'Unread' and deleted_at is null",
            &[&user_id],
        )
        .await?;
    let total: i64 = result.get("total");
    Ok(total)
}

pub async fn update_notification_status(
    notification_id: i32,
    status: &str,
    client: &Client,
) -> Result<(), Error> {
    client.execute("update notifications set status = $1 where notification_id = $2 and deleted_at is null", &[&status, &notification_id]).await?;
    Ok(())
}

pub async fn add_notification_to_admins(
    title: &str,
    message: &str,
    data: &Option<HashMap<String, Value>>,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let user_id_list = get_admin_user_ids(&client).await?;
    for user_id in user_id_list {
        add_notification(user_id, title, message, data, &client).await?;
    }
    Ok(())
}

pub async fn add_notifications_for_all_users(
    title: &str,
    message: &str,
    client: &Client,
) -> Result<(), Error> {
    let rows = client
        .query(
            "select user_id from users where deleted_at is null and role != 'admin'",
            &[],
        )
        .await?;
    for row in &rows {
        let user_id: i32 = row.get("user_id");
        client
            .execute(
                "insert into notifications (user_id, title, message) values ($1, $2, $3)",
                &[&user_id, &title, &message],
            )
            .await?;
    }
    Ok(())
}

pub async fn add_notification(
    user_id: i32,
    title: &str,
    message: &str,
    data: &Option<HashMap<String, Value>>,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "insert into notifications (user_id, title, message) values ($1, $2, $3)",
            &[&user_id, &title, &message],
        )
        .await?;
    let fcm_tokens = match get_fcm_tokens(user_id, &client).await {
        Ok(tokens) => tokens,
        Err(_) => vec![],
    };
    for fcm_token in &fcm_tokens {
        send_notification(title, message, fcm_token, data.clone()).await?;
    }
    Ok(())
}
