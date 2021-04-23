use std::pin::Pin;

use actix_multipart::Field;
use async_trait::async_trait;
use bytes::Bytes;
use futures::{stream, Stream};
use mime::Mime;
use mongodb::bson::{doc, oid::ObjectId, Bson, Document};
use reqwest::Result;
use serde::de::DeserializeOwned;

extern crate std;

mod internal {
    pub struct Dimension {
        width: i32,
        height: i32,
    }

    pub fn get_size(fid: i32) -> Dimension {
        Dimension {
            width: 0,
            height: 0,
        }
    }
}

pub type Dim = internal::Dimension;

pub trait Media {
    fn get_dim(&self) -> Dim;
    fn get_size(&self) -> i32;
    fn get_owner(&self) -> ObjectId;
    fn get_extension(&self) -> &Mime;
}

pub type BytesStream = Pin<Box<dyn Stream<Item = Result<Bytes>> + Send + Sync>>;

#[async_trait]
pub trait Readable {
    async fn read(&self) -> BytesStream;
}

#[async_trait]
pub trait Writable {
    async fn save(&self, stream: BytesStream) -> ();
    async fn alloc() -> Self;
}

pub trait Identifiable {
    type IdType;
    fn get_uid(&self) -> &Self::IdType;
    fn from_uid(uid: Self::IdType) -> Self;
    fn from_bson(bson: &Bson) -> Self;
}

pub struct Resource<StorageType>
where
    StorageType: Readable + Writable + Identifiable + DeserializeOwned,
{
    _storage: Option<StorageType>,
    _doc: Document,
    extension: Mime,
    owner: ObjectId,
    r_access: Vec<ObjectId>,
    w_access: Vec<ObjectId>,
    r_public: bool,
    w_public: bool,
}

impl<StorageType> Resource<StorageType>
where
    StorageType: Readable + Writable + Identifiable + DeserializeOwned,
{
    pub async fn read(&self, request_id_o: Option<ObjectId>) -> BytesStream {
        if self.r_public {
            return self._storage.as_ref().unwrap().read().await;
        }
        if let Some(request_id) = request_id_o {
            if request_id == self.get_owner() || self.r_access.contains(&request_id) {
                return self._storage.as_ref().unwrap().read().await;
            }
        }
        return Box::pin(stream::empty());
    }

    pub async fn save(&self, request_id_o: Option<ObjectId>, stream: BytesStream) {
        if self.w_public {
            self._storage.as_ref().unwrap().save(stream).await;
            return;
        }
        if let Some(request_id) = request_id_o {
            if request_id == self.get_owner() || self.w_access.contains(&request_id) {
                self._storage.as_ref().unwrap().save(stream).await;
                return;
            }
        }
    }

    pub async fn alloc(&mut self) {
        self._storage.get_or_insert(StorageType::alloc().await);
    }

    pub fn get_storage(&self) -> &Option<StorageType> {
        &self._storage
    }

    pub fn get_doc(&self) -> &Document {
        &self._doc
    }

    pub fn new(doc: &Document) -> Self {
        let r_access_docs = doc.get_array("r_access").unwrap();
        let r_access: Vec<ObjectId> = r_access_docs
            .iter()
            .map(|r| r.as_object_id().unwrap().clone())
            .collect();

        let w_access_docs = doc.get_array("w_access").unwrap();
        let w_access: Vec<ObjectId> = w_access_docs
            .iter()
            .map(|w| w.as_object_id().unwrap().clone())
            .collect();

        let storage = doc.get("_storage");

        Resource {
            _storage: match storage {
                Some(x) => Some(StorageType::from_bson(x)),
                None => None,
            },
            _doc: doc.clone(),
            owner: doc.get_object_id("owner").unwrap().clone(),
            extension: doc
                .get_str("extension")
                .unwrap()
                .to_owned()
                .parse::<Mime>()
                .unwrap(),
            r_access,
            w_access,
            r_public: doc.get_bool("r_public").unwrap_or_default(),
            w_public: doc.get_bool("w_public").unwrap_or_default(),
        }
    }
}

impl<StorageType> From<(&Field, ObjectId)> for Resource<StorageType>
where
    StorageType: Readable + Writable + Identifiable + DeserializeOwned,
{
    fn from(input: (&Field, ObjectId)) -> Self {
        let doc = doc! {
            "owner": input.1.clone(),
            "r_access": [input.1.clone()],
            "w_access": [input.1.clone()],
            "extension": input.0.content_type().essence_str(),
            "r_public": false,
            "w_public": false,
        };
        Self::new(&doc)
    }
}

impl<StorageType> Media for Resource<StorageType>
where
    StorageType: Readable + Writable + Identifiable + DeserializeOwned,
{
    fn get_dim(&self) -> Dim {
        todo!()
    }

    fn get_size(&self) -> i32 {
        todo!()
    }

    fn get_owner(&self) -> ObjectId {
        self.owner.clone()
    }

    fn get_extension(&self) -> &Mime {
        &self.extension
    }
}
