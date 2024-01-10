use std::net::SocketAddr;

use rocket::http::Status;

mod dashboard;
mod download;
mod upload;

#[allow(unused_imports)] // Used by main.rs
pub use dashboard::*;
#[allow(unused_imports)] // Used by main.rs
pub use download::*;
#[allow(unused_imports)] // Used by main.rs
pub use upload::*;

#[rocket::get("/")]
pub async fn root(remote_addr: SocketAddr) -> crate::response::Response {
    let msg = "
        Hi, please take a look at the /examples directory to understand how to use this api
    ";
    let buffer = read_static("index.html", remote_addr).unwrap();

    crate::response::Response {
        status: Status::Ok,
        content: buffer,
        c_type: rocket::http::ContentType::HTML,
    }
}

#[rocket::get("/style.css")]
pub async fn style(remote_addr: SocketAddr) -> crate::response::Response {
    let buffer = read_static("style.css", remote_addr).unwrap();

    crate::response::Response {
        status: Status::Ok,
        content: buffer,
        c_type: rocket::http::ContentType::CSS,
    }
}

#[rocket::get("/front.js")]
pub async fn front(remote_addr: SocketAddr) -> crate::response::Response {
    let buffer = read_static("front.js", remote_addr).unwrap();

    crate::response::Response {
        status: Status::Ok,
        content: buffer,
        c_type: rocket::http::ContentType::JavaScript,
    }
}

#[rocket::get("/front_bg.wasm")]
pub fn wasm(remote_addr: SocketAddr) -> crate::response::Response {
    let buffer = read_static("front_bg.wasm", remote_addr).unwrap();
    crate::response::Response {
        status: Status::Ok,
        content: buffer,
        c_type: rocket::http::ContentType::WASM,
    }
}

fn read_static(file_name: &str, remote_addr: SocketAddr) -> Option<Vec<u8>> {
    use std::io::Read as _;
    trace!("New static file query from {remote_addr}: {file_name}");
    let mut buffer = Vec::new();
    let _size = std::fs::File::open(format!("./static/{file_name}"))
        .ok()?
        .read_to_end(&mut buffer)
        .ok()?;
    Some(buffer)
}


#[rocket::options("/json")]
pub fn option_json() -> crate::response::JsonApiResponse {
    /*
        We're currently having issues connecting a NextJs sevrer to this storage server

        we belive that his might help
        but we have no idea what to set here and in the NextJs config

        The thing is that test_upload (in front/main.rs) works fine, and do somewaht the same thing as the NextJs

        CORS errors..
    */
    warn!("option req");
    crate::response::JsonApiResponseBuilder::default()
        .with_status(Status::NoContent)
        .with_header("Content-Type", "text/plain")
        // .with_header("Access-Control-Allow-Origin", "*")
        // .with_header("Access-Control-Allow-Method", "POST")
        // .with_header("Access-Control-Allow-Headers", "X-PINGOTHER, Content-Type")


        
        // .with_header("Content-Type", "text/plain")
        // .with_header("Access-Control-Allow-Origin", "*")
        // .with_header("Access-Control-Allow-Cedentials", "true")
        // .with_header("Access-Control-Expose-Headers", "*")
        // .with_header("Access-Control-Max-Age", "5")
        // .with_header("Access-Control-Allow-Method", "POST,OPTIONS,GET")
        // .with_header(
        //     "Access-Control-Allow-Headers",
        //     "Content-Type",
        // )
        .build()
}
