use std::fs;
use std::path::Path;

use image::{ImageBuffer, Rgba, RgbaImage};
use rayon::prelude::*;

pub(crate) async fn create_long_exposure_image(dir_path: &Path) -> image::ImageResult<()> {

    let time_measurement = chrono::Utc::now();

    let mut image_paths_iter = tokio::fs::read_dir(dir_path).await.expect("Should have read");
    let mut image_paths : Vec<_> = vec![];
    while let Ok(Some(entry)) = image_paths_iter.next_entry().await{
        image_paths.push(entry.path())
    }


    // Sortieren Sie die Pfade, falls diese nicht in der richtigen Reihenfolge sind.
    //image_paths.sort();

    let mut image_buffers : Vec<RgbaImage> =
        image_paths.par_iter()
            .filter_map(|path|{
                image::open(path).ok()
            })
            .map(|img|img.to_rgba8())
            .collect();

    let end_time = chrono::Utc::now();

    // Nehmen wir an, alle Bilder haben die gleiche Größe
    let (width, height) = image_buffers[0].dimensions();
    let mut long_exposure_img: RgbaImage = ImageBuffer::new(width, height);

    long_exposure_img.enumerate_pixels_mut()
        .par_bridge()
        .for_each(|(x,y,pixel)|{
            let r: Rgba<u8> = image_buffers.iter()
                .fold(Rgba([0, 0, 0, 255]), |max_pixel, img| {
                    let current_pixel = img.get_pixel(x, y);
                    blend_max(&max_pixel, current_pixel)
                });
            pixel.0 = r.0
        });
    let processing_time = chrono::Utc::now();

    println!("File Reading = {}",(end_time-time_measurement).num_milliseconds());
    println!("File Processing = {}", (processing_time-end_time).num_milliseconds());
    println!("Total Time = {}", (processing_time-time_measurement).num_milliseconds());


    long_exposure_img.save(dir_path.join("long_exposure_image.png"))?;
    Ok(())
}
fn blend_max(pixel1: &Rgba<u8>, pixel2: &Rgba<u8>) -> Rgba<u8> {
    Rgba([
        pixel1[0].max(pixel2[0]),
        pixel1[1].max(pixel2[1]),
        pixel1[2].max(pixel2[2]),
        255, // Alpha Kanal beibehalten
    ])
}

