use rocket::{
    http::{ContentType, Status},
    response::Responder,
    serde::{
        json::{
            serde_json::{self, json, Value as JsonValue},
            Json,
        },
        Deserialize, Serialize,
    },
    tokio::io::AsyncWriteExt,
};

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

#[derive(Debug)]
pub struct JsonApiResponse {
    json: JsonValue,
    status: Status,
}

impl<'r> Responder<'r, 'static> for JsonApiResponse {
    fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'static> {
        rocket::Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[rocket::post("/json", format = "application/json", data = "<data>")]
pub async fn upload_json(data: Json<UploadData>) -> JsonApiResponse {
    println!(
        "Received user data: {:?}, File size: {} bytes",
        data.metadata,
        data.file.len()
    );
    let id = uuid::Uuid::new_v4();

    let mut file = match rocket::tokio::fs::File::create(format!("./cache/{id}.json")).await {
        Ok(file) => file,
        Err(e) => panic!("Could not create file: {e}"),
    };
    let cache_content = json!({
        "user": data.metadata.username,
        "file_ext": data.metadata.file_ext,
        "content": data.file
    });

    if let Err(e) = file
        .write_all(
            serde_json::to_string_pretty(&cache_content)
                .unwrap()
                .as_bytes(),
        )
        .await
    {
        panic!("{e}")
    }

    JsonApiResponse {
        json: json!({"result": "created", "file_name": id.hyphenated().to_string()}),
        status: Status::Created,
    }
}

#[rocket::catch(400)]
pub async fn upload_json_400(_req: &rocket::Request<'_>) -> JsonApiResponse {
    JsonApiResponse {
        json: json!({"status": 400, "message": "Could not understand the given data."}),
        status: Status::from_code(400).unwrap(),
    }
}
