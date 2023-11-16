use rocket::{
    http::{ContentType, Status},
    response::Responder,
    serde::json::Json,
    Request, Response,
};

#[derive(Debug, PartialEq, Eq, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct InputData {
    username: String,
    password: String,
    // file: String,
}

#[derive(Debug)]
pub struct ApiResponse<T> {
    json: Json<T>,
    status: Status,
}

impl<'r, T: rocket::serde::Serialize> Responder<'r, 'static> for ApiResponse<T> {
    fn respond_to(self, req: &Request) -> rocket::response::Result<'static> {
        Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

#[rocket::post("/json", format = "application/json", data = "<data>")]
pub async fn upload_json(data: Json<InputData>) -> ApiResponse<String> {
    println!("{data:?}");
    ApiResponse {
        json: format!(
            "{{'username': '{u}', 'Password': '{pw}'}}",
            u = data.username,
            pw = data.password
        )
        .into(),
        status: Status::Ok,
    }
}
