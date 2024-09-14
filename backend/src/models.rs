use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Project {
    pub id: String,
    pub project_name: String,
    pub thumbnail_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub project_name: String,
    pub fps: usize,
    pub scale: String,
    pub video_file_extension: String,
    pub latest_long_exposure_image_name: Option<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct FrameData {
    pub frame_number: usize,
    pub frame_weight: f32,
}
#[derive(Deserialize)]
pub struct CreateLongExposureImageRequest {
    pub frames_to_include: Vec<FrameData>,
}

#[derive(Debug, Serialize)]
pub struct GetProjectsResponse {
    pub projects: Vec<Project>,
}
#[derive(Debug, Serialize)]
pub struct UploadVideoResponse {
    pub message: &'static str,
    pub project_id: String,
}