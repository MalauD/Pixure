use actix_web::{App, HttpServer};
use app::config_media;

mod app;
mod db;
mod models;
mod tools;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const PORT: i32 = 80;

    HttpServer::new(move || App::new().configure(config_media))
        .bind(format!("0.0.0.0:{}", PORT))?
        .run()
        .await
}
