use std::convert::TryInto;

use actix_web::{web, HttpResponse, Responder};

use crate::{
    db::get_mongo,
    models::{User, UserReq},
};

pub fn config_user(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("/login", web::post().to(login))
            .route("/register", web::post().to(register)),
    );
}

pub async fn login(user: web::Json<UserReq>) -> impl Responder {
    let db = get_mongo().await;
    let user_mod = db.get_user(&user).await.unwrap();
    match user_mod.login(&user) {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::Unauthorized(),
    }
}

pub async fn register(user: web::Json<UserReq>) -> impl Responder {
    let db = get_mongo().await;
    let user_mod = User::new(user.0);
    //println!("{:?}", user_mod.credential);
    if db.has_user_by_name(&user_mod).await {
        return HttpResponse::Unauthorized();
    }
    db.save_user(user_mod).await;
    HttpResponse::Ok()
}
