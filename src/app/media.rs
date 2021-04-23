use crate::db::get_mongo;
use crate::models::{Identifiable, Media, Resource};
use crate::tools::ResponseStream;
use crate::tools::SeaweedFsId;
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};
use mongodb::bson::{doc, oid::ObjectId};
use std::io::Write;
use std::sync::Arc;

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
        let filepath = Arc::new(format!(
            "./tmp/{}",
            sanitize_filename::sanitize(res.get_storage().as_ref().unwrap().get_uid())
        ));

        let fp = filepath.clone();
        let mut f = web::block(move || std::fs::File::create(fp.as_ref()))
            .await
            .expect("File creation error")
            .unwrap();

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f))
                .await
                .expect("Error writing to file")
                .unwrap();
        }

        let fp2 = filepath.clone();
        web::block(move || std::fs::remove_file(fp2.as_ref()))
            .await
            .expect("Error removing file")
            .unwrap();
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

    let doc = Resource::<SeaweedFsId>::new(&found.unwrap());
    HttpResponse::Ok()
        .content_type(doc.get_extension().essence_str())
        .streaming(ResponseStream {
            stream: doc.read(None).await,
        })
}
