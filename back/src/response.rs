use rocket::{
    http::{ContentType, Status},
    serde::json::serde_json::{json, Value as JsonValue},
};

pub struct JsonApiResponse {
    json: JsonValue,
    status: Status,
}

impl<'r> rocket::response::Responder<'r, 'static> for JsonApiResponse {
    fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'static> {
        rocket::Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

pub struct JsonApiResponseBuilder {
    inner: JsonApiResponse,
}

impl JsonApiResponseBuilder {
    pub fn with_json(mut self, value: JsonValue) -> Self {
        self.inner.json = value;
        self
    }

    pub fn with_status(mut self, status: Status) -> Self {
        self.inner.status = status;
        self
    }

    pub fn build(self) -> JsonApiResponse {
        self.inner
    }
}

impl Default for JsonApiResponseBuilder {
    fn default() -> Self {
        JsonApiResponseBuilder {
            inner: JsonApiResponse {
                json: json!({}),
                status: Status::Ok,
            },
        }
    }
}


pub struct Response{
    pub status: Status,
    pub content: Vec<u8>,
    pub c_type: rocket::http::ContentType,
}

impl<'r> rocket::response::Responder<'r, 'static> for Response {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        rocket::Response::build()
            .header(self.c_type)
            .status(self.status)
            .sized_body(self.content.len(), std::io::Cursor::new(self.content))
            .ok()
    }
}