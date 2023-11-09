#[macro_use] extern crate rocket;


use rocket::data::{ByteUnit};


const MAX_FILE_SIZE: ByteUnit = ByteUnit::Kilobyte(128);

#[post("/", data = "<file>")]
async fn upload(file: rocket::Data<'_>) -> String{
    
    // I wonder if there is a way to get the file name..
    let stream = file.open(MAX_FILE_SIZE);
    let Ok(buff) = stream.into_bytes().await else{
        return "Failled to unpack the file".to_string()
    };
    format!("Received file with len: {}", buff.len())
}


#[get("/<id>")]
async fn download(id: String) -> String{
    format!("Could not fetch {id}, this part is not done yet :)")
}

#[get("/")]
fn index() -> &'static str{
    "
    How to use:
        curl http://localhost:8000 --data-binary \"@transfer.bat\"
        From stdin (in cmd, koz buggy in pwsh):
        bat .\\transfer.bat | curl -X POST --data-binary @- localhost:8000
    "
}



#[launch]
fn rocket() -> _{
    rocket::build().mount("/", routes![index, upload, download])
}