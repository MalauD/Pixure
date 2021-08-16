use mongodb::{bson::doc, options::ClientOptions, Client, Database};
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

static MONGO: OnceCell<MongoClient> = OnceCell::new();
static MONGO_INITIALIZED: OnceCell<Mutex<bool>> = OnceCell::new();

pub struct MongoClient {
    pub(in crate::db) _database: Database,
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
    MONGO
        .get()
        .unwrap()
        ._database
        .run_command(
            doc! {
                "createIndexes": "Media",
                "indexes": [
                    {
                        "key": { "access": 1 },
                        "name": "access_index",
                        "unique": false
                    },
                ]
            },
            None,
        )
        .await
        .expect("Cannot create index");
    drop(initialized);
    MONGO.get().unwrap()
}
