use crate::{models::User, tools::ResourceIOError};
use actix_multipart::Field;
use async_trait::async_trait;
use bytes::Bytes;
use futures::Stream;
use mime::Mime;
use mongodb::bson::{doc, oid::ObjectId, Bson};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt::Debug, pin::Pin};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessRight {
    user: ObjectId,
    write: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resource<StorageType>
where
    StorageType: Readable + Writable + Identifiable + Serialize + Unpin + Debug + Clone,
{
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    _storage: Option<StorageType>,
    #[serde(
        serialize_with = "serialize_mime",
        deserialize_with = "deserialize_mime"
    )]
    extension: Mime,
    owner: ObjectId,
    access: Vec<AccessRight>,
    r_public: bool,
    w_public: bool,
}

fn serialize_mime<S>(element: &Mime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    element.essence_str().serialize(serializer)
}

fn deserialize_mime<'de, D>(deserializer: D) -> Result<Mime, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer).map(|mime_type| mime_type.parse::<Mime>().unwrap())
}

impl<StorageType> Resource<StorageType>
where
    StorageType: Readable + Writable + Identifiable + Serialize + Unpin + Debug + Clone,
{
    ///Return _id identifier produced by MongoDb
    pub fn get_id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    ///Get a stream of underlying storage
    pub async fn read(&self, request_user: Option<&User>) -> Result<BytesStream, ResourceIOError> {
        if self.r_public {
            return Ok(self._storage.as_ref().unwrap().read().await);
        }
        if let Some(user) = request_user {
            if let Some(request_id) = user.get_id() {
                if request_id == self.get_owner()
                    || self.access.iter().any(|a| a.user == request_id)
                {
                    return Ok(self._storage.as_ref().unwrap().read().await);
                }
            }
        }
        Err(ResourceIOError::InsufficientPermissions(
            "reading".to_string(),
        ))
    }

    ///Save storage to resource
    pub async fn save(
        &self,
        request_user: Option<&User>,
        data: Vec<u8>,
    ) -> Result<(), ResourceIOError> {
        if self.w_public {
            self._storage.as_ref().unwrap().save(data).await;
            return Ok(());
        }
        if let Some(user) = request_user {
            if let Some(request_id) = user.get_id() {
                if request_id == self.get_owner() || self.access.iter().any(|a| a.write == true) {
                    self._storage.as_ref().unwrap().save(data).await;
                    return Ok(());
                }
            }
        }
        Err(ResourceIOError::InsufficientPermissions(
            "writing".to_string(),
        ))
    }

    ///Allocate storage of underlying storage.
    ///Calls alloc() of Storage
    pub async fn alloc(&mut self) {
        self._storage.get_or_insert(StorageType::alloc().await);
    }

    ///Get underlying storage
    pub fn get_storage(&self) -> &Option<StorageType> {
        &self._storage
    }

    ///Change access rights of the resource
    pub fn update_public_access(&mut self, r_public: Option<bool>, w_public: Option<bool>) {
        self.r_public = match r_public {
            None => self.r_public,
            Some(e) => e,
        };

        self.w_public = match w_public {
            None => self.w_public,
            Some(e) => e,
        };
    }

    ///Create resource from http body Field
    pub fn from_field(field: &Field, user: &User) -> Self {
        let id = user.get_id().unwrap();
        Self {
            id: None,
            _storage: None,
            owner: id.clone(),
            access: vec![AccessRight {
                user: id.clone(),
                write: true,
            }],
            extension: field.content_type().clone(),
            r_public: false,
            w_public: false,
        }
    }
}

impl<StorageType> Media for Resource<StorageType>
where
    StorageType: Readable + Writable + Identifiable + Serialize + Unpin + Debug + Clone,
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
