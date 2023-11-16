const MAX_FILE_SIZE: rocket::data::ByteUnit = rocket::data::ByteUnit::Kilobyte(128);

#[rocket::post("/", data = "<file>")]
pub async fn upload(file: rocket::Data<'_>) -> String {
    // I wonder if there is a way to get the file name..
    let stream = file.open(MAX_FILE_SIZE);
    let Ok(buff) = stream.into_bytes().await else {
        return "Failled to unpack the file".to_string();
    };
    format!("Received file with len: {}", buff.len())
}
