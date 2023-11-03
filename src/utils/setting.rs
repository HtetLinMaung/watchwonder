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
