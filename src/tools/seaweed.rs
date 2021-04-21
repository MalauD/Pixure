use async_trait::async_trait;
use mongodb::bson::{from_bson, Bson};
use serde::Deserialize;
use std::string::String;

use super::get_seaweed;
use crate::models::{BytesStream, Identifiable, Readable, Writable};

#[derive(Deserialize)]
pub struct SeaweedFsId {
    id: String,
}

impl SeaweedFsId {
    pub fn get_volume(&self) -> i16 {
        let vec: Vec<&str> = self.id.split(",").collect();
        vec.first()
            .expect("Cannot get volume id")
            .parse::<i16>()
            .unwrap()
    }
}

#[async_trait]
impl Readable for SeaweedFsId {
    async fn read(&self) -> BytesStream {
        //TODO remove blocking
        let client = get_seaweed().await;
        let stream = client.get_file(self).await;
        return stream;
    }
}

#[async_trait]
impl Writable for SeaweedFsId {
    async fn save(&self, stream: BytesStream) -> () {
        let client = get_seaweed().await;
        let id = client.get_alloc().await;
        client.set_file(&id, stream).await;
    }
}

impl Identifiable for SeaweedFsId {
    type IdType = String;
    fn get_uid(&self) -> &String {
        &self.id
    }
    fn from_uid(uid: Self::IdType) -> Self {
        SeaweedFsId { id: uid }
    }
    fn from_bson(bson: &Bson) -> Self {
        Self::from_uid(from_bson(bson.clone()).unwrap_or_default())
    }
}
