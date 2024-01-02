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
use std::io::Cursor;
use std::io::Write as _;

#[derive(Debug, Serialize, Deserialize, rocket::FromForm, Clone)]
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
    // Setup

    let start_timer = std::time::Instant::now();
    let id = uuid::Uuid::new_v4();
    let metadata = data.metadata.clone();
    let file_data = &data.file;

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

    // Decode user input
    let Ok(file_content) = rbase64::decode(file_data) else {
        error!("[{id}] Could not decode request");
        return crate::response::JsonApiResponseBuilder::default()
            .with_json(
                json!({"result": "failled", "message": "Could not understand the given data."}),
            )
            .with_status(Status::BadRequest)
            .build();
    };

    rocket::tokio::spawn(async move {
        let start_timer = std::time::Instant::now();
        let Ok(file) = rocket::tokio::fs::File::create(format!("./cache/{id}.data")).await else {
            error!("[{id}] Could not create data file");
            return;
        };
        let file_length = file_content.len();

        // Compression algorithms seems rly uneffective with most files

        match brotli::BrotliCompress(
            &mut Cursor::new(file_content),
            &mut file.into_std().await,
            &brotli::enc::BrotliEncoderParams::default(),
        ) {
            Ok(bytes) => {
                debug!(
                    "[{id}] Finished compressing {} -> {}",
                    rocket::data::ByteUnit::Byte(file_length as u64),
                    rocket::data::ByteUnit::Byte(bytes as u64)
                );
            }
            Err(e) => {
                error!("[{id}] Failled to compress due to: {e}")
            }
        }

        // match brotli::CompressorWriter::new(&mut file.into_std().await, 4096, 11, 24)
        //     .write_all(&file_content)
        // {
        //     Ok(bytes) => {
        //         debug!(
        //             "[{id}] Finished compressing {} -> {}",
        //             rocket::data::ByteUnit::Byte(file_content.len() as u64),
        //             rocket::data::ByteUnit::Byte(0 as u64)
        //         );
        //     }
        //     Err(e) => error!("[{id}] Failled to compress due to: {e}"),
        // }

        // if let Err(e) =

        // {
        //     error!("[{id}] Failled to compress due to: {e}");
        //     return;
        // }

        // let out = bincode::serialize(&compressed_file_content).unwrap();

        // if let Err(e) = file
        //     .write_all(
        //         &out,
        //     )
        //     .await
        // {
        //     error!("[{id}] Could not create data file due to: {e}");
        //     return;
        // }

        debug!(
            "[{id}] Successfully wrote its data, took {}",
            time::display_duration(start_timer.elapsed())
        );

        /* -------------------------------------------------------------------------------
                                    Meta file

            Has all the usefull infos about the data file.
            I's written at the end so the download method wont find partial data
        ------------------------------------------------------------------------------- */

        let mut file = match rocket::tokio::fs::File::create(format!("./cache/{id}.meta")).await {
            Ok(file) => file,
            Err(e) => {
                error!("[{id}] Could not create meta file: {e}");
                return;
            }
        };
        if let Err(e) = file
            .write_all(
                serde_json::to_string_pretty(&json!({
                    "username": metadata.username,
                    "extension": metadata.file_ext
                }))
                .unwrap()
                .as_bytes(),
            )
            .await
        {
            error!("[{id}] Could not write meta file due to: {e}");
        }
        debug!("[{id}] Successfully wrote meta file");
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
