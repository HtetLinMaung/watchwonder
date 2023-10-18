pub struct PaginationOptions<'a> {
    pub select_columns: &'a str,
    pub base_query: &'a str,
    pub search_columns: Vec<&'a str>,
    pub order_options: Option<&'a str>,
    pub search: Option<&'a str>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

pub struct PaginationQueryResult {
    pub query: String,
    pub count_query: String,
}

pub fn generate_pagination_query(options: PaginationOptions) -> PaginationQueryResult {
    let mut query = format!("SELECT {} {}", options.select_columns, options.base_query);

    let mut count_query = format!("SELECT COUNT(*) as total {}", options.base_query);

    if let Some(s) = options.search {
        if !s.is_empty() {
            let search_clauses: Vec<String> = options
                .search_columns
                .iter()
                .map(|col| format!("{} LIKE '%{}%'", col, s))
                .collect();
            let search_query = search_clauses.join(" OR ");
            // Check if the query already contains a WHERE clause
            if query.contains(" WHERE ") || query.contains(" where ") {
                query = format!("{} AND ({})", query, search_query);
            } else {
                query = format!("{} WHERE ({})", query, search_query);
            }

            // Check if the count_query already contains a WHERE clause
            if count_query.contains(" WHERE ") || query.contains(" where ") {
                count_query = format!("{} AND ({})", count_query, search_query);
            } else {
                count_query = format!("{} WHERE ({})", count_query, search_query);
            }
        }
    }

    if let Some(order) = options.order_options {
        query = format!("{} ORDER BY {}", query, order);
    }

    if let (Some(page), Some(per_page)) = (options.page, options.per_page) {
        let offset = (page - 1) * per_page;
        query = format!("{} LIMIT {} OFFSET {}", query, per_page, offset);
    }
    println!("query: {query}");
    // println!("count_query: {count_query}");
    PaginationQueryResult { query, count_query }
}

// Usage:
// let options = PaginationOptions {
//     select_columns: "p.product_id, p.brand_id, b.name brand_name, ...".to_string(),
//     base_query: "FROM products p INNER JOIN ...".to_string(),
//     search_columns: vec!["b.name", "p.model", "p.description", ...].into_iter().map(String::from).collect(),
//     order_options: Some("model ASC, created_at DESC".to_string()),
//     search: Some("search_term".to_string()),
//     page: Some(1),
//     per_page: Some(10),
// };

// let (query, count_query) = generate_pagination_query(options);
