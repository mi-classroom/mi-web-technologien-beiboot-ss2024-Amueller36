use actix_web::{App, HttpResponse, HttpServer};

use crate::routes::upload::upload_file;

pub mod routes;
pub mod core;

#[actix_web::get("/")]
async fn root() -> HttpResponse {
    HttpResponse::Ok().body("Hello world")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(root)
            .service(upload_file)
    })
        .bind(("0.0.0.0", 8080))?.run().await
}
