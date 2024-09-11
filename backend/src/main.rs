use actix_cors::Cors;
use actix_files as fs;
use actix_web::{App, HttpResponse, HttpServer};

use crate::utils::{create_directory_if_not_created_yet, get_output_dir, get_upload_dir};

mod core;
mod routes;
mod utils;

#[actix_web::get("/")]
async fn root() -> HttpResponse {
    HttpResponse::Ok().body("Hello World")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let upload_dir = get_upload_dir();
    let output_dir = get_output_dir();
    create_directory_if_not_created_yet(upload_dir.to_str().unwrap()).await;
    create_directory_if_not_created_yet(output_dir.to_str().unwrap()).await;

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
            .service(root)
            .service(fs::Files::new("/outputs", output_dir.to_str().unwrap()).show_files_listing())
            .service(fs::Files::new("/uploads", upload_dir.to_str().unwrap()).show_files_listing())
            .service(routes::upload::upload_file)
            .service(routes::create_long_exposure_image::create_long_exposure_image_request)
            .service(routes::projects::get_projects)
            .service(routes::projects::get_project_metadata)
            .service(routes::projects::delete_project)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
