use std::path::Path;
use std::process::Command;

use actix_multipart::Multipart;
use actix_web::{HttpResponse, post};
use actix_web::web::{block, BytesMut};
use futures_util::TryStreamExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::core::image_magic::create_long_exposure_image;

#[derive(serde::Deserialize)]
pub struct MovieProcessingArgs {
    pub scale: String,
    pub fps: usize,
}

#[post("/upload")]
async fn upload_file(mut payload: Multipart) -> HttpResponse {
    let max_file_count: usize = 3;
    let upload_dir = "./upload/";
    let output_dir = "./output/";
    let mut current_count: usize = 0;

    let mut scale = String::from("1600:-1");
    let mut fps = 1;
    let mut video_data: Option<(String, BytesMut)> = None;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let name = content_disposition.get_name().unwrap_or_default();

        match name {
            "video_file" => {
                let file_name = content_disposition.get_filename().unwrap_or("tmpfile");
                video_data = Some((file_name.to_string(), BytesMut::new()));
                while let Some(chunk) = field.try_next().await.unwrap() {
                    video_data.as_mut().unwrap().1.extend_from_slice(&chunk);
                }
            },
            "scale" => {
                scale = read_text_from_field(field).await;
            },
            "fps" => {
                fps = read_text_from_field(field).await.parse::<usize>().unwrap_or(1);
            },
            _ => eprintln!("Unexpected field: {}", name),
        }
    }

    // Now process the video data if it exists
    if let Some((file_name, data)) = video_data {
        if current_count >= max_file_count {
            return HttpResponse::BadRequest().body("Maximum file count exceeded");
        }

        let movie_file_id = Uuid::new_v4();
        let destination = format!("{}/{}-{}", upload_dir, movie_file_id, file_name);
        let mut saved_file = fs::File::create(&destination).await.unwrap();
        saved_file.write_all(&data).await.unwrap();
        current_count += 1;

        // Perform FFmpeg processing
        fs::create_dir(format!("{}/{}", output_dir, movie_file_id)).await.unwrap();

        let ffmpeg_output = format!("{}/{}/ffout_%3d.png", output_dir, movie_file_id);
        block(move || {
            Command::new("ffmpeg")
                .args([
                    "-i", &destination,
                    "-vf", &format!("scale={}", scale),
                    "-r", &fps.to_string(),
                    &ffmpeg_output,
                ])
                .output()
                .expect("failed to execute ffmpeg");
        }).await.unwrap();

        // Perform long exposure image creation
        let path_to_extracted_images = format!("{}/{}/", output_dir, movie_file_id);
        if let Err(e) = create_long_exposure_image(Path::new(&path_to_extracted_images)) {
            eprintln!("Failed to create long exposure image: {}", e);
        }
    }

    HttpResponse::Ok().body("Files uploaded successfully and long exposure image was created.")
}

async fn read_text_from_field(mut field: actix_multipart::Field) -> String {
    let mut data = BytesMut::new();
    while let Some(chunk) = field.try_next().await.unwrap() {
        data.extend_from_slice(&chunk);
    }
    String::from_utf8(data.to_vec()).unwrap()
}