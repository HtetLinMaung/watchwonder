pub fn get_demo_user_id() -> i32 {
    let demo_user_id: i32 = std::env::var("DEMO_USER_ID")
        .unwrap_or("0".to_string())
        .parse()
        .unwrap();
    demo_user_id
}

pub fn get_demo_platform() -> String {
    std::env::var("DEMO_PLATFORM").unwrap_or("".to_string())
}

pub fn get_android_version() -> String {
    let android_version = std::env::var("ANDROID_VERSION").unwrap_or("0.0.0".to_string());
    android_version
}

pub fn get_ios_version() -> String {
    let ios_version = std::env::var("IOS_VERSION").unwrap_or("0.0.0".to_string());
    ios_version
}

pub fn get_version_update_message() -> String {
    let version_update_message = std::env::var("VERSION_UPDATE_MESSAGE").unwrap_or("".to_string());
    version_update_message
}

pub fn get_max_cash_on_delivery_amount() -> f64 {
    let max_cash_on_delivery_amount: f64 = std::env::var("MAX_CASH_ON_DELIVERY_AMOUNT")
        .unwrap_or("2000000".to_string())
        .parse()
        .unwrap();
    max_cash_on_delivery_amount
}
