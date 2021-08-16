use crate::{
    db::{get_mongo, PaginationOptions},
    models::{Sessions, User, UserReq},
    tools::{SeaweedFsId, UserError},
};
use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder};
use std::sync::RwLock;

type UserResponse = Result<HttpResponse, UserError>;

pub fn config_user(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .route("/login", web::post().to(login))
            .route("/register", web::post().to(register))
            .route("/logout", web::post().to(logout))
            .route("/user", web::get().to(get_account))
            .route("/mediaOwned", web::get().to(get_owned_medias)),
    );
}

pub async fn login(
    id: Identity,
    user: web::Json<UserReq>,
    sessions: web::Data<RwLock<Sessions>>,
) -> UserResponse {
    let db = get_mongo().await;
    if let Some(user_mod) = db.get_user(&user).await? {
        user_mod.login(&user)?;
        id.remember(user_mod.get_username());
        sessions
            .write()
            .unwrap()
            .map
            .insert(user_mod.get_username(), user_mod);
        Ok(HttpResponse::Ok().append_header(("location", "/")).finish())
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}

pub async fn register(
    id: Identity,
    user: web::Json<UserReq>,
    sessions: web::Data<RwLock<Sessions>>,
) -> UserResponse {
    let db = get_mongo().await;
    let user_mod = User::new(&user.0);

    if db.has_user_by_name(&user_mod).await? {
        return Ok(HttpResponse::Unauthorized().finish());
    }
    let user_saved = user_mod.clone();
    db.save_user(user_mod).await?;
    id.remember(user.get_username());
    sessions
        .write()
        .unwrap()
        .map
        .insert(user.get_username(), user_saved.clone());
    Ok(HttpResponse::Ok().append_header(("location", "/")).finish())
}

pub async fn logout(id: Identity) -> UserResponse {
    id.forget();
    Ok(HttpResponse::Ok().append_header(("location", "/")).finish())
}

pub async fn get_account(user: User) -> impl Responder {
    web::Json(user)
}

pub async fn get_owned_medias(
    user: User,
    pagination: web::Query<PaginationOptions>,
) -> UserResponse {
    let db = get_mongo().await;
    //FIXME find_owned_resources should not depend on SeaweedFsId
    let owned_res = db
        .find_owned_resources::<SeaweedFsId>(&user.get_id().unwrap(), &pagination)
        .await?;

    Ok(HttpResponse::Ok().json(owned_res))
}
