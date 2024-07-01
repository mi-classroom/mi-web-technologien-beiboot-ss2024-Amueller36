use std::ffi::OsStr;
use std::io::Error;
use std::path::Path;
use std::process::Command;

use actix_multipart::Multipart;
use actix_web::{HttpResponse, post};
use actix_web::web::{block, BytesMut};
use futures_util::TryStreamExt;
use serde::Serialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::io::ErrorKind;
use tracing::{debug, error, info, trace};
use uuid::Uuid;

use crate::core::long_exposure_image_logic::create_long_exposure_image;
use crate::utils::{
    create_directory_if_not_created_yet, get_output_dir, get_upload_dir, read_text_from_field,
};

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

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap_or_default();

        match name {
            "video_file" => {
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
    if let Some(data) = video_data {
        let movie_file_id = Uuid::new_v4();
        debug!("Upload dir is {:?}", &upload_dir);
        let uploaded_movie_save_file_path = upload_dir.join(format!(
            "{}.{}",
            movie_file_id,
            video_extension.unwrap_or("mp4".to_string())
        ));
        debug!("Uploaded movie path {:?}", &uploaded_movie_save_file_path);

        block(move || async move {
            // Save the uploaded video file
            let mut saved_video_file = fs::File::create(&uploaded_movie_save_file_path)
                .await
                .unwrap();
            saved_video_file.write_all(&data).await.unwrap();

            // Create directory for cut images
            let cut_images_save_dir_path =
                output_dir.join(format!("{}/frames/", movie_file_id.to_string()));
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
            Command::new("ffmpeg")
                .args([
                    "-i",
                    uploaded_movie_save_file_path.to_str().unwrap(),  // Input file path
                    "-threads", "0",                                  // Use Optimal amount of threads
                    "-vf", &format!("scale={}", scale),               // Scaling option
                    "-r",  &fps.to_string(),                          // Frames per second
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

            HttpResponse::Ok().json(UploadVideoResponse {
                message: "Video was uploaded successfully",
                video_id: movie_file_id.to_string(),
            })
        })
        .await
        .unwrap()
        .await
    } else {
        HttpResponse::BadRequest().body("No video data found.")
    }
}
