use std::path::PathBuf;

use image::{ImageBuffer, Rgba, RgbaImage};
use rayon::prelude::*;

use crate::models::FrameData;
use crate::utils;
use crate::utils::convert_image_path_to_serving_url;

pub async fn create_long_exposure_image(
    frames_dir_path: PathBuf,
    frames_data: Vec<FrameData>,
) -> Result<String, String> {

    #[cfg(debug)]
    let start_time = Utc::now();

    let mut image_buffers: Vec<(RgbaImage, f32)> = vec![];
    for frame in frames_data.iter() {
        let frame_file_name = format!("ffout_{:04}.png", frame.frame_number);
        let frame_path = frames_dir_path.join(frame_file_name);
        let img = image::open(&frame_path).expect("Should have opened image").to_rgba8();
        image_buffers.push((img, frame.frame_weight));
    }

    #[cfg(debug)]
    let file_processing_end_time = Utc::now();

    if image_buffers.is_empty() {
        return Err("No images were chosen".to_string());
    }

    let total_weight: f32 = image_buffers.iter().map(|(_, weight)| weight).sum();
    let (width, height) = image_buffers[0].0.dimensions();
    let mut long_exposure_img: RgbaImage = ImageBuffer::new(width, height);

    // Create a buffer to hold the blended pixels
    let blended_pixels: Vec<Vec<(f32, f32, f32, f32)>> = image_buffers.par_iter().map(|(img, weight)| {
        let mut temp_blended = vec![(0.0, 0.0, 0.0, 0.0); (width * height) as usize];
        for (x, y, pixel) in img.enumerate_pixels() {
            let index = (y * width + x) as usize;
            let Rgba(new_data) = *pixel;
            temp_blended[index].0 += new_data[0] as f32 * weight;
            temp_blended[index].1 += new_data[1] as f32 * weight;
            temp_blended[index].2 += new_data[2] as f32 * weight;
            temp_blended[index].3 += new_data[3] as f32 * weight;
        }
        temp_blended
    }).collect();

    // Combine the intermediate results into the final blended image
    let mut final_blended = vec![(0.0, 0.0, 0.0, 0.0); (width * height) as usize];
    for temp_blended in blended_pixels {
        for (index, pixel) in temp_blended.into_iter().enumerate() {
            final_blended[index].0 += pixel.0;
            final_blended[index].1 += pixel.1;
            final_blended[index].2 += pixel.2;
            final_blended[index].3 += pixel.3;
        }
    }

    // Normalize the pixel values and apply them back to the original image
    for (index, pixel) in final_blended.into_iter().enumerate() {
        let x = (index as u32) % width;
        let y = (index as u32) / width;
        long_exposure_img.put_pixel(x, y, Rgba([
            (pixel.0 / total_weight).min(255.0) as u8,
            (pixel.1 / total_weight).min(255.0) as u8,
            (pixel.2 / total_weight).min(255.0) as u8,
            (pixel.3 / total_weight).min(255.0) as u8,
        ]));
    }

    #[cfg(debug)]
    {
        let image_calculation_time = chrono::Utc::now();
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

    let long_exposure_image_file_path =
        utils::generate_timestamped_path(&frames_dir_path.join(".."), "long_exposure_image", "png");
    long_exposure_img
        .save(&long_exposure_image_file_path)
        .map_err(|e| e.to_string())?;
    Ok(convert_image_path_to_serving_url(
        &long_exposure_image_file_path,
    ).await)
}

fn blend_max(pixel1: &Rgba<u8>, pixel2: &Rgba<u8>) -> Rgba<u8> {
    Rgba([
        pixel1[0].max(pixel2[0]),
        pixel1[1].max(pixel2[1]),
        pixel1[2].max(pixel2[2]),
        255, // Alpha Kanal beibehalten
    ])
}

fn blend_average(pixel1: &Rgba<u8>, pixel2: &Rgba<u8>) -> Rgba<u8> {
    Rgba([
        (pixel1[0] as u16 + pixel2[0] as u16 / 2) as u8,
        (pixel1[1] as u16 + pixel2[1] as u16 / 2) as u8,
        (pixel1[2] as u16 + pixel2[2] as u16 / 2) as u8,
        255, // Alpha Kanal beibehalten
    ])
}
