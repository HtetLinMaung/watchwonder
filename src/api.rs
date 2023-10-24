mod address;
mod auth;
mod bank_account;
mod brand;
mod category;
mod currency;
mod fcm;
mod image;
mod insurance;
mod notification;
mod order;
mod product;
mod seller_review;
mod shop;
mod terms_and_conditions;
mod user;
mod vector;
mod warranty_type;

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
    cfg.service(warranty_type::get_warranty_types);
}
