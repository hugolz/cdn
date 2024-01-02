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
