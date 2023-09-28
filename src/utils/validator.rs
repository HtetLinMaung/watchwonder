use regex::Regex;

pub fn validate_email(email: &str) -> bool {
    // Define a regular expression for validating an Email
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

    // Use the regex to validate the input email
    email_regex.is_match(email)
}

pub fn validate_mobile(mobile: &str) -> bool {
    // Define a regular expression for validating a Myanmar Mobile Number
    // Myanmar mobile numbers typically start with '09', '+959', or '959' followed by 7 to 9 digits.
    let mobile_regex = Regex::new(r"^\+?959\d{7,9}$|^09\d{7,9}$").unwrap();

    // Use the regex to validate the input mobile number
    mobile_regex.is_match(mobile)
}
