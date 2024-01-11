use {
    crate::response::Response,
    rocket::http::{ContentType, Status},
    std::net::SocketAddr,
};

#[path = "routes/dashboard.rs"]
mod dashboard_route;
#[path = "routes/download.rs"]
mod download_route;
#[path = "routes/upload.rs"] // Naming conflict in main when registering route
mod upload_route;

#[allow(unused_imports)] // Used by main.rs
pub use dashboard_route::*;
#[allow(unused_imports)] // Used by main.rs
pub use download_route::*;
#[allow(unused_imports)] // Used by main.rs
pub use upload_route::*;

#[rocket::get("/")]
pub async fn root(remote_addr: SocketAddr) -> Response {
    let _old_msg = "

        Hi, please take a look at the /examples directory to understand how to use this api
    ";

    file_response("index.html", ContentType::HTML, remote_addr)
}

#[rocket::get("/style.css")]
pub async fn style(remote_addr: SocketAddr) -> Response {
    file_response("style.css", ContentType::CSS, remote_addr)
}

#[rocket::get("/front.js")]
pub async fn front(remote_addr: SocketAddr) -> Response {
    file_response("front.js", ContentType::JavaScript, remote_addr)
}

#[rocket::get("/front_bg.wasm")]
pub fn wasm(remote_addr: SocketAddr) -> Response {
    file_response("front_bg.wasm", ContentType::WASM, remote_addr)
}

fn file_response(file_name: &str, content_type: ContentType, remote_addr: SocketAddr) -> Response {
    match read_static(file_name, remote_addr) {
        Some(bytes) => Response {
            status: Status::Ok,
            content: bytes,
            content_type: content_type,
        },
        None => Response {
            status: Status::InternalServerError,
            content: Vec::new(),
            content_type: ContentType::Plain,
        },
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

#[rocket::options("/upload")]
pub fn upload_option() -> crate::response::JsonApiResponse {
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
