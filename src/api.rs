mod address;
mod auth;
mod bank_account;
mod brand;
mod buyer_protection;
mod case_depth;
mod case_diameter;
mod case_material;
mod case_width;
mod category;
mod chat;
mod condition;
mod currency;
mod dial_glass_type;
mod fcm;
mod gender;
mod image;
mod insurance;
mod movement_country;
mod movement_type;
mod notification;
mod order;
mod other_accessories_type;
mod payment_type;
mod product;
mod reason_type;
mod seller_information;
mod seller_report;
mod seller_review;
mod setting;
mod shop;
mod stock_quantity;
mod strap_material;
mod terms_and_conditions;
mod user;
mod vector;
mod warranty_type;
mod water_resistance;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(auth::login);
    cfg.service(auth::hash_password);
    cfg.service(auth::register);
    cfg.service(image::upload);
    cfg.service(shop::get_shops);
    cfg.service(category::get_categories);
    cfg.service(brand::get_brands);
    cfg.service(brand::add_brand);
    cfg.service(brand::update_brand);
    cfg.service(brand::delete_brand);
    cfg.service(product::get_products);
    cfg.service(address::get_address);
    cfg.service(order::add_order);
    cfg.service(order::get_orders);
    cfg.service(product::get_models);
    cfg.service(user::get_users);
    cfg.service(user::add_user);
    cfg.service(user::get_user_by_id);
    cfg.service(user::update_user);
    cfg.service(user::delete_user);
    cfg.service(order::get_order_items);
    cfg.service(order::update_order);
    cfg.service(product::add_product);
    cfg.service(product::get_product_by_id);
    cfg.service(product::update_product);
    cfg.service(product::delete_product);
    cfg.service(brand::get_brand_by_id);
    cfg.service(shop::add_shop);
    cfg.service(shop::get_shop_by_id);
    cfg.service(shop::update_shop);
    cfg.service(shop::delete_shop);
    cfg.service(category::add_category);
    cfg.service(category::get_category_by_id);
    cfg.service(category::update_category);
    cfg.service(category::delete_category);
    cfg.service(auth::change_password);
    cfg.service(user::get_user_profile);
    cfg.service(user::update_user_profile);
    cfg.service(fcm::add_fcm);
    cfg.service(notification::get_notifications);
    cfg.service(notification::get_unread_counts);
    cfg.service(notification::update_notification_status);
    cfg.service(product::get_recommended_products_for_product);
    cfg.service(auth::verify_token);
    cfg.service(product::get_recommended_products_for_user);
    cfg.service(vector::search_vectors);
    cfg.service(terms_and_conditions::add_terms_and_conditions);
    cfg.service(terms_and_conditions::get_terms_and_conditions);
    cfg.service(insurance::add_insurance_rule);
    cfg.service(insurance::get_insurance_rules);
    cfg.service(insurance::get_insurance_rule_by_id);
    cfg.service(insurance::update_insurance_rule);
    cfg.service(insurance::delete_insurance_rule);
    cfg.service(image::resize_image);
    cfg.service(user::delete_account);
    cfg.service(seller_review::add_seller_review);
    cfg.service(seller_review::get_seller_reviews);
    cfg.service(currency::get_currencies);
    cfg.service(image::remove_dangling_images);
    cfg.service(auth::forgot_password);
    cfg.service(bank_account::get_bank_accounts);
    cfg.service(bank_account::add_bank_account);
    cfg.service(bank_account::get_bank_account_by_id);
    cfg.service(bank_account::update_bank_account);
    cfg.service(bank_account::delete_bank_account);
    cfg.service(warranty_type::get_warranty_types);
    cfg.service(buyer_protection::add_buyer_protection);
    cfg.service(buyer_protection::get_buyer_protections);
    cfg.service(buyer_protection::get_buyer_protection_by_id);
    cfg.service(buyer_protection::update_buyer_protection);
    cfg.service(buyer_protection::delete_buyer_protection);
    cfg.service(dial_glass_type::get_dial_glass_types);
    cfg.service(condition::get_conditions);
    cfg.service(other_accessories_type::get_other_accessories_types);
    cfg.service(gender::get_genders);
    cfg.service(seller_information::get_seller_information);
    cfg.service(setting::get_settings);
    cfg.service(order::get_order_shop_name);
    cfg.service(chat::send_message);
    cfg.service(chat::get_chat_sessions);
    cfg.service(chat::update_message_status);
    cfg.service(chat::get_chat_messages);
    cfg.service(chat::update_message_status);
    cfg.service(chat::delete_message);
    cfg.service(chat::get_total_unread_counts);
    cfg.service(chat::update_instantio_state);
    cfg.service(chat::get_last_active_at);
    cfg.service(chat::get_chat_session_by_id);
    cfg.service(chat::get_chat_message_by_id);
    cfg.service(chat::delete_chat_session);
    cfg.service(case_diameter::get_case_diameters);
    cfg.service(case_depth::get_case_depths);
    cfg.service(case_width::get_case_widths);
    cfg.service(movement_type::get_movement_types);
    cfg.service(strap_material::get_strap_materials);
    cfg.service(case_material::get_case_materials);
    cfg.service(stock_quantity::get_stock_quantities);
    cfg.service(water_resistance::get_water_resistances);
    cfg.service(movement_country::get_movement_countries);
    cfg.service(seller_report::get_report_subjects);
    cfg.service(seller_report::get_seller_reports);
    cfg.service(seller_report::add_seller_report);
    cfg.service(seller_report::get_seller_report_by_id);
    cfg.service(fcm::notify_all);
    cfg.service(payment_type::get_payment_types);
    cfg.service(reason_type::get_reason_types);
}
