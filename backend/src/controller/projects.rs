use std::ffi::OsStr;
use std::path::Path;
use std::str::FromStr;

use actix_multipart::Multipart;
use actix_web::{delete, get, HttpResponse, post, web};
use actix_web::web::BytesMut;
use futures_util::TryStreamExt;
use serde_json::json;
use tracing::error;
use tracing::log::info;
use uuid::Uuid;

use crate::models::{CreateLongExposureImageRequest, GetProjectsResponse};
use crate::services::long_exposure_image_service::create_long_exposure_image_svc;
use crate::services::projects_service::{delete_project_by_id, fetch_projects, process_upload};
use crate::utils::{read_metadata_from_project, read_text_from_field};

#[get("/projects")]
pub async fn get_projects() -> HttpResponse {
    match fetch_projects().await {
        Ok(projects) => HttpResponse::Ok().json(GetProjectsResponse { projects }),
        Err(err) => {
            error!("An error occurred while fetching projects: {}", err);
            HttpResponse::InternalServerError().body("An error occurred while fetching projects")
        }
    }
}

#[get("/projects/{id}")]
pub async fn get_project_metadata(project_id: web::Path<String>) -> HttpResponse {
    let id = project_id.into_inner();
    match read_metadata_from_project(&id).await {
        Ok(metadata) => HttpResponse::Ok().json(metadata),
        Err(err) => {
            error!("An error occurred while fetching project metadata: {}", err);
            HttpResponse::InternalServerError().body("An error occurred while fetching project metadata")
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
            HttpResponse::InternalServerError().body("An error occurred while deleting the project")
        }
    }
}

/**
This endpoint accepts multipart form data to either create a new project or update an existing one.
- If a `project_id` is provided, it updates the existing project, if a different fps or scale was provided, and skips uploading the video file.
- If no `project_id` is provided, it creates a new project and expects a video file to be uploaded.
*/
#[post("/projects")]
pub async fn create_or_update_project(mut payload: Multipart) -> HttpResponse {
    let mut scale = String::new();
    let mut project_name = String::new();
    let mut fps = 0;
    let mut video_data: Option<BytesMut> = None;
    let mut video_extension: Option<String> = None;
    let mut video_id: Option<Uuid> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap_or_default();

        match name {
            "project_id" => {
                let id = read_text_from_field(field).await;
                info!("Video id is {}", id);
                video_id = Some(Uuid::from_str(&id).unwrap());
            }
            "video_file" => {
                // If video_id is present, skip uploading the video file
                if video_id.is_some() {
                    continue;
                }
                if let Some(filename) = content_disposition.get_filename() {
                    let extension = Path::new(filename)
                        .extension()
                        .and_then(OsStr::to_str)
                        .map(|ext| ext.to_lowercase());
                    video_extension = extension;
                }
                let mut data = BytesMut::new();
                while let Some(chunk) = field.try_next().await.unwrap() {
                    data.extend_from_slice(&chunk);
                }
                video_data = Some(data);
            }
            "project_name" => {
                project_name = read_text_from_field(field).await;
            }
            "scale" => {
                scale = read_text_from_field(field).await;
            }
            "fps" => {
                fps = read_text_from_field(field)
                    .await
                    .parse::<usize>()
                    .expect("Conversion of FPS failed");
            }
            _ => error!("Unexpected field: {}", name),
        }
    }

    // Call the service function
    match process_upload(video_id, video_data, video_extension, project_name, scale, fps).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(err) => {
            error!("An error occurred while uploading project: {}", err);
            HttpResponse::InternalServerError().body(format!("An error occurred while uploading the project {:?}", err))
        }
    }
}

#[post("/projects/{project_id}/createLongExposureImage")]
pub async fn create_long_exposure_image(
    path: web::Path<String>,
    request_body: web::Json<CreateLongExposureImageRequest>,
) -> HttpResponse {
    let project_id = path.into_inner();
    let image_request = request_body.into_inner();

    match create_long_exposure_image_svc(
        project_id,
        image_request.frames_to_include,
    )
        .await
    {
        Ok(path_to_long_exposure_img) => HttpResponse::Ok().body(path_to_long_exposure_img),
        Err(e) => {
            error!("Error creating long exposure image: {:?}", e);
            HttpResponse::InternalServerError().json(
                json!({
                    "message": "An internal server error occurred while trying to create the long exposure image.",
                    "error": format!("{}", e)
                })
            )
        }
    }
}