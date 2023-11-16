// #[macro_use]
// extern crate rocket;

mod basic;
mod json;

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
    rocket::build().mount(
        "/",
        rocket::routes![index, basic::upload, json::upload_json, download],
    )
}
