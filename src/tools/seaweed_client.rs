use cached::proc_macro::cached;
use core::future::Future;
use once_cell::sync::OnceCell;
use reqwest::{Body, Client};
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::models::{BytesStream, Identifiable};
use crate::tools::SeaweedFsId;

static SEAWEED_CLIENT: OnceCell<SeaweedFsClient> = OnceCell::new();
static SEAWEED_CLIENT_INITIALIZED: OnceCell<Mutex<bool>> = OnceCell::new();

pub struct SeaweedFsClient {
    _client: Client,
}

#[derive(Deserialize)]
pub struct VolumeLocation {
    url: String,
}

#[derive(Deserialize)]
pub struct VolumeLookup {
    locations: Vec<VolumeLocation>,
}

#[cached(size = 100)]
async fn get_volume_addr(volume: i16) -> String {
    let url = format!("http://5.1.1.1:9333/dir/lookup?volumeId={}", volume);
    let res = reqwest::get(url).await.expect("Cannot get");
    let parsed = res.json::<VolumeLookup>().await.expect("Failed");
    parsed.locations.first().unwrap().url.to_owned()
}

pub async fn get_seaweed() -> &'static SeaweedFsClient {
    if let Some(c) = SEAWEED_CLIENT.get() {
        return c;
    }

    let initializing_mutex =
        SEAWEED_CLIENT_INITIALIZED.get_or_init(|| tokio::sync::Mutex::new(false));

    let mut initialized = initializing_mutex.lock().await;

    if !*initialized {
        if SEAWEED_CLIENT
            .set(SeaweedFsClient {
                _client: Client::new(),
            })
            .is_ok()
        {
            *initialized = true;
        }
    }
    drop(initialized);
    SEAWEED_CLIENT.get().unwrap()
}

impl SeaweedFsClient {
    pub fn get_client(&self) -> &Client {
        &self._client
    }
    pub async fn get_file(&self, fid: &SeaweedFsId) -> BytesStream {
        let addr = get_volume_addr(fid.get_volume()).await;
        let url = format!("http://{}/{}", addr, fid.get_uid());
        let res = self.get_client().get(url).send().await.expect("Failed");
        Box::pin(res.bytes_stream())
    }

    pub async fn get_alloc(&self) -> SeaweedFsId {
        let url = "http://5.1.1.1:9333/dir/assign";
        let res = self.get_client().get(url).send().await.expect("Failed");
        res.json().await.unwrap()
    }

    pub async fn set_file<'a>(&'a self, fid: &'a SeaweedFsId, stream: BytesStream) {
        let addr = get_volume_addr(fid.get_volume()).await;
        let body = Body::wrap_stream(stream);
        self
            .get_client()
            .post(format!("http://{}/{}", addr, fid.get_uid()))
            .body(body)
            .send()
            .await
            .expect("Cannot Upload");
    }
}
