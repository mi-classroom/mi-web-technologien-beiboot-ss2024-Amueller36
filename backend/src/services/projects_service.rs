use std::path::{Path, PathBuf};

use actix_web::web::BytesMut;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::log::{debug, error, info};
use uuid::Uuid;

use crate::models::{Project, ProjectMetadata, UploadVideoResponse};
use crate::utils::{convert_image_path_to_serving_url, get_output_dir, get_upload_dir, read_metadata_from_project, save_metadata};

pub async fn fetch_projects() -> Result<Vec<Project>, std::io::Error> {
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

async fn process_project_dir(path: &Path) -> Option<Project> {
    let project_id = path.file_name()?.to_str()?.to_string();
    let thumbnail_path = find_thumbnail(path).await?;
    if !fs::try_exists(&thumbnail_path).await.unwrap_or(false) {
        return None;
    }
    let metadata = read_metadata_from_project(&project_id).await.ok()?;

    let serving_url = convert_image_path_to_serving_url(&thumbnail_path).await;

    Some(Project {
        id: project_id,
        project_name: metadata.project_name,
        thumbnail_path: serving_url,
    })
}

async fn find_thumbnail(movie_dir_output_path: &Path) -> Option<PathBuf> {
    if let Some(path) = find_long_exposure_image(movie_dir_output_path).await {
        Some(path)
    } else {
        Some(movie_dir_output_path.join("frames/ffout_thumbnail_0001.webp"))
    }
}

async fn find_long_exposure_image(path: &Path) -> Option<PathBuf> {
    let mut sub_dir_stream = fs::read_dir(path).await.ok()?;
    while let Some(sub_entry) = sub_dir_stream.next_entry().await.ok()? {
        let sub_path = sub_entry.path();
        if sub_path.is_file() {
            if let Some(file_name) = sub_path.file_name().and_then(|name| name.to_str()) {
                if file_name.starts_with("long_exposure_image_") && file_name.ends_with(".png") {
                    return Some(sub_path);
                }
            }
        }
    }
    None
}

pub async fn delete_project_by_id(project_id: &str) -> Result<(), std::io::Error> {
    let output_dir = get_output_dir();
    let upload_dir = get_upload_dir();
    let project_dir_path = output_dir.join(project_id);

    // Remove the uploaded video file
    let mut uploaded_file_found = false;
    let mut dir_entries = fs::read_dir(&upload_dir).await?;

    while let Some(entry) = dir_entries.next_entry().await? {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // Check if the file starts with the project_id
        if file_name_str.starts_with(project_id) {
            let uploaded_file_path = upload_dir.join(&file_name);

            // Remove the uploaded video file
            fs::remove_file(&uploaded_file_path).await?;
            uploaded_file_found = true;
            break;
        }
    }

    // Delete the project directory
    if project_dir_path.exists() && project_dir_path.is_dir() {
        fs::remove_dir_all(project_dir_path).await?;
    } else if !uploaded_file_found {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Project directory or uploaded video file not found",
        ));
    }

    Ok(())
}


pub async fn process_upload(
    video_id: Option<Uuid>,
    video_data: Option<BytesMut>,
    video_extension: Option<String>,
    project_name: String,
    scale: String,
    fps: usize,
) -> Result<UploadVideoResponse, Box<dyn std::error::Error>> {
    let upload_dir = get_upload_dir();
    let output_dir = get_output_dir();
    let mut metadata : Option<ProjectMetadata> = None;

    let video_id = if let Some(id) = video_id {
        // Read metadata asynchronously
        metadata = Some(read_metadata_from_project(&id.to_string()).await?);
        id
    } else {
        Uuid::new_v4()
    };


    debug!("Upload dir is {:?}", &upload_dir);

    let video_file_extension = match video_extension {
        Some(ext) => ext,
        None => {
            // If video_extension is not provided, read it from existing metadata
            if let Some(ref metadata) = metadata {
                metadata.video_file_extension.clone()
            } else {
                return Err("Video extension not provided and no metadata available".into());
            }
        }
    };
    // Check if the new fps and scale match the ones in metadata
    if let Some(ref existing_metadata) = metadata {
        if existing_metadata.fps == fps && existing_metadata.scale == scale {
            // The fps and scale match, we can skip processing
            info!("FPS and scale match existing metadata, skipping processing");
            return Ok(UploadVideoResponse {
                message: "Processing skipped as FPS and scale match existing project",
                video_id: video_id.to_string(),
            });
        }
    }

    let uploaded_movie_save_file_path = upload_dir.join(format!(
        "{}.{}",
        video_id,
        video_file_extension
    ));

    if let Some(uploaded_video_data) = video_data {
        let mut video_file = fs::File::create(&uploaded_movie_save_file_path).await?;
        video_file.write_all(&uploaded_video_data).await?;
        debug!("Uploaded movie path {:?}", &uploaded_movie_save_file_path);

    }

    // Create directory for cut images
    let cut_images_save_dir_path =
        output_dir.join(format!("{}/frames/", video_id.to_string()));

    // If a folder for the cut images already exists, delete it
    if fs::try_exists(&cut_images_save_dir_path).await? {
        debug!("Frames directory for project {} already exists, deleting its frames!", video_id);
        fs::remove_dir_all(&cut_images_save_dir_path).await?;
    }
    fs::create_dir_all(&cut_images_save_dir_path).await?;

    // Perform FFmpeg processing
    let ffmpeg_output_path = cut_images_save_dir_path
        .join("ffout_%4d.png")
        .to_str()
        .ok_or("Invalid ffmpeg output path")?
        .to_string();
    let webp_output_path = cut_images_save_dir_path
        .join("ffout_thumbnail_%4d.webp")
        .to_str()
        .ok_or("Invalid webp output path")?
        .to_string();

    info!("{}", format!("Video file extension: {}", video_file_extension.to_string()));
    info!("{}", format!("Upload save path: {}", uploaded_movie_save_file_path.to_str().unwrap()));
    info!("{}", format!("FPS {}", fps.to_string()));
    info!("{}", format!("Scale {}", scale));


    Command::new("ffmpeg")
        .args([
            "-i",
            &uploaded_movie_save_file_path.to_str().unwrap(), // Input file path
            "-threads",
            "0", // Use optimal amount of threads
            "-vf",
            &format!("scale={}", scale), // Scaling option
            "-r",
            &fps.to_string(), // Frames per second
            &ffmpeg_output_path,

            // Output for WebP thumbnails
            "-vf",
            "scale=720:-1", // Scaling for thumbnail WebP images
            "-r",
            &fps.to_string(), // FPS for WebP
            "-c:v",
            "libwebp", // Codec for WebP
            "-lossless",
            "0", // 0 for lossy, 1 for lossless
            "-compression_level",
            "6", // Compression level (0 to 6 highest)
            "-q:v",
            "25", // Quality level (0 worst to 100 best)
            "-preset",
            "default", // Encoding preset
            "-an",     // No audio
            &webp_output_path, // Output path for WebP
        ])
        .output()
        .await
        .expect("Failed to execute ffmpeg");

    // Save metadata to a file
    let metadata = ProjectMetadata {
        project_name,
        fps,
        scale,
        video_file_extension,
        latest_long_exposure_image_name: None,
    };

    save_metadata(&metadata, &video_id.to_string())?;

    Ok(UploadVideoResponse {
        message: "Video was uploaded successfully",
        video_id: video_id.to_string(),
    })
}