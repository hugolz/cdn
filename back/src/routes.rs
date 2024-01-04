use rocket::http::Status;

mod download;
mod upload;

#[allow(unused_imports)] // Used by main.rs
pub use download::*;
#[allow(unused_imports)] // Used by main.rs
pub use upload::*;

#[rocket::get("/")]
pub async fn root() -> crate::response::Response {
    let msg = "
        Hi, please take a look at the /examples directory to understand how to use this api
    ";
    let buffer = read_static("index.html").unwrap();

    crate::response::Response {
        status: Status::Ok,
        content: buffer,
        c_type: rocket::http::ContentType::HTML,
    }
}

#[rocket::get("/style.css")]
pub async fn style() -> crate::response::Response {
    let buffer = read_static("style.css").unwrap();

    crate::response::Response {
        status: Status::Ok,
        content: buffer,
        c_type: rocket::http::ContentType::CSS,
    }
}

#[rocket::get("/front.js")]
pub async fn front() -> crate::response::Response {
    let buffer = read_static("front.js").unwrap();

    crate::response::Response {
        status: Status::Ok,
        content: buffer,
        c_type: rocket::http::ContentType::JavaScript,
    }
}

#[rocket::get("/front_bg.wasm")]
pub fn wasm() -> crate::response::Response {
    let buffer = read_static("front_bg.wasm").unwrap();
    crate::response::Response {
        status: Status::Ok,
        content: buffer,
        c_type: rocket::http::ContentType::WASM,
    }
}

fn read_static(file_name: &str) -> Option<Vec<u8>> {
    use std::io::Read as _;
    let mut buffer = Vec::new();
    let _size = std::fs::File::open(format!("./static/{file_name}"))
        .ok()?
        .read_to_end(&mut buffer)
        .ok()?;
    Some(buffer)
}
