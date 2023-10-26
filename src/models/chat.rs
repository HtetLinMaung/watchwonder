// use chrono::NaiveDateTime;
// use serde::{Deserialize, Serialize};
// use tokio_postgres::{types::ToSql, Client};

// use crate::utils::{
//     common_struct::PaginationResult,
//     sql::{generate_pagination_query, PaginationOptions},
// };

// #[derive(Deserialize)]
// pub struct MessageRequest {
//     pub receiver_id: i32,
//     pub chat_id: i32,
//     pub message_text: String,
//     pub image_urls: Vec<String>,
// }

// pub async fn add_message(
//     data: MessageRequest,
//     sender_id: i32,
//     role: &str,
//     client: &Client,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut chat_id = data.chat_id;
//     let row = client
//         .query_one(
//             "select count(*) as total from chats where chat_id = $1 and deleted_at is null",
//             &[&data.chat_id],
//         )
//         .await?;
//     let total: i64 = row.get("total");
//     let agent_id = if role == "agent" {
//         sender_id
//     } else {
//         data.receiver_id
//     };
//     let user_id = if role != "agent" {
//         sender_id
//     } else {
//         data.receiver_id
//     };
//     if total == 0 {
//         let row = client
//             .query_one(
//                 "insert into chats (user_id, agent_id) values ($1, $2) returning chat_id",
//                 &[&user_id, &agent_id],
//             )
//             .await?;
//         chat_id = row.get("chat_id");
//     }

//     let row =client
//         .query_one(
//             "insert into messages (chat_id, sender_id, message_text) values ($1, $2, $3) returning message_id",
//             &[&chat_id, &sender_id, &data.message_text],
//         )
//         .await?;
//     let message_id: i32 = row.get("message_id");
//     for image_url in &data.image_urls {
//         client
//             .execute(
//                 "insert into message_images (message_id, image_url) values ($1, $2)",
//                 &[&message_id, &image_url],
//             )
//             .await?;
//     }
//     Ok(())
// }
// #[derive(Serialize)]
// pub struct ChatSession {
//     pub chat_id: i32,
//     pub user_name: String,
//     pub agent_id: i32,
//     pub agent_name: String,
//     pub last_message_text: String,
//     pub created_at: NaiveDateTime,
// }

// pub async fn get_chat_sessions(
//     search: &Option<String>,
//     page: Option<usize>,
//     per_page: Option<usize>,
//     user_id: i32,
//     role: &str,
//     client: &Client,
// ) -> Result<PaginationResult<ChatSession>, Box<dyn std::error::Error>> {
//     let mut base_query = "from chats c join users u on c.user_id = u.user_id join users a ON c.agent_id = a.user_id join (select message_text, created_at from messages where deleted_at is null order by created_at desc limit 1) as m on m.chat_id = c.chat_id where c.deleted_at is null".to_string();
//     let mut params: Vec<Box<dyn ToSql + Sync>> = vec![];

//     if role != "admin" {
//         params.push(Box::new(user_id));
//         params.push(Box::new(user_id));
//         base_query = format!(
//             "{base_query} and (c.user_id = ${} or c.agent_id = ${})",
//             params.len() - 1,
//             params.len()
//         );
//     }

//     let order_options = "m.created_at desc";

//     let result = generate_pagination_query(PaginationOptions {
//         select_columns:
//             "c.chat_id, u.user_id as user_id, u.name as user_name, a.user_id as agent_id, a.name as agent_name, m.message_text as last_message_text, m.status, m.created_at",
//         base_query: &base_query,
//         search_columns: vec!["u.name", "a.name"],
//         search: search.as_deref(),
//         order_options: Some(&order_options),
//         page,
//         per_page,
//     });

//     let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();

//     let row = client.query_one(&result.count_query, &params_slice).await?;
//     let total: i64 = row.get("total");

//     let mut page_counts = 0;
//     let mut current_page = 0;
//     let mut limit = 0;
//     if page.is_some() && per_page.is_some() {
//         current_page = page.unwrap();
//         limit = per_page.unwrap();
//         page_counts = (total as f64 / limit as f64).ceil() as usize;
//     }

//     let chat_sessions: Vec<ChatSession> = client
//         .query(&result.query, &params_slice)
//         .await?
//         .iter()
//         .map(|row| ChatSession {
//             chat_id: row.get("chat_id"),
//             user_name: row.get("user_name"),
//             agent_id: row.get("agent_id"),
//             agent_name: row.get("agent_name"),
//             last_message_text: row.get("last_message_text"),
//             created_at: row.get("created_at"),
//         })
//         .collect();

//     Ok(PaginationResult {
//         data: chat_sessions,
//         total,
//         page: current_page,
//         per_page: limit,
//         page_counts,
//     })
// }

// #[derive(Serialize)]
// pub struct ChatMessage {
//     pub message_id: i32,
//     pub sender_id: i32,
//     pub message_text: String,
//     pub status: String,
//     pub image_urls: Vec<String>,
//     pub created_at: NaiveDateTime,
// }

// pub async fn get_chat_messages(
//     search: &Option<String>,
//     page: Option<usize>,
//     per_page: Option<usize>,
//     chat_id: i32,
//     client: &Client,
// ) -> Result<PaginationResult<ChatMessage>, Box<dyn std::error::Error>> {
//     let base_query =
//         "from messages m join users u on u.user_id = m.sender_id where m.deleted_at is null and m.chat_id = $1"
//             .to_string();
//     let params: Vec<Box<dyn ToSql + Sync>> = vec![Box::new(chat_id)];

//     let order_options = "m.created_at desc";

//     let result = generate_pagination_query(PaginationOptions {
//         select_columns: "m.message_id, m.sender_id, m.message_text, m.status, m.created_at",
//         base_query: &base_query,
//         search_columns: vec!["m.message_text"],
//         search: search.as_deref(),
//         order_options: Some(&order_options),
//         page,
//         per_page,
//     });

//     let params_slice: Vec<&(dyn ToSql + Sync)> = params.iter().map(AsRef::as_ref).collect();

//     let row = client.query_one(&result.count_query, &params_slice).await?;
//     let total: i64 = row.get("total");

//     let mut page_counts = 0;
//     let mut current_page = 0;
//     let mut limit = 0;
//     if page.is_some() && per_page.is_some() {
//         current_page = page.unwrap();
//         limit = per_page.unwrap();
//         page_counts = (total as f64 / limit as f64).ceil() as usize;
//     }

//     let rows = client.query(&result.query, &params_slice).await?;

//     let mut chat_messages: Vec<ChatMessage> = vec![];
//     for row in &rows {
//         let message_id: i32 = row.get("message_id");
//         let image_rows = client
//             .query(
//                 "select image_url from message_images where deleted_at is null and message_id = $1",
//                 &[&message_id],
//             )
//             .await?;
//         chat_messages.push(ChatMessage {
//             message_id,
//             sender_id: row.get("sender_id"),
//             message_text: row.get("message_text"),
//             status: row.get("status"),
//             image_urls: image_rows
//                 .iter()
//                 .map(|image_row| image_row.get("image_url"))
//                 .collect(),
//             created_at: row.get("created_at"),
//         });
//     }

//     Ok(PaginationResult {
//         data: chat_messages,
//         total,
//         page: current_page,
//         per_page: limit,
//         page_counts,
//     })
// }
