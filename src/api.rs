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
    cfg.service(brand::addbrands);
    cfg.service(product::get_products);
    cfg.service(address::get_address);
    cfg.service(order::add_order);
    cfg.service(order::get_orders);
    cfg.service(product::get_models);
    cfg.service(user::get_users);
}
