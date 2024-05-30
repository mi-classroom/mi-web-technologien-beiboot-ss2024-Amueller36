use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use chrono::Utc;
use image::error::{ParameterError, ParameterErrorKind};
use image::{ImageBuffer, ImageError, ImageResult, Rgba, RgbaImage};
use rayon::prelude::*;
use tracing::{debug, error, info};

use crate::utils::convert_image_path_to_serving_url;

fn generate_timestamped_path(base_path: &PathBuf, base_name: &str, extension: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    base_path.join(format!("{}_{}.{}", base_name, timestamp, extension))
}

pub async fn create_long_exposure_image(
    frames_dir_path: PathBuf,
    selected_images: Vec<usize>,
) -> Result<String, String> {
    #[cfg(dbg)]
    let time_measurement = chrono::Utc::now();

    info!("Frames dir path is {:?}", frames_dir_path);
    let mut image_paths_iter = tokio::fs::read_dir(&frames_dir_path)
        .await
        .expect("Should have read");
    let mut image_paths: Vec<_> = vec![];
    while let Ok(Some(entry)) = image_paths_iter.next_entry().await {
        let frame_file_name = entry.file_name().to_string_lossy().to_string();

        let frame_file_name_without_extension = Path::new(&frame_file_name)
            .file_stem()
            .and_then(OsStr::to_str)
            .expect("Conversion from frame_file_name to str failed")
            .to_string();

        debug!("File name of frame is {frame_file_name_without_extension}");
        let frame_number_as_string: String = frame_file_name_without_extension
            .split("ffout_")
            .last()
            .expect("Couldn't find String 'ffout' in filename of frame")
            .replace(r"0+", "")
            .to_string();
        debug!("Frame Number as String is : {}", frame_number_as_string);
        let frame_number: usize = frame_number_as_string
            .parse()
            .expect("Some error occured parsing Frame Number From String to usize");
        if selected_images.contains(&frame_number) {
            debug!("Frame {} was included", frame_number);
            image_paths.push(entry.path())
        }
    }

    // Sortieren Sie die Pfade, falls diese nicht in der richtigen Reihenfolge sind.
    //image_paths.sort();

    let image_buffers: Vec<RgbaImage> = image_paths
        .par_iter()
        .filter_map(|path| image::open(path).ok())
        .map(|img| img.to_rgba8())
        .collect();
    #[cfg(dbg)]
    let end_time = chrono::Utc::now();

    if image_buffers.is_empty() {
        error!("Could not create long exposure image because no images/frames were chosen");
        return Err("No images were chosen".to_string());
    }
    // Nehmen wir an, alle Bilder haben die gleiche Größe
    let (width, height) = image_buffers[0].dimensions();
    let mut long_exposure_img: RgbaImage = ImageBuffer::new(width, height);

    long_exposure_img
        .enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x, y, pixel)| {
            let r: Rgba<u8> = image_buffers
                .iter()
                .fold(Rgba([0, 0, 0, 255]), |max_pixel, img| {
                    let current_pixel = img.get_pixel(x, y);
                    blend_max(&max_pixel, current_pixel)
                });
            pixel.0 = r.0
        });

    #[cfg(dbg)]
    {
        let processing_time = chrono::Utc::now();
        println!(
            "File Reading = {}",
            (end_time - time_measurement).num_milliseconds()
        );
        println!(
            "File Processing = {}",
            (processing_time - end_time).num_milliseconds()
        );
        println!(
            "Total Time = {}",
            (processing_time - time_measurement).num_milliseconds()
        );
    }
    let long_exposure_image_file_path =
        generate_timestamped_path(&frames_dir_path.join(".."), "long_exposure_image", "png");
    long_exposure_img
        .save(&long_exposure_image_file_path)
        .map_err(|e| e.to_string())?;
    Ok(convert_image_path_to_serving_url(
        &long_exposure_image_file_path,
    ))
}

fn blend_max(pixel1: &Rgba<u8>, pixel2: &Rgba<u8>) -> Rgba<u8> {
    Rgba([
        pixel1[0].max(pixel2[0]),
        pixel1[1].max(pixel2[1]),
        pixel1[2].max(pixel2[2]),
        255, // Alpha Kanal beibehalten
    ])
}
