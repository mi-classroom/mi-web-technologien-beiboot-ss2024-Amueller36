use std::env;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use actix_web::web::BytesMut;
use futures_util::TryStreamExt;
use regex::Regex;
use serde_json::to_string_pretty;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncBufReadExt;
use tracing::trace;
use crate::routes::projects::ProjectMetadata;

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
pub async fn convert_image_path_to_serving_url(image_path: &PathBuf) -> String {
    let domain = env::var("DOMAIN").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Resolve the absolute path and normalize it
    let absolute_path = fs::canonicalize(image_path).await.unwrap();

    // Convert the path to a string and replace backslashes with forward slashes
    let path_str = absolute_path.to_str().unwrap().replace("\\", "/");

    // Split the path at /media/ and take the second part
    let clean_path = path_str.splitn(2, "/media/").nth(1).unwrap_or(&path_str);

    // Remove leading slashes
    let final_path = clean_path.trim_start_matches('/');

    format!("{}/{}", domain, final_path)
}

pub fn save_metadata(metadata: &ProjectMetadata, metadata_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Open the file with write and truncate options
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(metadata_path)
        .expect("Failed to open metadata.json");

    // Serialize the metadata to a pretty JSON string
    let serialized_metadata = to_string_pretty(metadata)?;

    // Write the serialized metadata to the file
    file.write_all(serialized_metadata.as_bytes())?;

    Ok(())
}