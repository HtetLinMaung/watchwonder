use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PaginationResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Vec<T>,
    pub total: i64,
    pub page: usize,
    pub per_page: usize,
    pub page_counts: usize,
}
