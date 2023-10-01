mod auth;
mod brand;
mod category;
mod image;
mod product;
mod shop;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(auth::login);
    cfg.service(auth::hash_password);
    cfg.service(auth::register);
    cfg.service(image::upload);
    cfg.service(shop::get_shops);
    cfg.service(category::get_categories);
    cfg.service(brand::get_brands);
    cfg.service(product::get_products);
}
