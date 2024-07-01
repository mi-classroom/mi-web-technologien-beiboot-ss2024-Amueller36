use actix_web;
use actix_web::{FromRequest, HttpRequest, HttpResponse, post, Responder, web};
use actix_web::dev::Payload;
use actix_web::web::block;
use serde::Deserialize;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::core::long_exposure_image_logic::create_long_exposure_image;
use crate::utils::get_output_dir;

#[derive(Deserialize, Debug, PartialEq)]
pub struct FrameData {
    pub frame_number: usize,
    pub frame_weight: f32,
}
#[derive(Deserialize)]
struct CreateLongExposureImageRequest {
    video_id: String,
    frames_to_include: Vec<FrameData>,
}

#[post("/sendFrames")]
pub async fn create_long_exposure_image_request(
    create_long_exposure_image_request: web::Json<CreateLongExposureImageRequest>,
) -> HttpResponse {
    let image_request = create_long_exposure_image_request.into_inner();
    
    let img_result = block(move || {
        let output_dir = get_output_dir();
        let path_to_cut_images = output_dir.join(format!("{}/frames/", image_request.video_id));

        debug!("Frames to include are: {:?}", image_request.frames_to_include);
        create_long_exposure_image(path_to_cut_images, image_request.frames_to_include)
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

