use tokio_postgres::{Client, Error};

fn prefix_zeros(original: &str, desired_length: usize) -> String {
    // If the original string is already long enough, return it as is
    if original.len() >= desired_length {
        return original.to_string();
    }

    // Calculate the number of zeros needed
    let zeros_needed = desired_length - original.len();

    // Create a string with the required number of zeros and append the original string
    "0".repeat(zeros_needed) + original
}

pub async fn generate_invoice_id(client: &Client) -> Result<String, Error> {
    let row = client
        .query_one(
            "update counters set n = n + 1 where label = '777' returning n",
            &[],
        )
        .await?;

    let n: i32 = row.get("n");
    let n: &str = &format!("{}", n);
    let invoice_id = format!("777{}", prefix_zeros(n, 6));

    Ok(invoice_id)
}
