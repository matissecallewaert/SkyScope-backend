use rocket::response::status::BadRequest;
use std::fs;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tokio::task;

#[post("/label")]
pub async fn start_label_tool() -> Result<&'static str, BadRequest<Option<String>>> {
    let save_dir = Path::new("assets/uploads/labeled_images");

    if !save_dir.exists() {
        match fs::create_dir_all(save_dir) {
            Ok(_) => {}
            Err(e) => {
                return Err(BadRequest(Some(format!(
                    "Failed to create directory: {}",
                    e
                ))))
            }
        }
    }

    if let Err(error) = check_pip_version().await {
        return Err(BadRequest(Some(error)));
    }

    if let Err(error) = check_installation().await {
        return Err(BadRequest(Some(error)));
    }

    let result = Command::new("label-studio")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    if let Err(error) = result {
        return Err(BadRequest(Some(format!(
            "Failed to start label-studio: {}",
            error
        ))));
    }

    Ok("Label Studio started")
}

async fn check_pip_version() -> Result<(), String> {
    let output = task::spawn_blocking(|| {
        Command::new("python")
            .arg("-m")
            .arg("pip")
            .arg("--version")
            .output()
    })
    .await
    .map_err(|_| "Failed to execute command".to_string())?;

    if output.await.unwrap().status.success() {
        println!("pip is installed");
        return Ok(());
    }

    Err("pip is not installed".to_string())
}

async fn check_installation() -> Result<(), String> {
    let output = task::spawn_blocking(|| {
        Command::new("pip")
            .arg("install")
            .arg("label-studio")
            .output()
    })
    .await
    .map_err(|_| "Failed to execute command".to_string())?;

    if output.await.unwrap().status.success() {
        println!("label-studio installed");
        return Ok(());
    }

    Err("Label-studio could not be installed".to_string())
}
