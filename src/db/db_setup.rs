use std::ops::Deref;

use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    options::ClientOptions,
    Client, Database,
};
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::Mutex;

use crate::models::{Identifiable, Readable, Resource, Writable};

static MONGO: OnceCell<MongoClient> = OnceCell::new();
static MONGO_INITIALIZED: OnceCell<Mutex<bool>> = OnceCell::new();

pub struct MongoClient {
    _database: Database,
}

impl MongoClient {
    pub async fn save_resource<T>(&self, doc: &Resource<T>)
    where
        T: Readable + Writable + Identifiable + DeserializeOwned + Serialize,
    {
        let coll = self._database.collection::<Document>("Media");
        coll.insert_one(doc.get_doc(), None).await;
    }

    pub async fn find_resource<T>(&self, id: &ObjectId) -> Resource<T>
    where
        T: Readable + Writable + Identifiable + DeserializeOwned + Serialize,
    {
        let coll = self._database.collection::<Document>("Media");
        let found = coll
            .find_one(
                doc! {
                    "_id": id
                },
                None,
            )
            .await
            .expect("Error");

        Resource::<T>::new(&found.unwrap())
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
