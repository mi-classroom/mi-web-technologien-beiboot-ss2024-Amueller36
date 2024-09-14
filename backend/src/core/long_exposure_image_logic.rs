use std::path::PathBuf;

use image::{ImageBuffer, Rgba, RgbaImage};
use rayon::prelude::*;

use crate::models::FrameData;
use crate::utils;
use crate::utils::convert_image_path_to_serving_url;

/**

Creates a long-exposure image by blending multiple frames with their associated weights.
The blending takes into account pixel brightness and alpha values to adjust each frame's contribution.
# Arguments
- `frames_dir_path`: The directory where the frames are located.
- `frames_data`: A vector of `FrameData` which contains information such as frame number and weight.
# Returns
- A `Result` containing the image source as usable url of the generated image on success, or an error message on failure.

 */
pub async fn create_long_exposure_image(
    frames_dir_path: PathBuf,
    frames_data: Vec<FrameData>,
) -> Result<String, String> {
    #[cfg(debug)]
    let start_time = Utc::now();

    // Collect images and their user-specified weights
    let image_buffers: Vec<(RgbaImage, f32)> = frames_data
        .par_iter()
        .map(|frame| {
            let frame_file_name = format!("ffout_{:04}.png", frame.frame_number);
            let frame_path = frames_dir_path.join(frame_file_name);
            let img = image::open(&frame_path)
                .map_err(|e| format!("Failed to open image: {}", e))?
                .to_rgba8();
            Ok::<(RgbaImage, f32), String>((img, frame.frame_weight))
        })
        .collect::<Result<Vec<_>, String>>()?;

    #[cfg(debug)]
    let file_processing_end_time = Utc::now();

    if image_buffers.is_empty() {
        return Err("No images were chosen".to_string());
    }

    let (width, height) = image_buffers[0].0.dimensions();
    let mut long_exposure_img: RgbaImage = ImageBuffer::new(width, height);

    // Normalize frame weights
    let total_frame_weight: f32 = image_buffers.par_iter().map(|(_, weight)| *weight).sum();
    if total_frame_weight == 0.0 {
        return Err("Total frame weight cannot be zero".to_string());
    }

    // Normalize weights so that the total weight sums to 1
    let image_buffers: Vec<_> = image_buffers
        .into_par_iter()
        .map(|(img, mut weight)| {
            weight /= total_frame_weight;
            (img, weight)
        })
        .collect();

    // Create buffer to hold the final pixel data
    let final_pixels: Vec<(u8, u8, u8, u8)> = (0..(width * height) as usize)
        .into_par_iter()
        .map(|index| {
            let mut r_accum = 0.0;
            let mut g_accum = 0.0;
            let mut b_accum = 0.0;
            let mut a_accum = 0.0;
            let mut weight_accum = 0.0;

            // For each pixel in all frames, accumulate weighted colors
            for (img, frame_weight) in &image_buffers {
                let x = (index as u32) % width;
                let y = (index as u32) / width;
                let pixel = img.get_pixel(x, y);
                let Rgba(new_data) = *pixel;

                // Normalize alpha to [0,1]
                let alpha = new_data[3] as f32 / 255.0;

                // Calculate pixel brightness
                //https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color
                let brightness = 0.299 * new_data[0] as f32
                    + 0.587 * new_data[1] as f32
                    + 0.114 * new_data[2] as f32;
                let brightness_norm = brightness / 255.0; // Normalize to [0,1]

                // Adjust pixel weight using brightness
                let brightness_weight = brightness_norm.powf(4.5); // You can tweak this value
                let pixel_weight = frame_weight * alpha * brightness_weight;

                // Accumulate weighted color values
                r_accum += new_data[0] as f32 * pixel_weight;
                g_accum += new_data[1] as f32 * pixel_weight;
                b_accum += new_data[2] as f32 * pixel_weight;
                a_accum += alpha * pixel_weight; // Accumulate alpha

                // Accumulate weight
                weight_accum += pixel_weight;
            }

            if weight_accum > 0.0 {
                let r = (r_accum / weight_accum).min(255.0);
                let g = (g_accum / weight_accum).min(255.0);
                let b = (b_accum / weight_accum).min(255.0);
                let a = ((a_accum / weight_accum) * 255.0).min(255.0);
                (r as u8, g as u8, b as u8, a as u8)
            } else {
                (0, 0, 0, 0)
            }
        })
        .collect();

    // Sequentially write the accumulated pixels into the final image
    for (index, (r, g, b, a)) in final_pixels.into_iter().enumerate() {
        let x = (index as u32) % width;
        let y = (index as u32) / width;
        long_exposure_img.put_pixel(x, y, Rgba([r, g, b, a]));
    }

    #[cfg(debug)]
    {
        let image_calculation_time = Utc::now();
        debug!(
            "File Reading = {}",
            (file_processing_end_time - start_time).num_milliseconds()
        );
        debug!(
            "Long Exposure Image Calculations= {}",
            (image_calculation_time - file_processing_end_time).num_milliseconds()
        );
        debug!(
            "Total Time = {}",
            (image_calculation_time - start_time).num_milliseconds()
        );
    }

    let long_exposure_image_file_path = utils::generate_timestamped_path(
        &frames_dir_path.join(".."),
        "long_exposure_image",
        "png",
    );
    long_exposure_img
        .save(&long_exposure_image_file_path)
        .map_err(|e| e.to_string())?;

    Ok(
        convert_image_path_to_serving_url(&long_exposure_image_file_path).await,
    )
}