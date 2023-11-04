use std::{collections::HashMap, fs};

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::{types::ToSql, Client};

use crate::utils::{
    common_struct::PaginationResult,
    fcm::send_notification,
    socketio,
    sql::{generate_pagination_query, PaginationOptions},
};

use super::{fcm, user};

#[derive(Deserialize)]
pub struct MessageRequest {
    pub receiver_id: i32,
    pub chat_id: i32,
    pub message_text: String,
    pub image_urls: Vec<String>,
}

pub async fn add_message(
    data: &MessageRequest,
    sender_id: i32,
    role: &str,
    client: &Client,
) -> Result<i32, Box<dyn std::error::Error>> {
    let mut chat_id = data.chat_id;
    let row = client
        .query_one(
            "select count(*) as total from chats where chat_id = $1 and deleted_at is null",
            &[&data.chat_id],
        )
        .await?;
    let total: i64 = row.get("total");

    if total == 0 {
        let chat_exists_query = format!("select chat_id from chat_participants where user_id in ({}, {}) group by chat_id having count(distinct user_id) = 2", sender_id, data.receiver_id);
        match client.query_one(&chat_exists_query, &[]).await {
            Ok(ce_row) => {
                chat_id = ce_row.get("chat_id");
            }
            Err(err) => {
                println!("{:?}", err);
                let row = client
                    .query_one(
                        "insert into chats (is_group) values (FALSE) returning chat_id",
                        &[],
                    )
                    .await?;
                chat_id = row.get("chat_id");
                client
                    .execute(
                        "insert into chat_participants (chat_id, user_id) values ($1, $2)",
                        &[&chat_id, &sender_id],
                    )
                    .await?;
                client
                    .execute(
                        "insert into chat_participants (chat_id, user_id) values ($1, $2)",
                        &[&chat_id, &data.receiver_id],
                    )
                    .await?;
                let mut rooms = get_admin_ids(client).await?;
                rooms.push(data.receiver_id);
                tokio::spawn(async move {
                    let mut payload: HashMap<String, Value> = HashMap::new();
                    payload.insert("chat_id".to_string(), Value::Number(chat_id.into()));
                    match socketio::emit("new-chat", &rooms, Some(payload)).await {
                        Ok(_) => {
                            println!("event sent successfully.");
                        }
                        Err(err) => {
                            println!("{:?}", err);
                        }
                    };
                });
            }
        };
    }
    let row = client
        .query_one(
            "select count(*) as total from chat_participants where user_id = $1",
            &[&sender_id],
        )
        .await?;
    let total: i64 = row.get("total");
    if total == 0 && role == "admin" {
        client
            .execute(
                "insert into chat_participants (chat_id, user_id) values ($1, $2)",
                &[&chat_id, &sender_id],
            )
            .await?;
    }

    if total > 0 || role == "admin" {
        let row =client
        .query_one(
            "insert into messages (chat_id, sender_id, message_text) values ($1, $2, $3) returning message_id",
            &[&chat_id, &sender_id, &data.message_text],
        )
        .await?;
        let message_id: i32 = row.get("message_id");
        for image_url in &data.image_urls {
            client
                .execute(
                    "insert into message_images (message_id, image_url) values ($1, $2)",
                    &[&message_id, &image_url],
                )
                .await?;
        }

        let mut rooms = get_admin_ids(client).await?;
        rooms.push(data.receiver_id);
        tokio::spawn(async move {
            let mut payload = HashMap::new();
            payload.insert("message_id".to_string(), Value::Number(message_id.into()));
            match socketio::emit("new-message", &rooms, Some(payload)).await {
                Ok(_) => {
                    println!("event sent successfully.");
                }
                Err(err) => {
                    println!("{:?}", err);
                }
            };
        });

        let fcm_tokens = fcm::get_fcm_tokens(data.receiver_id, client).await?;
        let admin_fcm_tokens = fcm::get_admin_fcm_tokens(client).await?;
        let sender_name = user::get_user_name(sender_id, client).await.unwrap();
        let message_text = data.message_text.clone();
        tokio::spawn(async move {
            for fcm_token in &fcm_tokens {
                let mut map = HashMap::new();
                map.insert(
                    "event".to_string(),
                    Value::String("new-message".to_string()),
                );
                map.insert("message_id".to_string(), Value::Number(message_id.into()));
                match send_notification(&sender_name, &message_text, fcm_token, Some(map)).await {
                    Ok(_) => {
                        println!("notification message sent successfully.");
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                };
            }
            for fcm_token in &admin_fcm_tokens {
                let mut map = HashMap::new();
                map.insert(
                    "event".to_string(),
                    Value::String("new-message".to_string()),
                );
                map.insert("message_id".to_string(), Value::Number(message_id.into()));
                match send_notification(&sender_name, &message_text, fcm_token, Some(map)).await {
                    Ok(_) => {
                        println!("notification message sent successfully.");
                    }
                    Err(err) => {
                        println!("{:?}", err);
                    }
                };
            }
        });
    }
    Ok(chat_id)
}

#[derive(Serialize)]
pub struct ChatParticipant {
    pub user_id: i32,
    pub name: String,
    pub profile_image: String,
    pub is_me: bool,
}

#[derive(Serialize)]
pub struct ChatSession {
    pub chat_id: i32,
    pub chat_name: String,
    pub sender_id: i32,
    pub sender_name: String,
    pub profile_image: String,
    pub last_message_text: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub chat_participants: Vec<ChatParticipant>,
    pub unread_counts: i64,
}

pub async fn get_chat_sessions(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    user_id: i32,
    role: &str,
    client: &Client,
) -> Result<PaginationResult<ChatSession>, Box<dyn std::error::Error>> {
    let mut base_query = "from chats c join (select message_text, created_at, sender_id, chat_id, status from messages where deleted_at is null order by created_at desc limit 1) as m on m.chat_id = c.chat_id join users u on m.sender_id = u.user_id where c.deleted_at is null and u.deleted_at is null".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

    if role != "admin" {
        params.push(Box::new(user_id));
        base_query = format!(
            "{base_query} and c.chat_id in (select chat_id from chat_participants where user_id = ${})",
            params.len()
        );
    }

    let order_options = "m.created_at desc";

    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "c.chat_id, m.sender_id, u.name as sender_name, m.message_text as last_message_text, m.status, m.created_at",
        base_query: &base_query,
        search_columns: vec!["m.message_text", "u.name"],
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

    let mut chat_sessions: Vec<ChatSession> = vec![];
    for row in client.query(&result.query, &params_slice).await? {
        let chat_id: i32 = row.get("chat_id");

        let cp_rows=  client.query("select cp.user_id, u.name, u.profile_image from chat_participants cp join users u on u.user_id = cp.user_id where cp.chat_id = $1", &[&chat_id]).await?;
        let mut chat_participants: Vec<ChatParticipant> = vec![];
        let mut chat_names: Vec<String> = vec![];
        let mut profile_images: Vec<String> = vec![];
        for cp_row in &cp_rows {
            let cp_user_id: i32 = cp_row.get("user_id");
            let cp_name: String = cp_row.get("name");
            if user_id != cp_user_id {
                chat_names.push(cp_name);
                profile_images.push(cp_row.get("profile_image"));
            }
            let cp_user_id: i32 = cp_row.get("user_id");
            chat_participants.push(ChatParticipant {
                user_id: cp_row.get("user_id"),
                name: cp_row.get("name"),
                profile_image: cp_row.get("profile_image"),
                is_me: cp_user_id == user_id,
            })
        }
        let chat_name = chat_names.join(", ");
        let profile_image = profile_images.join(", ");

        let message_row = client
            .query_one(
                "select count(*) as unread_counts from messages where chat_id = $1 and sender_id != $2 and deleted_at is null and status != 'read'",
                &[&chat_id, &user_id],
            )
            .await?;

        chat_sessions.push(ChatSession {
            chat_id,
            chat_name,
            sender_id: row.get("sender_id"),
            sender_name: row.get("sender_name"),
            profile_image,
            last_message_text: row.get("last_message_text"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            chat_participants,
            unread_counts: message_row.get("unread_counts"),
        })
    }

    Ok(PaginationResult {
        data: chat_sessions,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_chat_session_by_id(
    chat_id: i32,
    user_id: i32,
    receiver_id: i32,
    client: &Client,
) -> Result<ChatSession, Box<dyn std::error::Error>> {
    let chat_id = if chat_id == 0 {
        let query = format!("select chat_id from chat_participants where user_id in ({}, {}) group by chat_id having count(distinct user_id) = 2", user_id, receiver_id);
        match client.query_one(&query, &[]).await {
            Ok(row) => row.get("chat_id"),
            Err(err) => {
                println!("{:?}", err);
                0
            }
        }
    } else {
        chat_id
    };
    let row =  client.query_one("select c.chat_id, m.sender_id, u.name as sender_name, m.message_text as last_message_text, m.status, m.created_at from chats c join (select message_text, created_at, sender_id, chat_id, status from messages where deleted_at is null order by created_at desc limit 1) as m on m.chat_id = c.chat_id join users u on m.sender_id = u.user_id where c.deleted_at is null and u.deleted_at is null and c.chat_id = $1", &[&chat_id]).await?;
    let cp_rows=  client.query("select cp.user_id, u.name, u.profile_image from chat_participants cp join users u on u.user_id = cp.user_id where cp.chat_id = $1", &[&chat_id]).await?;
    let mut chat_participants: Vec<ChatParticipant> = vec![];
    let mut chat_names: Vec<String> = vec![];
    let mut profile_images: Vec<String> = vec![];
    for cp_row in &cp_rows {
        let cp_user_id: i32 = cp_row.get("user_id");
        let cp_name: String = cp_row.get("name");
        if user_id != cp_user_id {
            chat_names.push(cp_name);
            profile_images.push(cp_row.get("profile_image"));
        }
        let cp_user_id: i32 = cp_row.get("user_id");
        chat_participants.push(ChatParticipant {
            user_id: cp_row.get("user_id"),
            name: cp_row.get("name"),
            profile_image: cp_row.get("profile_image"),
            is_me: cp_user_id == user_id,
        })
    }
    let chat_name = chat_names.join(", ");
    let profile_image = profile_images.join(", ");

    let message_row = client
      .query_one(
          "select count(*) as unread_counts from messages where chat_id = $1 and sender_id != $2 deleted_at is null and status != 'read'",
          &[&chat_id, &user_id],
      )
      .await?;

    Ok(ChatSession {
        chat_id,
        chat_name,
        sender_id: row.get("sender_id"),
        sender_name: row.get("sender_name"),
        profile_image,
        last_message_text: row.get("last_message_text"),
        status: row.get("status"),
        created_at: row.get("created_at"),
        chat_participants,
        unread_counts: message_row.get("unread_counts"),
    })
}

#[derive(Serialize)]
pub struct ChatMessage {
    pub chat_id: i32,
    pub message_id: i32,
    pub sender_id: i32,
    pub sender_name: String,
    pub profile_image: String,
    pub message_text: String,
    pub status: String,
    pub is_my_message: bool,
    pub image_urls: Vec<String>,
    pub created_at: NaiveDateTime,
}

pub async fn get_chat_messages(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    chat_id: i32,
    user_id: i32,
    receiver_id: i32,
    client: &Client,
) -> Result<PaginationResult<ChatMessage>, Box<dyn std::error::Error>> {
    let chat_id = if chat_id == 0 {
        let query = format!("select chat_id from chat_participants where user_id in ({}, {}) group by chat_id having count(distinct user_id) = 2", user_id, receiver_id);
        match client.query_one(&query, &[]).await {
            Ok(row) => row.get("chat_id"),
            Err(err) => {
                println!("{:?}", err);
                0
            }
        }
    } else {
        chat_id
    };
    let base_query =
        "from messages m join users u on u.user_id = m.sender_id where m.deleted_at is null and m.chat_id = $1"
            .to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(chat_id)];

    let order_options = "m.created_at desc";

    let result = generate_pagination_query(PaginationOptions {
        select_columns: "m.chat_id, m.message_id, m.sender_id, u.name as sender_name, u.profile_image, m.message_text, m.status, m.created_at",
        base_query: &base_query,
        search_columns: vec!["m.message_text"],
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

    let rows = client.query(&result.query, &params_slice).await?;

    let mut chat_messages: Vec<ChatMessage> = vec![];
    for row in &rows {
        let message_id: i32 = row.get("message_id");
        let image_rows = client
            .query(
                "select image_url from message_images where deleted_at is null and message_id = $1",
                &[&message_id],
            )
            .await?;
        let sender_id = row.get("sender_id");
        chat_messages.push(ChatMessage {
            chat_id: row.get("chat_id"),
            message_id,
            sender_id,
            sender_name: row.get("sender_name"),
            profile_image: row.get("profile_image"),
            message_text: row.get("message_text"),
            status: row.get("status"),
            image_urls: image_rows
                .iter()
                .map(|image_row| image_row.get("image_url"))
                .collect(),
            is_my_message: sender_id == user_id,
            created_at: row.get("created_at"),
        });
    }

    Ok(PaginationResult {
        data: chat_messages,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_chat_message_by_id(
    message_id: i32,
    user_id: i32,
    client: &Client,
) -> Result<ChatMessage, Box<dyn std::error::Error>> {
    let row = client.query_one("select m.chat_id, m.message_id, m.sender_id, u.name as sender_name, u.profile_image, m.message_text, m.status, m.created_at from messages m join users u on u.user_id = m.sender_id where m.deleted_at is null and m.message_id = $1", &[&message_id]).await?;

    let message_id: i32 = row.get("message_id");
    let image_rows = client
        .query(
            "select image_url from message_images where deleted_at is null and message_id = $1",
            &[&message_id],
        )
        .await?;
    let sender_id = row.get("sender_id");
    Ok(ChatMessage {
        chat_id: row.get("chat_id"),
        message_id,
        sender_id,
        sender_name: row.get("sender_name"),
        profile_image: row.get("profile_image"),
        message_text: row.get("message_text"),
        status: row.get("status"),
        image_urls: image_rows
            .iter()
            .map(|image_row| image_row.get("image_url"))
            .collect(),
        is_my_message: sender_id == user_id,
        created_at: row.get("created_at"),
    })
}

pub async fn update_message_status(
    message_id: i32,
    status: &str,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update messages set status = $1 where message_id = $2",
            &[&status, &message_id],
        )
        .await?;
    Ok(())
}

pub async fn is_own_message(message_id: i32, user_id: i32, client: &Client) -> bool {
    let row= client.query_one("select count(*) as total from messages where message_id = $1 and sender_id = $2 and deleted_at is null", &[&message_id, &user_id]).await.unwrap();
    let total: i64 = row.get("total");
    total > 0
}

pub async fn delete_message(
    message_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    client
        .execute(
            "update messages set deleted_at = CURRENT_TIMESTAMP where message_id = $1",
            &[&message_id],
        )
        .await?;
    let rows = client
        .query(
            "select image_url from message_images where deleted_at is null and message_id = $1",
            &[&message_id],
        )
        .await?;
    for row in &rows {
        let image_url: String = row.get("image_url");
        let image_path = image_url.replace("/image", "./image");
        match fs::remove_file(image_path) {
            Ok(_) => println!("File deleted successfully!"),
            Err(e) => println!("Error deleting file: {}", e),
        };
    }
    Ok(())
}

pub async fn get_total_unread_counts(
    role: &str,
    user_id: i32,
    client: &Client,
) -> Result<i64, Box<dyn std::error::Error>> {
    let mut query = "select count(*) as unread_counts from messages where deleted_at is null and status != 'read' and sender_id != $1".to_string();
    let mut params: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(user_id)];

    if role != "admin" {
        params.push(Box::new(user_id));
        query = format!(
            "{query} and chat_id in (select chat_id from chat_participants where user_id = ${})",
            params.len()
        );
    }

    let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();
    let row = client.query_one(&query, &params_slice).await?;
    Ok(row.get("unread_counts"))
}

pub async fn get_admin_ids(client: &Client) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
    let rows = client
        .query(
            "select user_id from users where role = 'admin' and deleted_at is null",
            &[],
        )
        .await?;
    Ok(rows.iter().map(|row| row.get("user_id")).collect())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateStateData {
    pub room: Option<i32>,
    pub from: Option<i32>,
    pub to: Option<Vec<i32>>,
    pub payload: Option<Value>, // This will hold the arbitrary JSON data
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateStateRequest {
    pub event: String,
    pub data: UpdateStateData,
}

pub async fn update_instantio_state(
    body: &UpdateStateRequest,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    if body.event == "disconnect" {
        let user_id = body.data.room.unwrap();
        client
            .execute(
                "update users set is_online = false, last_active_at = CURRENT_TIMESTAMP where user_id = $1",
                &[&user_id],
            )
            .await?;
    } else if body.event == "join" {
        let user_id = body.data.room.unwrap();
        client
            .execute(
                "update users set is_online = true where user_id = $1",
                &[&user_id],
            )
            .await?;
    }
    Ok(())
}

pub async fn get_last_active_at(
    user_id: i32,
    client: &Client,
) -> Result<Option<NaiveDateTime>, Box<dyn std::error::Error>> {
    let row = client
        .query_one(
            "select last_active_at from users where user_id = $1",
            &[&user_id],
        )
        .await?;
    Ok(row.get("last_active_at"))
}
