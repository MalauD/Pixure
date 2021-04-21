use crate::db::get_mongo;
use crate::models::{Media, Resource};
use crate::tools::ResponseStream;
use crate::tools::SeaweedFsId;
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use futures::{StreamExt, TryFuture, TryStreamExt};
use mongodb::bson::{doc, oid::ObjectId};

pub fn config_media(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/media")
            .route("/", web::post().to(add_media))
            .route("{id}", web::get().to(get_media)),
    );
}

pub async fn add_media(mut payload: Multipart) -> impl Responder {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let res = Resource::<SeaweedFsId>::from((field, ObjectId::new()));
        res.save(None, field.into_stream()).await;
    }
    HttpResponse::Ok()
}

pub async fn get_media(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    let db = get_mongo().await;

    let coll = db.collection("Media");

    let found = coll
        .find_one(
            doc! {
                "_id": ObjectId::with_string(&id).unwrap()
            },
            None,
        )
        .await
        .expect("Error");

    let mut doc = Resource::<SeaweedFsId>::new(&found.unwrap());
    HttpResponse::Ok()
        .content_type(doc.get_extension().essence_str())
        .streaming(ResponseStream {
            stream: doc.read(None).await,
        })
}
