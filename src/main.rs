use actix_web::{App, HttpServer};
use app::{config_media, config_user};

mod app;
mod db;
mod models;
mod tools;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const PORT: i32 = 80;
    std::fs::create_dir_all("./tmp").unwrap();
    HttpServer::new(move || App::new().configure(config_media).configure(config_user))
        .bind(format!("0.0.0.0:{}", PORT))?
        .run()
        .await
}
