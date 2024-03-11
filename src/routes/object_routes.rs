use rocket::serde::json::Json;

use rocket::data::{Data, ToByteUnit};
use rocket::http::Status;
use rocket::response::status::Custom;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

#[get("/object_list")]
pub async fn get_object_list() -> Result<Json<Vec<String>>, Custom<&'static str>> {
    let object_list_path = Path::new("assets/object_list/object_list.txt");
    let mut objects = Vec::new();

    let file = File::open(object_list_path)
        .await
        .map_err(|_| Custom(Status::InternalServerError, "Cannot open object list file"))?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await.map_err(|_| {
        Custom(
            Status::InternalServerError,
            "Failed to read line from object list file",
        )
    })? {
        objects.push(line.to_string());
    }

    Ok(Json(objects))
}

#[post("/add_object", data = "<object_name>")]
pub async fn add_object_to_list(
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
                Err(_) => {
                    return Err(Custom(
                        rocket::http::Status::InternalServerError,
                        "Failed to open file",
                    ))
                }
            };

            // Write the string to the file, appending a newline
            if let Err(_) = writeln!(file, "{}", string.into_inner()) {
                return Err(Custom(
                    rocket::http::Status::InternalServerError,
                    "Failed to write to file",
                ));
            }

            Ok("Object Uploaded Successfully")
        }
        _ => Err(Custom(rocket::http::Status::BadRequest, "Invalid data")),
    }
}
