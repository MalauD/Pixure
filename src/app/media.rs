use crate::models::{Media, Resource};
use crate::tools::{ResponseStream, SeaweedFsId};
use crate::{db::get_mongo, tools::ResourceIOError};
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};
use mongodb::bson::oid::ObjectId;

pub fn config_media(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/media")
            .route("/upload", web::post().to(add_media))
            .route("{id}", web::get().to(get_media)),
    );
}

pub async fn add_media(mut payload: Multipart) -> impl Responder {
    let db = get_mongo().await;
    let mut has_error: Option<ResourceIOError> = None;
    //TODO sanitize input
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut res = Resource::<SeaweedFsId>::from_field(&field, None);
        res.alloc().await;
        res.update_public_access(Some(true), Some(true));

        let mut file_data = Vec::new();
        while let Some(chunk) = field.next().await {
            file_data.append(&mut chunk.unwrap().to_vec());
        }

        let result = res.save(None, file_data).await;
        result.unwrap_or_else(|e| {
            has_error = Some(e);
        });

        db.save_resource(res).await;
    }

    match has_error {
        Some(_) => HttpResponse::Unauthorized(),
        None => HttpResponse::Ok(),
    }
}

pub async fn get_media(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let db = get_mongo().await;

    let doc: Resource<SeaweedFsId> = db
        .find_resource(&ObjectId::with_string(&id).unwrap())
        .await
        .unwrap()
        .unwrap();

    let stream = doc.read(None).await;

    match stream {
        Ok(s) => HttpResponse::Ok()
            .content_type(doc.get_extension().essence_str())
            .streaming(ResponseStream { stream: s }),
        Err(_) => HttpResponse::Unauthorized().finish(),
    }
}
