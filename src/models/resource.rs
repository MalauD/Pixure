use crate::tools::ResourceErrorIO;
use actix_multipart::Field;
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use mime::Mime;
use mongodb::bson::{doc, oid::ObjectId, to_bson, Bson, Document};
use serde::{de::DeserializeOwned, Serialize};
use std::pin::Pin;

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

#[derive(Debug)]
pub enum UseType {
    Reading,
    Writing,
}

impl std::fmt::Display for UseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type BytesStream = Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send + Sync>>;

#[async_trait]
pub trait Readable {
    async fn read(&self) -> BytesStream;
}

#[async_trait]
pub trait Writable {
    async fn save(&self, data: Vec<u8>) -> ();
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
    StorageType: Readable + Writable + Identifiable + DeserializeOwned + Serialize,
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
    StorageType: Readable + Writable + Identifiable + DeserializeOwned + Serialize,
{
    pub async fn read(
        &self,
        request_id_o: Option<ObjectId>,
    ) -> Result<BytesStream, ResourceErrorIO> {
        if self.r_public {
            return Ok(self._storage.as_ref().unwrap().read().await);
        }
        if let Some(request_id) = request_id_o {
            if request_id == self.get_owner() || self.r_access.contains(&request_id) {
                return Ok(self._storage.as_ref().unwrap().read().await);
            }
        }
        return Err(ResourceErrorIO::InsufficientPermissions);
    }

    pub async fn save(
        &self,
        request_id_o: Option<ObjectId>,
        data: Vec<u8>,
    ) -> Result<(), ResourceErrorIO> {
        if self.w_public {
            self._storage.as_ref().unwrap().save(data).await;
            return Ok(());
        }
        if let Some(request_id) = request_id_o {
            if request_id == self.get_owner() || self.w_access.contains(&request_id) {
                self._storage.as_ref().unwrap().save(data).await;
                return Ok(());
            }
        }
        return Err(ResourceErrorIO::InsufficientPermissions);
    }

    pub async fn alloc(&mut self) {
        self._storage.get_or_insert(StorageType::alloc().await);
        self._doc.insert(
            "_storage",
            to_bson(self._storage.as_ref().unwrap()).unwrap(),
        );
    }

    pub fn get_storage(&self) -> &Option<StorageType> {
        &self._storage
    }

    pub fn get_doc_ref(&self) -> &Document {
        &self._doc
    }

    pub fn get_doc_ref_mut(&mut self) -> &mut Document {
        &mut self._doc
    }

    pub fn get_doc(&self) -> Document {
        self._doc.clone()
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

    pub fn update_public_access(&mut self, r_public: Option<bool>, w_public: Option<bool>) {
        self.r_public = match r_public {
            None => self.r_public,
            Some(e) => e,
        };
        *self.get_doc_ref_mut().get_bool_mut("r_public").unwrap() = self.r_public;

        self.w_public = match w_public {
            None => self.w_public,
            Some(e) => e,
        };
        *self.get_doc_ref_mut().get_bool_mut("w_public").unwrap() = self.w_public;
    }
}

impl<StorageType> From<(&Field, ObjectId)> for Resource<StorageType>
where
    StorageType: Readable + Writable + Identifiable + DeserializeOwned + Serialize,
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
    StorageType: Readable + Writable + Identifiable + DeserializeOwned + Serialize,
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
