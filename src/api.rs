mod address;
mod auth;
mod brand;
mod category;
mod image;
mod order;
mod product;
mod shop;
mod user;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(auth::login);
    cfg.service(auth::hash_password);
    cfg.service(auth::register);
    cfg.service(image::upload);
    cfg.service(shop::get_shops);
    cfg.service(category::get_categories);
    cfg.service(brand::get_brands);
    cfg.service(brand::add_brands);
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
}
