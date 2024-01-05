use crate::cache::Cache;
use rocket::serde::json::serde_json::{json, Value};
use rocket::tokio::sync::RwLock;

#[rocket::get("/dashboard")]
pub async fn dashboard_cache_count(cache: &rocket::State<RwLock<Cache>>) -> String {
    let data = cache
        .read()
        .await
        .data
        .iter()
        .map(|cache_entry| rocket::serde::json::to_string(&**cache_entry).unwrap())
        .collect::<Vec<String>>();

    format!("{data:?}",)
}
