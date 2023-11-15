// #[macro_use]
// extern crate rocket;
use rocket::data::ByteUnit;
use rocket::serde::json::Json;

#[derive(Debug, PartialEq, Eq, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
struct UserData {
    username: String,
    password: String,
    // file: String,
}

const MAX_FILE_SIZE: ByteUnit = ByteUnit::Kilobyte(128);

#[rocket::post("/json", format = "application/json", data = "<file>")]
async fn upload_json(file: Json<UserData>) -> String {
    println!("{file:?}");
    format!("Username: {u}\nPassword: {pw}", u = file.username, pw = file.password)
}

#[rocket::post("/", data="<file>")]
async fn upload(file: rocket::Data<'_>) -> String{
    // I wonder if there is a way to get the file name..
    let stream = file.open(MAX_FILE_SIZE);
    let Ok(buff) = stream.into_bytes().await else {
        return "Failled to unpack the file".to_string();
    };
    format!("Received file with len: {}", buff.len())
}

#[rocket::get("/<id>")]
async fn download(id: String) -> String {
    format!("Could not fetch {id}, this part is not done yet :)")
}

#[rocket::get("/")]
fn index() -> &'static str {
    "
    How to use:
        File content:
            From file name:
                curl http://localhost:8000/ --request POST --data-binary \"@file.txt\"
            From stdin (in cmd, koz buggy in pwsh):            
                cat file.txt | curl http://localhost:8000 --request POST --data-binary @-
        Json: (Not finished yet, and again in cmd koz buggy in pwsh)
            curl http://localhost:8000/json --header \"Content-Type: application/json\" --request POST --data \"{\\\"username\\\": \\\"xyz\\\", \\\"password\\\": \\\"xyz\\\"}\" 
    "
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes![index, upload, upload_json, download])
}
