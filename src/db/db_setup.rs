use std::fmt::Debug;

use mongodb::{
    bson::{doc, oid::ObjectId, to_bson},
    error::{Error, Result},
    options::ClientOptions,
    Client, Database,
};
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::models::{Identifiable, Readable, Resource, User, UserReq, Writable};

static MONGO: OnceCell<MongoClient> = OnceCell::new();
static MONGO_INITIALIZED: OnceCell<Mutex<bool>> = OnceCell::new();

pub struct MongoClient {
    _database: Database,
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
        coll.insert_one(user, None).await;
        Ok(())
    }

    pub async fn has_user_by_name(&self, user: &User) -> Result<bool> {
        let coll = self._database.collection::<User>("User");
        coll.count_documents(doc! {"username": user.get_username()}, None)
            .await
            .map(|c| c != 0)
    }
}

pub async fn get_mongo() -> &'static MongoClient {
    if let Some(c) = MONGO.get() {
        return c;
    }

    let initializing_mutex = MONGO_INITIALIZED.get_or_init(|| tokio::sync::Mutex::new(false));

    let mut initialized = initializing_mutex.lock().await;

    if !*initialized {
        if let Ok(client_options) =
            ClientOptions::parse("mongodb://localhost:27017/?appName=Pixure").await
        {
            if let Ok(client) = Client::with_options(client_options) {
                if MONGO
                    .set(MongoClient {
                        _database: client.database("Pixure"),
                    })
                    .is_ok()
                {
                    *initialized = true;
                }
            }
        }
    }
    drop(initialized);
    MONGO.get().unwrap()
}
