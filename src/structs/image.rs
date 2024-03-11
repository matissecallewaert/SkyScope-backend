use rocket::serde::Serialize;

#[derive(Serialize)]
pub struct Image {
    pub path: String,
    pub timestamp: String,
}
