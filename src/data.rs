use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, rocket::FromForm, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Metadata {
    pub username: String,
    pub file_ext: String,
    // ...
}

#[derive(Debug, Serialize, Deserialize, rocket::FromForm)]
#[serde(crate = "rocket::serde")]
pub struct UploadData {
    pub metadata: Metadata,
    pub file: String,
}
