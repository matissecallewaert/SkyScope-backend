use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::path::Path;

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
