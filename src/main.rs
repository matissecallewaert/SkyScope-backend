#[macro_use]
extern crate rocket;

mod cors;
mod routes;
use cors::CORS;

mod structs {
    pub mod image;
}

use routes::image_routes::{delete_image, get_uploaded_images, upload_image};
use routes::label_routes::start_label_tool;
use routes::object_routes::{get_object_list, delete_object};

use rocket::fs::FileServer;
use rocket::{options, response::status::NoContent};

use std::fs;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[options("/<_..>")]
fn cors_preflight() -> NoContent {
    NoContent
}

#[rocket::main]
async fn main() {
    if let Err(e) = fs::create_dir_all("assets/uploads/images") {
        println!("Failed to create directory: {}", e);
        return;
    }

    if let Err(e) = fs::create_dir_all("assets/object_list") {
        println!("Failed to create directory: {}", e);
        return;
    }

    if let Err(e) = rocket::build()
        .attach(CORS)
        .mount(
            "/api",
            routes![
                cors_preflight,
                index,
                get_uploaded_images,
                upload_image,
                delete_image,
                get_object_list,
                start_label_tool,
                delete_object
            ],
        )
        .mount("/images", FileServer::from("assets/uploads/images"))
        .launch()
        .await
    {
        println!("Failed to launch Rocket: {}", e);
    }
}
