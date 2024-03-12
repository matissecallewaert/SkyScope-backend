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
use routes::object_routes::get_object_list;

use rocket::fs::FileServer;
use std::fs;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/myrocket")]
fn myrocket() -> String {
    "My 🚀 server".to_string()
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
            routes![
                index,
                myrocket,
                get_uploaded_images,
                upload_image,
                delete_image,
                get_object_list,
                start_label_tool
            ],
        )
        .mount("/images", FileServer::from("assets/uploads/images"))
        .launch()
        .await
    {
        println!("Failed to launch Rocket: {}", e);
    }
}
