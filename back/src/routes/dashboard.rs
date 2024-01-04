use crate::cache::Cache;
use rocket::tokio::sync::RwLock;

#[rocket::get("/dashboard/cache_count")]
pub async fn dashboard_cache_count(cache: &rocket::State<RwLock<Cache>>) -> String {
    format!("{}", cache.read().await.data.len())
}
