use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[get("/health")]
async fn health() -> impl Responder {
    "I'm up and running!"
}

#[derive(Deserialize)]
struct ImageSlug {
    image: String,
}

#[get("/exists")]
async fn check_image_exist(info: web::Query<ImageSlug>) -> impl Responder {
    info.image.clone()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(health).service(check_image_exist))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
