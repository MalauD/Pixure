use std::sync::RwLock;

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder};

use crate::{
    db::get_mongo,
    models::{Sessions, User, UserReq},
};

pub fn config_user(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("/login", web::post().to(login))
            .route("/register", web::post().to(register))
            .route("/logout", web::post().to(logout))
            .route("/user", web::get().to(get_account)),
    );
}

pub async fn login(
    id: Identity,
    user: web::Json<UserReq>,
    sessions: web::Data<RwLock<Sessions>>,
) -> impl Responder {
    let db = get_mongo().await;
    if let Some(user_mod) = db.get_user(&user).await {
        match user_mod.login(&user) {
            Ok(_) => {
                id.remember(user_mod.get_username());
                sessions
                    .write()
                    .unwrap()
                    .map
                    .insert(user_mod.get_username(), user_mod);
                HttpResponse::Ok().append_header(("location", "/")).finish()
            }
            Err(_) => HttpResponse::Unauthorized().finish(),
        }
    } else {
        HttpResponse::Forbidden().finish()
    }
}

pub async fn register(
    id: Identity,
    user: web::Json<UserReq>,
    sessions: web::Data<RwLock<Sessions>>,
) -> impl Responder {
    let db = get_mongo().await;
    let user_mod = User::new(&user.0);

    if db.has_user_by_name(&user_mod).await {
        return HttpResponse::Unauthorized().finish();
    }
    let user_saved = user_mod.clone();
    db.save_user(user_mod).await;
    id.remember(user.get_username());
    sessions
        .write()
        .unwrap()
        .map
        .insert(user.get_username(), user_saved.clone());
    HttpResponse::Ok().append_header(("location", "/")).finish()
}

pub async fn logout(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::Ok().append_header(("location", "/")).finish()
}

pub async fn get_account(user: User) -> impl Responder {
    web::Json(user)
}
