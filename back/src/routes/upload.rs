use crate::cache::Cache;
use rocket::tokio::sync::RwLock;
use rocket::{
    http::Status,
    serde::json::{
        serde_json::json,
        Json,
    },
};


#[rocket::post("/json", format = "application/json", data = "<data>")]
pub async fn upload_json(
    data: Json<crate::data::UploadData>,
    cache: &rocket::State<RwLock<Cache>>,
) -> crate::response::JsonApiResponse {
    // Setup


    let start_timer = std::time::Instant::now();
    let id = uuid::Uuid::new_v4();
    let metadata = data.metadata.clone();
    let file_data = &data.file;
    let wait_store = false;

    // Validation of user input

    if !regex::Regex::new(r"^[A-Za-z0-9]*$")
        .unwrap()
        .is_match(&metadata.file_ext)
    {
        return crate::response::JsonApiResponseBuilder::default()
            .with_json(json!({"result": "denied", "message": "The specified extension should only contain alphanumeric characters"}))
            .with_status(Status::BadRequest).build();
    }

    // Start
    debug!(
        "Received new upload request on /json\nUsing id: {id}\nUsername: {}\nFile ext: {}\nFile size: {}",
        metadata.username,
        metadata.file_ext,
        rocket::data::ByteUnit::Byte(file_data.len() as u64)
    );

    // Decode user input | Decoding makes the compression 'faster' koz it has less data to compress
    // let file_content = file_data.clone().into_bytes();
    let Ok(file_content) = rbase64::decode(file_data) else {
        error!("[{id}] Could not decode request");
        return crate::response::JsonApiResponseBuilder::default()
            .with_json(
                json!({"result": "failled", "message": "Could not understand the given data."}),
            )
            .with_status(Status::BadRequest)
            .build();
    };

    let mut c = cache.write().await;
    warn!("Cache size: {}", c.data.len());
    let exec = c.store(id, metadata, file_content);

    if wait_store {
        debug!("[{id}] Waiting for cache to finish storing the data");
        if let Err(e) = exec.await.unwrap() {
            error!("[{id}] An error occured while storing the given data: {e}");
        }
    }

    debug!(
        "[{id}] Responded in {}",
        time::format(start_timer.elapsed())
    );

    crate::response::JsonApiResponseBuilder::default()
        .with_json(json!({"result": "created", "file_name": id.hyphenated().to_string()}))
        .with_status(Status::Created)
        .build()
}

#[rocket::post("/", data = "<file>")]
pub async fn basic_upload(file: rocket::Data<'_>) -> String {
    // I wonder if there is a way to get the file name..
    let stream = file.open(rocket::data::ByteUnit::Kilobyte(128));
    let Ok(buff) = stream.into_bytes().await else {
        return "Failled to unpack the file".to_string();
    };
    format!("Received file with len: {}", buff.len())
}
