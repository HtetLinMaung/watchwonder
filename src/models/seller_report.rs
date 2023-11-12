use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tokio_postgres::{types::ToSql, Client, Error};

use crate::utils::{
    common_struct::PaginationResult,
    sql::{generate_pagination_query, PaginationOptions},
};

use super::notification;

#[derive(Serialize)]
pub struct ReportSubject {
    pub subject_id: i32,
    pub description: String,
}

pub async fn get_report_subjects(
    client: &Client,
) -> Result<Vec<ReportSubject>, Box<dyn std::error::Error>> {
    let rows = client
        .query(
            "select subject_id, description from report_subjects where deleted_at is null",
            &[],
        )
        .await?;
    Ok(rows
        .iter()
        .map(|row| ReportSubject {
            subject_id: row.get("subject_id"),
            description: row.get("description"),
        })
        .collect())
}

#[derive(Deserialize)]
pub struct SellerReportRequest {
    pub seller_id: i32,
    pub phone: String,
    pub subject_id: i32,
    pub message: String,
}

pub async fn add_seller_report(
    data: &SellerReportRequest,
    user_id: i32,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let row= client
        .query_one(
            "insert into seller_reports (user_id, seller_id, subject_id, message, phone) values ($1, $2, $3, $4, $5) returning report_id",
            &[&user_id, &data.seller_id, &data.subject_id, &data.message, &data.phone],
        )
        .await?;
    let report_id: i32 = row.get("report_id");
    let row= client.query_one("select u.name, rs.description from seller_reports sr join users u on u.user_id = sr.user_id join report_subjects rs on rs.subject_id = sr.subject_id where sr.report_id = $1", &[&report_id]).await?;
    let user_name: String = row.get("name");
    let report_subject: String = row.get("description");
    let title = format!("New Report Submitted");
    let message = format!(
        "{user_name} has submitted a report. Subject: '{report_subject}'. Please review the details."
    );
    match notification::add_notification_to_admins(&title, &message, &client).await {
        Ok(()) => {
            println!("Notification added successfully.");
        }
        Err(err) => {
            println!("Error adding notification: {:?}", err);
        }
    };

    Ok(())
}

#[derive(Serialize)]
pub struct SellerReport {
    pub report_id: i32,
    pub phone: String,
    pub user_name: String,
    pub seller_name: String,
    pub subject: String,
    pub message: String,
    pub created_at: NaiveDateTime,
}

pub async fn get_seller_reports(
    search: &Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
    client: &Client,
) -> Result<PaginationResult<SellerReport>, Error> {
    let base_query =
        "from seller_reports sr join users u1 on u1.user_id = sr.user_id join users u2 on u2.user_id = sr.seller_id join report_subjects rs on rs.subject_id = sr.subject_id where sr.deleted_at is null"
            .to_string();
    let params: Vec<Box<dyn ToSql + Sync>> = vec![];
    let order_options = "created_at desc";

    let result = generate_pagination_query(PaginationOptions {
        select_columns:
            "report_id, u1.name user_name, u2.name seller_name, rs.description subject, sr.message, sr.phone, sr.created_at",
        base_query: &base_query,
        search_columns: vec!["u1.name", "u2.name", "rs.description", "sr.message", "sr.phone"],
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

    let seller_reports: Vec<SellerReport> = client
        .query(&result.query, &params_slice)
        .await?
        .iter()
        .map(|row| SellerReport {
            report_id: row.get("report_id"),
            user_name: row.get("user_name"),
            seller_name: row.get("seller_name"),
            subject: row.get("subject"),
            message: row.get("message"),
            phone: row.get("phone"),
            created_at: row.get("created_at"),
        })
        .collect();

    Ok(PaginationResult {
        data: seller_reports,
        total,
        page: current_page,
        per_page: limit,
        page_counts,
    })
}

pub async fn get_seller_report_by_id(report_id: i32, client: &Client) -> Option<SellerReport> {
    match client.query_one("select report_id, u1.name user_name, u2.name seller_name, rs.description subject, sr.message, sr.phone, sr.created_at from seller_reports sr join users u1 on u1.user_id = sr.user_id join users u2 on u2.user_id = sr.seller_id join report_subjects rs on rs.subject_id = sr.subject_id where sr.deleted_at is null and sr.report_id = $1", &[&report_id]).await {
        Ok(row) => Some(SellerReport {
            report_id: row.get("report_id"),
            user_name: row.get("user_name"),
            seller_name: row.get("seller_name"),
            subject: row.get("subject"),
            message: row.get("message"),
            phone: row.get("phone"),
            created_at: row.get("created_at"),
        }),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    }
}
