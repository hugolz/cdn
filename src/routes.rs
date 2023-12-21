use rocket::{
    http::Status,
    serde::{
        json::{
            serde_json::{self, json},
            Json,
        },
        Deserialize, Serialize,
    },
    tokio::io::AsyncWriteExt,
};
use std::io::Write as _;

#[derive(Debug, Serialize, Deserialize, rocket::FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Metadata {
    username: String,
    file_ext: String,
    // ...
}

#[derive(Debug, Serialize, Deserialize, rocket::FromForm)]
#[serde(crate = "rocket::serde")]
pub struct UploadData {
    metadata: Metadata,
    file: String,
}

#[rocket::post("/json", format = "application/json", data = "<data>")]
pub async fn upload_json(data: Json<UploadData>) -> crate::response::JsonApiResponse {
    let start_timer = std::time::Instant::now();
    let id = uuid::Uuid::new_v4();
    let metadata = &data.metadata;
    debug!(
        "Received new upload request on /json\nUsing id: {id}\nUsername: {}\nFile ext: {}\nFile size: {}",
        metadata.username,
        metadata.file_ext,
        rocket::data::ByteUnit::Byte(data.file.len() as u64)
    );
    let Ok(file_content) = rbase64::decode(&data.file) else {
        error!("[{id}] Could not decode request");
        return crate::response::JsonApiResponseBuilder::default()
            .with_json(json!({"status": 400, "message": "Could not understand the given data."}))
            .with_status(Status::BadRequest)
            .build();
    };

    // let compressed_file = file_content;

    rocket::tokio::spawn(async move {
        let start_timer = std::time::Instant::now();
        let mut file = match rocket::tokio::fs::File::create(format!("./cache/{id}.json")).await {
            Ok(file) => file,
            Err(e) => {
                error!("[{id}] Could not create file: {e}");
                return;
            }
        };

        let mut compressed  = brotli::CompressorWriter::new(
            file.into_std().await,
            4096,
            11,
            22
        );


        if let Err(e) = compressed
            .write_all(
                serde_json::to_string_pretty(&json!({
                    "user": data.metadata.username,
                    "file_ext": data.metadata.file_ext,
                    "content": file_content
                }))
                .unwrap()
                .as_bytes(),
            )
        {
            error!("[{id}] Could not create file");

        }

        debug!("[{id}] Finished compresing the data, took {}", time::display_duration(start_timer.elapsed()))
    });


    debug!(
        "[{id}] Success in {}",
        time::display_duration(start_timer.elapsed())
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

#[rocket::get("/<id>")]
pub async fn download(id: &str) -> String {
    format!("Could not fetch {id}, this part is not done yet :)")
}

#[rocket::get("/")]
pub fn root() -> &'static str {
    "
        Hi, please take a look at the /examples directory to understand how to use this api
    "
}
