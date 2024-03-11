#[macro_use]
extern crate rocket;

mod cors;
use cors::CORS;

mod structs {
    pub mod image;
}
use crate::structs::image::Image;

use rocket::serde::json::Json;

use chrono::{DateTime, Utc};
use rocket::data::{Data, ToByteUnit};
use rocket::fs::FileServer;
use rocket::http::{ContentType, Status};
use rocket::response::status::Custom;
use std::fs;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::io::{BufReader, AsyncBufReadExt};
use multer::Multipart;
use std::fs::OpenOptions;
use std::io::Write;
use uuid::Uuid;


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/myrocket")]
fn myrocket() -> String {
    "My 🚀 server".to_string()
}

#[get("/uploaded_images")]
async fn get_uploaded_images() -> Result<Json<Vec<Image>>, Custom<&'static str>> {
    let image_dir = Path::new("assets\\uploads\\images");
    let mut images = Vec::new();

    let entries = fs::read_dir(image_dir).map_err(|_| {
        Custom(
            Status::InternalServerError,
            "Failed to read the image directory",
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|_| {
            Custom(
                Status::InternalServerError,
                "Failed to read an entry in the image directory",
            )
        })?;

        let path = entry.path();

        let url_path = path
            .strip_prefix(image_dir)
            .map_err(|_| {
                Custom(
                    Status::InternalServerError,
                    "Failed to strip image directory prefix",
                )
            })?
            .to_string_lossy();

        let image_url = format!("/images/{}", url_path);

        let metadata = fs::metadata(&path)
            .map_err(|_| Custom(Status::InternalServerError, "Failed to read file metadata"))?;

        let modified_time = metadata.modified().map_err(|_| {
            Custom(
                Status::InternalServerError,
                "Failed to read file modified time",
            )
        })?;

        // Convert SystemTime to DateTime<Utc>
        let datetime: DateTime<Utc> = modified_time.into();
        // Format the timestamp as a string
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

        let image = Image {
            path: image_url,
            timestamp: timestamp_str,
        };

        images.push(image);
    }

    Ok(Json(images))
}

#[post("/upload_image", data = "<file>")]
async fn upload_image(
    content_type: &ContentType,
    file: Data<'_>,
) -> Result<&'static str, Custom<&'static str>> {
    if !content_type.is_form_data() {
        return Err(Custom(
            Status::BadRequest,
            "Content type is not multipart/form-data",
        ));
    }

    let (_, boundary) = content_type
        .params()
        .find(|&(k, _)| k == "boundary")
        .ok_or_else(|| {
            Custom(
                Status::BadRequest,
                "Content-Type: multipart/form-data without boundary",
            )
        })?;

    let image_dir = Path::new("assets/uploads/images");

    fs::create_dir_all(image_dir).map_err(|_| {
        Custom(
            Status::InternalServerError,
            "Failed to create image directory",
        )
    })?;

    let mut multipart = Multipart::with_reader(file.open(2.mebibytes()), boundary);

    // Iterate over the fields
    while let Ok(Some(mut field)) = multipart.next_field().await {
        println!("Field name: {:?}", field.name());
        println!("Field filename: {:?}", field.file_name()); 
        println!("Field content-type: {:?}", field.content_type());

        let file_path = if let Some(filename) = field.file_name() {
            image_dir.join(filename /*TODO: Filename needs to be sanitized*/)
        } else {
            image_dir.join(Uuid::new_v4().to_string() + ".jpg")
        };

        println!("File path: {:?}", file_path); // Debug: print the file path

        let mut file = File::create(&file_path)
            .await
            .map_err(|_| Custom(Status::InternalServerError, "Failed to create file"))?;

        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|_| Custom(Status::InternalServerError, "Error reading chunk"))?
        {
            file.write_all(&chunk)
                .await
                .map_err(|_| Custom(Status::InternalServerError, "Failed to write to file"))?;
        }
    }

    Ok("File uploaded successfully")
}

#[post("/delete_image/<image_id>")]
fn delete_image(image_id: &str) -> Result<&'static str, (Status, String)> {
    let path = Path::new("assets/uploads/images").join(image_id);
    println!("Path: {:?}", path); 
    match fs::remove_file(path) {
        Ok(_) => Ok("Photo deleted successfully"),
        Err(e) => Err((Status::InternalServerError, format!("Failed to delete image: {}", e))),
    }
}

#[get("/object_list")]
async fn get_object_list() -> Result<Json<Vec<String>>, Custom<&'static str>> {
    let object_list_path = Path::new("assets/object_list/object_list.txt");
    let mut objects = Vec::new();

    let file = File::open(object_list_path).await.map_err(|_| {
        Custom(Status::InternalServerError, "Cannot open object list file")
    })?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();    

    while let Some(line) = lines.next_line().await.map_err(|_| {
        Custom(Status::InternalServerError, "Failed to read line from object list file")
    })? {
        objects.push(line.to_string());
    }

    Ok(Json(objects))
}

#[post("/add_object", data = "<object_name>")]
async fn add_object_to_list(
    object_name: Data<'_>,
) -> Result<&'static str, Custom<&'static str>> {

    // Define the path to the file
    let path = Path::new("assets/object_list/object_list.txt");
            
    // Convert Data to a String
    let string_data = object_name.open(2.mebibytes()).into_string().await;

    match string_data {
        Ok(string) if string.is_complete() => {
            let mut file = match OpenOptions::new()
                .create(true) // Create file if it does not exist
                .append(true) // Append to the file
                .open(&path)
            {
                Ok(file) => file,
                Err(_) => return Err(Custom(rocket::http::Status::InternalServerError, "Failed to open file")),
            };
            
            // Write the string to the file, appending a newline
            if let Err(_) = writeln!(file, "{}", string.into_inner()) {
                return Err(Custom(rocket::http::Status::InternalServerError, "Failed to write to file"));
            }
            
            Ok("Object Uploaded Successfully")
        },
        _ => Err(Custom(rocket::http::Status::BadRequest, "Invalid data")),
    }
}


#[rocket::main]
async fn main() {

    if let Err(e) = fs::create_dir_all("assets/uploads/images") {
        println!("Failed to create directory: {}", e);
        return;
    }
    
    if let Err(e) = rocket::build()
        .attach(CORS)
        .mount(
            "/api",
            routes![index, myrocket, get_uploaded_images, upload_image, delete_image, get_object_list, add_object_to_list],
        )
        .mount("/images", FileServer::from("assets/uploads/images"))
        .launch()
        .await
    {
        println!("Failed to launch Rocket: {}", e);
    }
}