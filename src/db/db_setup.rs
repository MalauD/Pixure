use mongodb::{Client, Database, options::ClientOptions};
use once_cell::sync::OnceCell;
use tokio::sync::Mutex;

static MONGO: OnceCell<Database> = OnceCell::new();
static MONGO_INITIALIZED: OnceCell<Mutex<bool>> = OnceCell::new();

pub async fn get_mongo() -> &'static Database {
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
                if MONGO.set(client.database("Pixure")).is_ok() {
                    *initialized = true;
                }
            }
        }
    }
    drop(initialized);
    MONGO.get().unwrap()
}
