#[rocket::get("/<id>")]
pub async fn basic_download(id: &str) -> String {
    format!("Could not fetch {id}, this part is not done yet :)")
}
