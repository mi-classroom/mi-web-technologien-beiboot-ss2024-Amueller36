use actix_cors::Cors;
use actix_files as fs;
use actix_web::{App, HttpServer};

use crate::utils::{create_directory_if_not_created_yet, get_output_dir, get_upload_dir};

mod core;
mod controller;
mod utils;
mod services;
mod error;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let upload_dir = get_upload_dir();
    let output_dir = get_output_dir();
    create_directory_if_not_created_yet(upload_dir.to_str().unwrap()).await;
    create_directory_if_not_created_yet(output_dir.to_str().unwrap()).await;

    // Logging
    tracing_subscriber::fmt::init();

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .max_age(3600),
            )
            .service(fs::Files::new("/outputs", output_dir.to_str().unwrap()).show_files_listing())
            .service(fs::Files::new("/uploads", upload_dir.to_str().unwrap()).show_files_listing())
            .service(controller::projects::create_or_update_project)
            .service(controller::projects::create_long_exposure_image)
            .service(controller::projects::get_projects)
            .service(controller::projects::get_project_metadata)
            .service(controller::projects::delete_project)
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
