use std::ffi::OsStr;
use std::io::Error;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use actix_multipart::Multipart;
use actix_web::{HttpResponse, post};
use actix_web::web::{block, BytesMut};
use futures_util::TryStreamExt;
use serde::Serialize;
use serde_json::json;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::io::ErrorKind;
use tracing::{debug, error, info, trace};
use uuid::Uuid;

use crate::core::long_exposure_image_logic::create_long_exposure_image;
use crate::utils::{create_directory_if_not_created_yet, get_output_dir, get_upload_dir, read_text_from_field, save_metadata};
use crate::routes::projects::ProjectMetadata;

#[derive(Serialize)]
struct UploadVideoResponse {
    message: &'static str,
    video_id: String,
}

#[post("/upload")]
async fn upload_file(mut payload: Multipart) -> HttpResponse {
    let upload_dir = get_upload_dir();
    let output_dir = get_output_dir();

    let mut scale = String::new();
    let mut fps = 0;
    let mut video_data: Option<BytesMut> = None;
    let mut video_extension: Option<String> = None;
    let mut video_id: Option<Uuid> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap_or_default();

        match name {
            "video_id" => {
                let id = read_text_from_field(field).await;
                info!("{}", format!("Video id is {id}"));
                video_id = Some(Uuid::from_str(&id).unwrap());
            }
            "video_file" => {
                // When the same video was already uploaded, then skip the upload!
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
                video_data = Some(BytesMut::new());
                while let Some(chunk) = field.try_next().await.unwrap() {
                    video_data.as_mut().unwrap().extend_from_slice(&chunk);
                }
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

    // Now process the video data if it exists
    if video_id.is_none() {
        video_id = Some(Uuid::new_v4());
    }
    let video_id = video_id.unwrap();
    debug!("Upload dir is {:?}", &upload_dir);
    let uploaded_movie_save_file_path = upload_dir.join(format!(
        "{}.{}",
        video_id,
        video_extension.unwrap_or("mp4".to_string())
    ));
    debug!("Uploaded movie path {:?}", &uploaded_movie_save_file_path);

    block(move || async move {
        // Save the uploaded video file
        if let Some(uploaded_video_data) = video_data {
            let mut video_file = fs::File::create(&uploaded_movie_save_file_path)
                .await
                .unwrap();
            video_file.write_all(&uploaded_video_data).await.unwrap();
        }

        // Create directory for cut images
        let cut_images_save_dir_path =
            output_dir.join(format!("{}/frames/", video_id.to_string()));

        // if a folder for the cut images already exists, delete it
        if fs::try_exists(&cut_images_save_dir_path).await.expect("Couldn't check if frames directory already exists.") {
            info!("Frames directory for project {video_id} already exists, deleting it's frames!.");
            fs::remove_dir_all(&cut_images_save_dir_path).await.expect("Couldn't delete existing frames directory.");
        }
        fs::create_dir_all(&cut_images_save_dir_path).await.unwrap();

        // Perform FFmpeg processing
        let ffmpeg_output_path = cut_images_save_dir_path
            .join("ffout_%4d.png")
            .to_str()
            .unwrap()
            .to_string();
        let webp_output_path = cut_images_save_dir_path
            .join("ffout_thumbnail_%4d.webp")
            .to_str()
            .unwrap()
            .to_string();

        let video_file_extension = uploaded_movie_save_file_path.extension().unwrap().to_string_lossy().to_string();
        Command::new("ffmpeg")
            .args([
                "-i",
                &uploaded_movie_save_file_path.to_str().unwrap(),  // Input file path
                "-threads", "0",                                  // Use Optimal amount of threads
                "-vf", &format!("scale={}", scale),               // Scaling option
                "-r", &fps.to_string(),                          // Frames per second
                &ffmpeg_output_path,

                // Output for WebP thumbnails
                "-vf", "scale=720:-1",                            // Scaling for Thumbnail WebP Images
                "-r", &fps.to_string(),                           // FPS for WebP
                "-c:v", "libwebp",                                // Codec for WebP
                "-lossless", "0",                                 // 0 for lossy, 1 for lossless
                "-compression_level", "6",                        // Compression level (0 to 6(highest))
                "-q:v", "25",                                     // Quality level (0(worst quality) to 100(best quality))
                "-preset", "default",                             // Encoding preset (default balances compression quality and speed of the encoding)
                "-an",                                            // No audio
                &webp_output_path,                                // Output path for WebP
            ])
            .output()
            .expect("failed to execute ffmpeg");

        // Save metadata to a file
        let metadata = ProjectMetadata {
            fps,
            scale,
            video_file_extension,
            latest_long_exposure_image_name: None,
        };

        let metadata_file_path = output_dir.join(format!("{video_id}/metadata.json"));

        save_metadata(&metadata, &metadata_file_path).expect("An Error occured, while trying to save metadata!");

        HttpResponse::Ok().json(UploadVideoResponse {
            message: "Video was uploaded successfully",
            video_id: video_id.to_string(),
        })
    })
        .await
        .unwrap()
        .await
}
