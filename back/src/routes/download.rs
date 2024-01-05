use std::str::FromStr;

use crate::{
    cache::Cache,
    response::{JsonApiResponse, JsonApiResponseBuilder},
};
use rocket::tokio::sync::RwLock;
use rocket::{
    http::Status,
    serde::json::{serde_json::json, Json},
};

#[rocket::get("/<id>")]
pub async fn basic_download(id: &str, cache: &rocket::State<RwLock<Cache>>) -> JsonApiResponse {
    debug!("Download request of: {id}");
    let Ok(id) = uuid::Uuid::from_str(id) else {
        warn!("Could not understand given id: {id}");
        return JsonApiResponseBuilder::default()
            .with_json(json!( {
                "message": "could not understand given id",
                "result": "denied"
            }))
            .with_status(Status::BadRequest)
            .build();
    };
    let (meta, data) = cache.read().await.load(id).await.unwrap();

    let data_b64 = rbase64::encode(&data);

    JsonApiResponseBuilder::default()
        .with_json(json!({
            "metadata": meta,
            "file": data_b64
        }))
        .with_status(Status::Ok)
        .build()
}
