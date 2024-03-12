use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use std::path::Path;
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncBufReadExt, BufReader};

#[get("/get-object-list")]
pub async fn get_object_list() -> Result<Json<Vec<String>>, Custom<&'static str>> {
    let object_list_path = Path::new("assets/object_list/object_list.txt");

    if let Some(parent) = object_list_path.parent() {
        fs::create_dir_all(parent).await.map_err(|_| {
            Custom(
                Status::InternalServerError,
                "Cannot create directory for object list",
            )
        })?;
    }

    let file = OpenOptions::new()
        .read(true)
        .create(true) // This option ensures the file is created if it does not exist
        .open(object_list_path)
        .await
        .map_err(|_| {
            Custom(
                Status::InternalServerError,
                "Cannot open or create object list file",
            )
        })?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut objects = Vec::new();

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
