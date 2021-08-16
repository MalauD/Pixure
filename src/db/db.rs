use crate::{
    db::MongoClient,
    models::{Identifiable, Readable, Resource, User, UserReq, Writable},
};

use core::fmt::Debug;
use mongodb::{
    bson::{doc, oid::ObjectId, to_bson},
    error::Result,
    options::FindOptions,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::convert::TryInto;
use tokio_stream::StreamExt;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
    page: usize,
    max_results: u32,
}

impl MongoClient {
    pub async fn save_resource<T>(&self, doc: Resource<T>) -> Result<()>
    where
        T: Readable
            + Writable
            + Identifiable
            + DeserializeOwned
            + Serialize
            + Unpin
            + Debug
            + Clone,
    {
        let coll = self._database.collection::<Resource<T>>("Media");
        coll.insert_one(doc, None).await?;
        Ok(())
    }

    pub async fn find_resource<T>(&self, id: &ObjectId) -> Result<Option<Resource<T>>>
    where
        T: Readable
            + Writable
            + Identifiable
            + DeserializeOwned
            + Serialize
            + Unpin
            + Debug
            + Clone,
    {
        let coll = self._database.collection::<Resource<T>>("Media");
        coll.find_one(
            doc! {
                "_id": id
            },
            None,
        )
        .await
    }

    pub async fn find_owned_resources<T>(
        &self,
        user_id: &ObjectId,
        pagination: &PaginationOptions,
    ) -> Result<Option<Vec<Resource<T>>>>
    where
        T: Readable
            + Writable
            + Identifiable
            + DeserializeOwned
            + Serialize
            + Unpin
            + Debug
            + Clone,
    {
        let coll = self._database.collection::<Resource<T>>("Media");
        let mut cursor = coll
            .find(
                doc! {
                    "owner": user_id
                },
                FindOptions::builder()
                    .batch_size(pagination.max_results.max(50))
                    .build(),
            )
            .await?
            .skip(pagination.page);
        let mut result =
            Vec::<Resource<T>>::with_capacity(pagination.max_results.max(50).try_into().unwrap());
        while let Some(value) = cursor.next().await {
            if let Ok(res) = value {
                result.push(res);
            }
        }
        Ok(Some(result))
    }

    pub async fn update_resource<T>(&self, res: &Resource<T>) -> Result<()>
    where
        T: Readable
            + Writable
            + Identifiable
            + Serialize
            + Unpin
            + Debug
            + DeserializeOwned
            + Clone,
    {
        let coll = self._database.collection::<Resource<T>>("Media");
        coll.update_one(
            doc! {"_id": res.get_id().unwrap()},
            doc! {"$set": to_bson(res).unwrap()},
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn get_user(&self, user: &UserReq) -> Result<Option<User>> {
        let coll = self._database.collection::<User>("User");
        coll.find_one(doc! {"username": user.get_username()}, None)
            .await
    }

    pub async fn save_user(&self, user: User) -> Result<()> {
        let coll = self._database.collection::<User>("User");
        coll.insert_one(user, None).await?;
        Ok(())
    }

    pub async fn has_user_by_name(&self, user: &User) -> Result<bool> {
        let coll = self._database.collection::<User>("User");
        coll.count_documents(doc! {"username": user.get_username()}, None)
            .await
            .map(|c| c != 0)
    }
}
