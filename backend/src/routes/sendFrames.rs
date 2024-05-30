use actix_web;
use actix_web::dev::Payload;
use actix_web::web::block;
use actix_web::{post, web, FromRequest, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::core::long_exposure_image_logic::create_long_exposure_image;
use crate::utils::get_output_dir;

#[derive(Deserialize)]
struct CreateLongExposureImageRequest {
    video_id: String,
    frames_to_include: Vec<usize>,
}

#[post("/sendFrames")]
pub async fn create_long_exposure_image_request(
    selected_frames: web::Json<CreateLongExposureImageRequest>,
) -> HttpResponse {
    let img_result = web::block(|| {
        let output_dir = get_output_dir();
        let selected_frames = selected_frames.into_inner();

        let frames_to_include = selected_frames.frames_to_include.clone();
        let video_id = selected_frames.video_id.clone();
        let path_to_cut_images = output_dir.join(format!("{}/frames/", video_id));

        debug!("Frames to Include sind : {:?}", frames_to_include);
        create_long_exposure_image(path_to_cut_images, frames_to_include)
    })
    .await
    .unwrap();

    match img_result.await {
        Ok(path_to_long_exposure_img) => HttpResponse::Ok().body(path_to_long_exposure_img),
        Err(_) => {
            error!("There was an error while trying to create the long exposure image.");
            HttpResponse::InternalServerError().body(
                "An internal server error occurred while trying to create the long exposure image.",
            )
        }
    }
}
