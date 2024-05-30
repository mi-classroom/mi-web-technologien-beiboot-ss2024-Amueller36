use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use actix_web::web::BytesMut;
use futures_util::TryStreamExt;
use regex::Regex;
use tokio::fs;
use tracing::trace;

pub async fn read_text_from_field(mut field: actix_multipart::Field) -> String {
    let mut data = BytesMut::new();
    while let Some(chunk) = field.try_next().await.unwrap() {
        data.extend_from_slice(&chunk);
    }
    String::from_utf8(data.to_vec()).unwrap()
}

pub fn get_upload_dir() -> PathBuf {
    let dir = std::env::var("MOVIE_UPLOAD_DIR").unwrap_or("./media/uploads/".to_string());
    if !dir.ends_with("/") {
        PathBuf::from(dir + "/")
    } else {
        PathBuf::from(dir)
    }
}

pub fn get_output_dir() -> PathBuf {
    let dir =
        std::env::var("LONG_EXPOSURE_IMG_OUTPUT_DIR").unwrap_or("./media/outputs/".to_string());
    if !dir.ends_with("/") {
        PathBuf::from(dir + "/")
    } else {
        PathBuf::from(dir)
    }
}

pub async fn create_directory_if_not_created_yet(path: &str) {
    let path = Path::new(path);
    let dir_result = fs::create_dir_all(path).await;
    if let Err(error_code) = dir_result {
        match error_code.kind() {
            ErrorKind::NotFound => {
                panic!("Tried to create a directory at path '{}' but got NotFound, please manually create those folders.", path.display())
            }
            ErrorKind::PermissionDenied => {
                panic!("Tried to create a directory at path '{}' but got PermissionDenied, please manually create those folders.", path.display())
            }
            ErrorKind::AlreadyExists => {
                trace!("Directory path {} already exists, no need for recreation.", path.display());
            }
            _ => panic!("An unexpected error occurred while trying to create a directory at path '{}'. Error: {}", path.display(), error_code)
        }
    }
}

pub fn convert_image_path_to_serving_url(image_path: &PathBuf) -> String {
    let domain = std::env::var("DOMAIN").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let path = image_path.to_str().unwrap().to_string();
    let path = path
        .replace("\\", "/")
        .replace("./media", "")
        .replace("frames", "")
        .replace("..", "");

    // Remove the first character if it is a slash
    let final_path = if path.starts_with("/") {
        path[1..].to_string()
    } else {
        path
    };

    format!("{}/{}", domain, final_path)
}
