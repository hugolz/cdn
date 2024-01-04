use std::str::FromStr;

use crate::{cache::Cache, response::{JsonApiResponse, JsonApiResponseBuilder}};
use rocket::tokio::sync::RwLock;
use rocket::{
    http::Status,
    serde::json::{
        serde_json::json,
        Json,
    },
};

#[rocket::get("/<id>")]
pub async fn basic_download(id: &str, cache: &rocket::State<RwLock<Cache>>) -> JsonApiResponse {
    debug!("Download request of: {id}");
    let (meta, data) = cache.read().await.load(uuid::Uuid::from_str(id).unwrap()).await.unwrap();

    let data_b64 = rbase64::encode(&data);

    JsonApiResponseBuilder::default().with_json(json!({
        "metadata": meta,
        "file": data_b64
    })).with_status(Status::Ok).build()
}
