use std::path::PathBuf;
use actix_web::{delete, get, HttpResponse, web};
use futures_util::{FutureExt, StreamExt};
use image::imageops::thumbnail;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::log::error;
use crate::routes::create_long_exposure_image::create_long_exposure_image_request;
use crate::utils::{convert_image_path_to_serving_url, get_output_dir, get_upload_dir};


#[derive(Debug, Serialize)]
struct GetProjectsResponse {
    projects: Vec<Project>,
}

#[derive(Debug, Serialize)]
struct Project {
    id: String,
    thumbnail_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub fps: usize,
    pub scale: String,
    pub video_file_extension: String,
    pub latest_long_exposure_image_name: Option<String>,
}

#[get("/projects")]
pub async fn get_projects() -> HttpResponse {
    match fetch_projects().await {
        Ok(projects) => HttpResponse::Ok().json(GetProjectsResponse { projects }),
        Err(err) => {
            error!("An error occurred while fetching projects: {}", err);
            HttpResponse::InternalServerError().body(format!("An error occurred while fetching projects: {}", err))
        }
    }
}

#[get("/projects/{id}")]
pub async fn get_project_metadata(project_id: web::Path<String>) -> HttpResponse {
    let id = project_id.into_inner();
    match fetch_project_metadata(&id).await {
        Ok(metadata) => HttpResponse::Ok().json(metadata),
        Err(err) => {
            error!("An error occurred while fetching project metadata: {}", err);
            HttpResponse::InternalServerError().body(format!("An error occurred while fetching project metadata: {}", err))
        }
    }
}

#[delete("/projects/{id}")]
pub async fn delete_project(project_id: web::Path<String>) -> HttpResponse {
    let id = project_id.into_inner();
    match delete_project_by_id(&id).await {
        Ok(_) => HttpResponse::Ok().body(format!("Project {} deleted successfully", id)),
        Err(err) => {
            error!("An error occurred while deleting the project: {}", err);
            HttpResponse::InternalServerError().body(format!("An error occurred while deleting the project: {}", err))
        }
    }
}

async fn fetch_project_metadata(project_id: &str) -> Result<ProjectMetadata, std::io::Error> {
    let output_dir = get_output_dir();
    let metadata_file_path = output_dir.join(project_id).join("metadata.json");

    if metadata_file_path.exists() {
        let metadata_content = fs::read_to_string(metadata_file_path).await?;
        let metadata: ProjectMetadata = serde_json::from_str(&metadata_content)?;
        Ok(metadata)
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Metadata file not found"))
    }
}

async fn fetch_projects() -> Result<Vec<Project>, std::io::Error> {
    let output_dir = get_output_dir();
    let mut projects = Vec::new();
    let mut stream = fs::read_dir(output_dir).await?;

    while let Some(entry) = stream.next_entry().await? {
        let path = entry.path();
        if path.is_dir() {
            if let Some(project) = process_project_dir(&path).await {
                projects.push(project);
            }
        }
    }

    Ok(projects)
}

/*
    Retrieves the project Id and a path to either a long exposure image (if exists) or the first thumbnail
*/
async fn process_project_dir(path: &std::path::Path) -> Option<Project> {
    let dir_name = path.file_name()?.to_str()?;
    let thumbnail_path = find_thumbnail(path).await?;
    if !fs::try_exists(&thumbnail_path).await.unwrap() {
        return None;
    }
    let serving_url = convert_image_path_to_serving_url(&thumbnail_path).await;

    Some(Project {
        id: dir_name.to_string(),
        thumbnail_path: serving_url,
    })
}

/*
    Returns either a path to a long exposure image(if exists) or to a thumbnail.
 */
async fn find_thumbnail(movie_dir_output_path: &std::path::Path) -> Option<PathBuf> {
    if let Some(path) = find_long_exposure_image(movie_dir_output_path).await {
        return Some(path);
    } else {
        Some(movie_dir_output_path.join("frames/ffout_thumbnail_0001.webp"))
    }
}

async fn find_long_exposure_image(path: &std::path::Path) -> Option<PathBuf> {
    if let Ok(mut sub_dir_stream) = fs::read_dir(path).await {
        while let Ok(Some(sub_entry)) = sub_dir_stream.next_entry().await {
            let sub_path = sub_entry.path();
            if sub_path.is_file() {
                if let Some(file_name) = sub_path.file_name().and_then(|name| name.to_str()) {
                    if file_name.starts_with("long_exposure_image_") && file_name.ends_with(".png") {
                        return Some(sub_path);
                    }
                }
            }
        }
    }
    None
}

async fn delete_project_by_id(project_id: &str) -> Result<(), std::io::Error> {
    let output_dir = get_output_dir();
    let upload_dir = get_upload_dir();
    let uploaded_video_path = upload_dir.join(project_id);
    let project_dir_path = output_dir.join(project_id);

    if project_dir_path.try_exists().unwrap() && project_dir_path.is_dir() {
        // Delete the entire project directory and its contents
        fs::remove_dir_all(project_dir_path).await?;

        if uploaded_video_path.try_exists().unwrap() {
            fs::remove_file(uploaded_video_path).await?
        }
        Ok(())
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Project directory not found"))
    }
}