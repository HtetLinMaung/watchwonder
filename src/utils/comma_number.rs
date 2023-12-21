pub fn format_with_commas(num: f64) -> String {
    let num_as_int = num as i64;
    let num_str = num_as_int.to_string();
    let mut result = String::new();
    let chars: Vec<char> = num_str.chars().rev().collect();

    for (i, ch) in chars.iter().enumerate() {
        if i % 3 == 0 && i != 0 {
            result.push(',');
        }
        result.push(*ch);
    }

    result.chars().rev().collect()
}
