// #[macro_use]
// extern crate rocket;

mod basic;
mod json;

#[rocket::get("/<id>")]
async fn download(id: &str) -> String {
    format!("Could not fetch {id}, this part is not done yet :)")
}

#[rocket::get("/")]
fn root() -> &'static str {
    "
        Hi, please take a look at the /examples directory to understand how to use this api
    "
}

#[rocket::catch(403)]
pub async fn root_403() -> String {
    "403".to_string()
}

#[rocket::launch]
async fn rocket() -> _ {
    rocket::build()
        .register("/", rocket::catchers![root_403])
        .mount("/", rocket::routes![root, json::upload_json, basic::upload, download])
        .register("/json", rocket::catchers![json::upload_json_400])
}
