use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{web::Data, App, HttpServer};
use app::{config_media, config_user};
use std::sync::RwLock;

use crate::models::Sessions;

mod app;
mod db;
mod models;
mod tools;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const PORT: i32 = 80;

    let sessions: Data<RwLock<Sessions>> = Data::new(RwLock::new(Default::default()));

    HttpServer::new(move || {
        App::new()
            .app_data(sessions.clone())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("pixure-id")
                    .secure(false),
            ))
            .configure(config_media)
            .configure(config_user)
    })
    .bind(format!("0.0.0.0:{}", PORT))?
    .run()
    .await
}
