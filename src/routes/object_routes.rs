use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::{Deserialize, json::Json};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::Path;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct DeleteRequest {
    object_name: String,
}

#[get("/get-object-list")]
pub fn get_object_list() -> Result<Json<Vec<String>>, Custom<&'static str>> {
    let object_list_path = Path::new("assets/object_list/object_list.txt");

    let file = match OpenOptions::new()
        .read(true)
        .write(true) // Necessary for creating a file
        .create(true)
        .open(object_list_path)
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open or create the file: {:?}", e);
            return Err(Custom(
                Status::InternalServerError,
                "Cannot open or create object list file",
            ));
        }
    };

    let reader = BufReader::new(file);
    let lines = reader.lines();

    let mut objects = Vec::new();
    for line in lines {
        match line {
            Ok(line) => objects.push(line),
            Err(e) => {
                eprintln!("Failed to read line: {:?}", e);
                return Err(Custom(
                    Status::InternalServerError,
                    "Failed to read line from object list file",
                ));
            }
        }
    }

    Ok(Json(objects))
}

// Rocket does not provide stable REST delete, use POST instead
#[post("/delete-object", format = "json", data = "<delete_request>")]
pub fn delete_object(delete_request: Json<DeleteRequest>) -> Result<&'static str, Custom<&'static str>> {
    let object_list_path = Path::new("assets/object_list/object_list.txt");

    let file = match OpenOptions::new().read(true).open(&object_list_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open the file: {:?}", e);
            return Err(Custom(
                Status::InternalServerError,
                "Cannot open object list file",
            ));
        },
    };

    let reader = BufReader::new(file);
    let objects: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| line.trim() != delete_request.object_name.trim())
        .collect();



    let file = match OpenOptions::new().write(true).truncate(true).open(&object_list_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open the file for writing: {:?}", e);
            return Err(Custom(
                Status::InternalServerError,
                "Cannot open object list file for writing",
            ));
        },
    };

    let mut writer = BufWriter::new(file);
    for object in objects {
        if let Err(e) = writeln!(writer, "{}", object) {
            eprintln!("Failed to write to the file: {:?}", e);
            return Err(Custom(
                Status::InternalServerError,
                "Failed to write to object list file",
            ));
        }
    }

    Ok("Object deleted successfully")
}

