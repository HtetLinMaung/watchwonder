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
