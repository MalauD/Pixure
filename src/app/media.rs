use crate::db::get_mongo;
use crate::models::{Identifiable, Media, Resource};
use crate::tools::ResponseStream;
use crate::tools::SeaweedFsId;
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
    //TODO sanitize input
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut res = Resource::<SeaweedFsId>::from((&field, ObjectId::new()));
        res.alloc().await;
        println!("{}", res.get_storage().as_ref().unwrap().get_uid());
        get_mongo().await.save_resource(&res).await;

        let mut file_data = Vec::new();
        while let Some(chunk) = field.next().await {
            file_data.append(&mut chunk.unwrap().to_vec());
        }

        res.save(None, file_data).await;
    }
    HttpResponse::Ok()
}

pub async fn get_media(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let db = get_mongo().await;

    let doc: Resource<SeaweedFsId> = db.find_resource(&ObjectId::with_string(&id).unwrap()).await;

    HttpResponse::Ok()
        .content_type(doc.get_extension().essence_str())
        .streaming(ResponseStream {
            stream: doc.read(None).await,
        })
}
