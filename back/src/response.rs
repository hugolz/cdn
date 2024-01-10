use rocket::{
    http::{ContentType, Status},
    serde::json::serde_json::{json, Value as JsonValue},
};

pub struct JsonApiResponse {
    json: JsonValue,
    status: Status,
    headers: std::collections::HashMap<String, String>,
}

impl<'r> rocket::response::Responder<'r, 'static> for JsonApiResponse {
    fn respond_to(self, req: &rocket::Request) -> rocket::response::Result<'static> {
        let mut resp = rocket::Response::build_from(self.json.respond_to(req).unwrap());

        let mut resp = resp.status(self.status);

        for (name, value) in self.headers {
            resp = resp.raw_header(name, value);
        }

        let out = resp.ok();
        trace!("{out:?}");

        out
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

    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.inner
            .headers
            .insert(name.to_string(), value.to_string());
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
                headers: {
                    let mut h = std::collections::HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());

                    // Unstable be carefull
                    h.insert("Access-Control-Allow-Origin".to_string(), "http://localhost:3000".to_string());
                    h.insert("Access-Control-Allow-Method".to_string(), "POST,GET,OPTIONS".to_string());
                    h.insert("Access-Control-Allow-Headers".to_string(), "X-PINGOTHER, Content-Type".to_string());
                    h
                },
            },
        }
    }
}

pub struct Response {
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
