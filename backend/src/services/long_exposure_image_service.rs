use tracing::log::trace;

use crate::core::long_exposure_image_logic::create_long_exposure_image;
use crate::error::ServiceError;
use crate::models::FrameData;
use crate::services::long_exposure_image_service::ServiceError::CreateImageError;
use crate::utils::{get_output_dir, read_metadata_from_project, save_project_metadata};

pub async fn create_long_exposure_image_svc(
    project_id: String,
    frames_to_include: Vec<FrameData>,
) -> Result<String, ServiceError> {
    let output_dir = get_output_dir();
    let path_to_cut_images = output_dir.join(format!("{}/frames/", project_id));

    trace!("Frames to include are: {:?}", frames_to_include);

    let path_to_long_exposure_img = create_long_exposure_image(path_to_cut_images, frames_to_include)
        .await
        .map_err(CreateImageError)?;

    let mut metadata = read_metadata_from_project(&project_id).await?;

    metadata.latest_long_exposure_image_name = Some(path_to_long_exposure_img.clone());

    // Save metadata asynchronously
    save_project_metadata(&metadata, &project_id)?;

    Ok(path_to_long_exposure_img)
}


