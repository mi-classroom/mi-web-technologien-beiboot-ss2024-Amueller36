use std::fs;
use std::path::Path;

use image::{ImageBuffer, Rgba, RgbaImage};

pub(crate) fn create_long_exposure_image(dir_path: &Path) -> image::ImageResult<()> {
    let mut image_paths: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect();

    // Sortieren Sie die Pfade, falls diese nicht in der richtigen Reihenfolge sind.
    image_paths.sort();

    let mut image_buffers = Vec::new();

    for path in image_paths {
        let img = image::open(path)?.to_rgba8();
        image_buffers.push(img);
    }

    // Nehmen wir an, alle Bilder haben die gleiche Größe
    let (width, height) = image_buffers[0].dimensions();
    let mut long_exposure_img: RgbaImage = ImageBuffer::new(width, height);

    for (x, y, pixel) in long_exposure_img.enumerate_pixels_mut() {
        let mut max_pixel = Rgba([0, 0, 0, 255]);

        for img in &image_buffers {
            let current_pixel = img.get_pixel(x, y);
            max_pixel = blend_max(&max_pixel, &current_pixel);
        }

        *pixel = max_pixel;
    }

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

