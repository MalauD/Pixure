use crate::models::{Media, Resource, User};
use crate::tools::{ResponseStream, SeaweedFsId};
use crate::{db::get_mongo, tools::ResourceIOError};
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures::{StreamExt, TryStreamExt};
use mongodb::bson::oid::ObjectId;

type ResourceResponse = Result<HttpResponse, ResourceIOError>;

pub fn config_media(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/media")
            .route("/upload", web::post().to(add_media))
            .route("{id}", web::get().to(get_media)),
    );
}

pub async fn add_media(mut payload: Multipart, user: User) -> ResourceResponse {
    let db = get_mongo().await;
    //TODO sanitize input
    while let Ok(Some(mut field)) = payload.try_next().await {
        let mut res = Resource::<SeaweedFsId>::from_field(&field, &user);
        res.alloc().await;
        //res.update_public_access(Some(true), Some(true));

        let mut file_data = Vec::new();
        while let Some(chunk) = field.next().await {
            file_data.append(&mut chunk.unwrap().to_vec());
        }
        println!("Saving file");
        res.save(Some(&user), file_data).await?;

        db.save_resource(res).await?;
    }

    Ok(HttpResponse::Ok().finish())
}

pub async fn get_media(path: web::Path<String>, user: User) -> ResourceResponse {
    let id = path.into_inner();
    let db = get_mongo().await;

    let doc: Resource<SeaweedFsId> = db
        .find_resource(&ObjectId::with_string(&id).unwrap())
        .await
        .unwrap()
        .unwrap();

    let stream = doc.read(Some(&user)).await;

    match stream {
        Ok(s) => Ok(HttpResponse::Ok()
            .content_type(doc.get_extension().essence_str())
            .streaming(ResponseStream { stream: s })),
        Err(_) => Ok(HttpResponse::Unauthorized().finish()),
    }
}
